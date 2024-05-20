use crate::*;
use py_ir::types::TypeDefine;
use py_lex::SharedString;
use std::collections::HashMap;
use terl::Span;

#[derive(Default)]
pub struct Defs {
    pub(crate) fn_signs: FnSigns,
}

impl Defs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_fn(
        &mut self,
        unmangled: SharedString,
        mangled: SharedString,
        sign: defs::FnSign,
    ) -> Type {
        self.fn_signs.new_fn(unmangled, mangled, sign)
    }

    pub fn get_mangled(&self, name: &str) -> &Overload {
        self.fn_signs.get_mangled(name)
    }

    pub fn try_get_mangled(&self, name: &str) -> Option<&Overload> {
        self.fn_signs.try_get_mangled(name)
    }

    pub fn get_unmangled(&self, name: &str) -> Option<&[Overload]> {
        self.fn_signs.get_unmangled(name)
    }
}

#[derive(Default)]
pub struct FnSigns {
    fn_signs: Vec<Overload>,
    unmangled: HashMap<SharedString, Vec<Overload>>,
    mangled: HashMap<SharedString, Overload>,
}

impl FnSigns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_fn(
        &mut self,
        unmangled: SharedString,
        mangled: SharedString,
        sign: defs::FnSign,
    ) -> Type {
        let value = defs::FnSignWithName {
            sign,
            name: mangled.clone(),
        };

        let overload: Overload = value.into();

        self.fn_signs.push(overload.clone());
        self.unmangled
            .entry(unmangled)
            .or_default()
            .push(overload.clone());
        self.mangled.insert(mangled, overload.clone());
        Type::Overload(overload)
    }

    pub fn get_unmangled(&self, name: &str) -> Option<&[Overload]> {
        self.unmangled.get(name).map(|v| &**v)
    }

    pub fn get_mangled(&self, name: &str) -> &Overload {
        self.mangled.get(name).unwrap()
    }

    pub fn try_get_mangled(&self, name: &str) -> Option<&Overload> {
        self.mangled.get(name)
    }

    // pub fn search_fns
}

#[derive(Debug, Clone)]
pub struct FnSignWithName {
    pub sign: FnSign,
    pub name: SharedString,
}

impl std::ops::Deref for FnSignWithName {
    type Target = FnSign;

    fn deref(&self) -> &Self::Target {
        &self.sign
    }
}

impl std::ops::DerefMut for FnSignWithName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sign
    }
}

#[derive(Debug, Clone)]
pub struct FnSign {
    /// return type of the function must be cleared (will change in future versions)
    pub ty: TypeDefine,
    pub params: Vec<Parameter>,
    pub retty_span: Span,
    pub sign_span: Span,
}

impl FnSign {
    pub fn new(ty: TypeDefine, params: Vec<Parameter>, retty_span: Span, sign_span: Span) -> Self {
        Self {
            ty,
            params,
            retty_span,
            sign_span,
        }
    }
}

impl std::fmt::Display for FnSignWithName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // NOTO: this may only work with default Chinese Mangler
        let unmangled = self.name.split_ascii_whitespace().next().unwrap();
        f.write_str(unmangled)?;
        f.write_str("(")?;
        match self.params.len() {
            0 => f.write_str(")")?,
            1 => f.write_fmt(format_args!("{})", self.params[0].ty))?,
            _ => {
                f.write_fmt(format_args!("{}", self.params[0].ty))?;
                for param in &self.params[1..] {
                    f.write_fmt(format_args!(", {}", param.name))?;
                }
                f.write_str(")")?
            }
        }
        f.write_fmt(format_args!(" -> {}", self.ty))
    }
}

pub type Parameter = py_ir::Parameter<TypeDefine>;

#[derive(Debug, Clone)]
pub struct VarDef {
    pub ty: GroupIdx,
    pub mutable: bool,
}
