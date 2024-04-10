use super::declare::*;
use super::mangle::*;
use super::*;
use crate::parse::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use terl::*;

pub struct DefineScope<M: Mangler> {
    pub(crate) fn_signs: FnSigns,
    prefex: Vec<ManglePrefix>,
    _m: PhantomData<M>,
}

impl<M: Mangler> DefineScope<M> {
    pub fn new() -> Self {
        Self {
            fn_signs: FnSigns::default(),
            prefex: Vec::default(),
            _m: PhantomData,
        }
    }

    pub fn new_with_main() -> Self {
        // Fns::new_with_main must mangel "main" to "main"
        Self {
            fn_signs: FnSigns::new_with_main::<M>(),
            prefex: Vec::default(),
            _m: PhantomData,
        }
    }
    fn mangle_unit<'m>(&'m self, item: MangleItem<'m>) -> MangleUnit {
        MangleUnit {
            prefix: std::borrow::Cow::Borrowed(&self.prefex),
            item,
        }
    }

    pub fn mangle(&self, item: MangleItem) -> String {
        let unit = self.mangle_unit(item);
        M::mangle(unit)
    }

    pub fn mangle_ty(&self, ty: &mir::TypeDefine) -> MangleUnit {
        match ty {
            mir::TypeDefine::Primitive(pty) => self.mangle_unit(MangleItem::Type {
                ty: Cow::Owned(pty.to_string()),
            }),
            mir::TypeDefine::Complex(_) => todo!(),
        }
    }

    pub fn mangle_fn(&self, name: &str, sign: &FnSign) -> String {
        let params = sign
            .params
            .iter()
            .map(|param| self.mangle_ty(&param.ty))
            .collect::<Vec<_>>();
        self.mangle(MangleItem::Fn {
            name: Cow::Borrowed(name),
            params,
        })
    }

    pub fn get_fn(&self, res: usize) -> &FnSignWithName {
        self.fn_signs.get_fn(res)
    }

    pub fn get_mangled(&self, name: &str) -> Type {
        self.fn_signs.get_mangled(name)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<Type> {
        self.fn_signs.get_unmangled(name)
    }
}

pub struct ModScope<M: Mangler = DefaultMangler> {
    current: usize,
    fns: Vec<FnScope>,
    pub(crate) defs: DefineScope<M>,
}

impl<M: Mangler> ModScope<M> {
    pub fn new() -> Self {
        Self {
            current: 0,
            fns: vec![FnScope::new("main")],
            defs: DefineScope::new_with_main(),
        }
    }

    pub fn regist_fn(&mut self, name: String, sign: FnSign) -> Type {
        let mangled = self.defs.mangle_fn(&name, &sign);
        self.defs.fn_signs.new_fn(name, mangled, sign)
    }

    pub fn create_fn<F>(&mut self, name: String, sign: FnSign, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let mangled = self.defs.mangle_fn(&name, &sign);
        self.defs
            .fn_signs
            .new_fn(name.to_owned(), mangled.clone(), sign);

        self.current += 1;
        self.fns.push(FnScope::new(mangled));
        f(self)?;
        self.current -= 1;

        Ok(())
    }

    pub fn search_var(&self, name: &str) -> Option<defs::VarDef> {
        if let Some(param) = self
            .defs
            .get_fn(self.current)
            .params
            .iter()
            .find(|param| param.name == name)
        {
            return Some(defs::VarDef::new(
                Type::Owned(param.ty.clone()),
                param.loc,
                false,
            ));
        }

        let fn_scope = self.local();

        for scope in fn_scope.scope_stack.iter().rev() {
            if let Some(var_def) = scope.vars.get(name) {
                return Some(var_def.clone());
            }
        }

        todo!()
    }

    fn local(&self) -> &FnScope {
        &self.fns[self.current]
    }

    fn local_mut(&mut self) -> &mut FnScope {
        &mut self.fns[self.current]
    }

    // from FnScope
    pub fn spoce<T, F>(&mut self, f: F) -> Result<(mir::Statements, T)>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let scope = Default::default();

        self.local_mut().scope_stack.push(scope);
        let t = f(self)?;
        let pool = self.local_mut().scope_stack.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub fn solve_decalre(&mut self) -> Result<()> {
        todo!()
    }

    pub fn build_overload_declare<B>(&mut self, builder: B) -> terl::Result<GroupIdx>
    where
        B: FnOnce(&DefineScope<M>) -> GroupBuilder<M>,
    {
        let builder = builder(&self.defs);
        self.fns[self.current]
            .declare_map
            .new_group::<declare::kind::Overload, M>(&self.defs, builder)
    }

    pub fn load_stmts(&mut self, stmts: &[PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }

    pub fn to_ast_inner<A>(&mut self, s: &A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<M>,
    {
        A::to_ast(s, span, self)
    }

    pub fn to_ast<A>(&mut self, pu: &PU<A>) -> Result<A::Forward>
    where
        A: Ast<M>,
    {
        self.to_ast_inner::<A>(&**pu, pu.get_span())
    }
}

/// a scope that represents a fn's local scope
///
/// [`DeclareMap`] is used to picking overloads, declare var's types etc
///
/// un processed ast move into this struct and then become `mir`, mir misses
/// a part of type information, and fn_call is not point to monomorphic fn
///
/// these message will be filled by [`DeclareMap`], or a [`Error`] will be thrown
#[derive(Default)]
pub struct FnScope {
    // mangled
    pub fn_name: String,
    // a counter
    pub alloc_id: usize,
    pub scope_stack: Vec<BasicScope>,
    pub declare_map: DeclareMap,
}

impl FnScope {
    pub fn new(fn_name: impl ToString) -> Self {
        Self {
            fn_name: fn_name.to_string(),
            scope_stack: Default::default(),
            alloc_id: Default::default(),
            declare_map: Default::default(),
        }
    }

    fn this_scope(&mut self) -> &mut BasicScope {
        self.scope_stack.last_mut().unwrap()
    }

    pub(crate) fn push_stmt(&mut self, stmt: impl Into<mir::Statement>) {
        self.this_scope().stmts.push(stmt.into())
    }

    pub fn finish(mut self) -> mir::Statements {
        assert!(self.scope_stack.len() == 1, "unclosed parse!?");
        self.scope_stack.pop().unwrap().stmts
    }
}

/// usually be folded into other structs,like FnDef, If, While...
pub struct BasicScope {
    // defines
    pub vars: HashMap<String, VarDef>,
    // statements in scope
    pub stmts: mir::Statements,
}

impl Default for BasicScope {
    fn default() -> Self {
        Self {
            vars: Default::default(),
            stmts: Default::default(),
        }
    }
}
