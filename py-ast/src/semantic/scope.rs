use super::mangle::*;
use py_declare::*;
use py_lex::SharedString;
use std::collections::HashMap;
use terl::*;

pub struct Defines<M: Mangle = DefaultMangler> {
    pub defs: Defs,
    pub mangler: Mangler<M>,
}

impl<M: Mangle> Defines<M> {
    /// # Return
    ///
    /// mangled_name
    pub fn regist_fn(
        &mut self,
        fn_define: &crate::parse::FnDefine,
        fn_sign: defs::FnSign,
    ) -> Result<SharedString> {
        let mangled_name = SharedString::from(self.mangler.mangle_fn(&fn_define.name, &fn_sign));

        if let Some(previous) = self.defs.try_get_mangled(&mangled_name) {
            let previous_define = previous
                .sign_span
                .make_message(format!("funcion {} has been definded here", fn_define.name));
            let mut err = fn_sign
                .sign_span
                .make_error(format!("double define for function {}", fn_define.name))
                .append(previous_define);
            if previous.ty != fn_sign.ty {
                err += "note: overload which only return type is differnet is not allowed";
            }
            err += format!("note: if you want to overload funcion {}, you can define them with different parameters",fn_define.name);
            return Err(err);
        }

        self.defs
            .new_fn(fn_define.name.shared(), mangled_name.clone(), fn_sign);
        Ok(mangled_name)
    }
}

impl<M: Mangle> Default for Defines<M> {
    fn default() -> Self {
        Self {
            defs: Default::default(),
            mangler: Default::default(),
        }
    }
}

impl<M: Mangle> Defines<M> {
    pub fn new(defs: Defs, mangler: Mangler<M>) -> Self {
        Self { defs, mangler }
    }
}

/// a scope that represents a fn's local scope
///
/// [`DeclareMap`] is used to picking overloads, declare var's types, etc
///
/// un processed ast move into this struct and then become [`mir::Statements`], mir misses
/// a part of type information, and fn_call is not point to monomorphic fn
///
///
/// then [`DeclareMap`] will declare them and output [`py_ir::Statements`],  or a [`Error`] will be thrown
#[derive(Default)]
pub struct FnScope {
    // mangled
    pub fn_name: String,
    // a counter
    temps: usize,
    parameters: HashMap<SharedString, GroupIdx>,
    pub declare_map: DeclareMap,
}

impl FnScope {
    pub fn new<'p, PI, SI>(fn_name: impl ToString, params: PI, spans: SI) -> Self
    where
        PI: IntoIterator<Item = &'p defs::Parameter>,
        SI: IntoIterator<Item = Span>,
    {
        let mut declare_map = DeclareMap::default();
        let parameters = spans
            .into_iter()
            .zip(params)
            .map(|(at, param)| {
                (
                    param.name.clone(),
                    declare_map.new_static_group(at, std::iter::once(param.ty.clone().into())),
                )
            })
            .collect();

        Self {
            fn_name: fn_name.to_string(),
            parameters,
            declare_map,
            ..Default::default()
        }
    }

    #[inline]
    pub fn temp_name(&mut self) -> SharedString {
        // whitespace to make temp name will not be accessed
        (format!("_{}", self.temps), self.temps += 1).0.into()
    }

    #[inline]
    pub fn search_parameter(&mut self, name: &str) -> Option<defs::VarDef> {
        self.parameters.get(name).map(|ty| defs::VarDef {
            ty: *ty,
            mutable: false,
        })
    }
}

#[derive(Default)]
pub struct BasicScope {
    // defines
    pub vars: HashMap<String, defs::VarDef>,
}

pub struct BasicScopes {
    // defines
    scopes: Vec<BasicScope>,
}

impl std::ops::DerefMut for BasicScopes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scopes
    }
}

impl std::ops::Deref for BasicScopes {
    type Target = Vec<BasicScope>;

    fn deref(&self) -> &Self::Target {
        &self.scopes
    }
}

impl Default for BasicScopes {
    fn default() -> Self {
        Self {
            scopes: vec![BasicScope::default()],
        }
    }
}

impl BasicScopes {
    #[inline]
    pub fn search_variable(&mut self, name: &str) -> Option<defs::VarDef> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var.clone());
            }
        }
        None
    }

    fn current_scope(&mut self) -> &mut BasicScope {
        self.scopes.last_mut().unwrap()
    }

    pub fn regist_variable(&mut self, name: impl ToString, def: defs::VarDef) {
        let name = name.to_string();
        let current = self.current_scope();
        current.vars.insert(name, def);
    }
}
