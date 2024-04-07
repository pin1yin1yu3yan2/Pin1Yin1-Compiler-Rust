use super::declare::*;
use super::mangle::*;
use super::*;
use crate::ir;
use crate::parse::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use terl::*;

/// Marker
pub trait Scope: Sized {
    fn to_ast_inner<A>(&mut self, s: &A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<Self>,
    {
        A::to_ast(s, span, self)
    }

    fn to_ast<A>(&mut self, pu: &PU<A>) -> Result<A::Forward>
    where
        A: Ast<Self>,
    {
        self.to_ast_inner::<A>(&**pu, pu.get_span())
    }
}

pub struct DefineScope<M: Mangler> {
    fn_signs: FnSigns,
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

    pub fn mangle_ty(&self, ty: &ir::TypeDefine) -> MangleUnit {
        match ty {
            ir::TypeDefine::Primitive(pty) => self.mangle_unit(MangleItem::Type {
                ty: Cow::Owned(pty.to_string()),
            }),
            ir::TypeDefine::Complex(_) => todo!(),
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

    pub fn get_fn(&self, res: TypeRes) -> &FnSignWithName {
        self.fn_signs.get_fn(res)
    }
}

pub struct ModScope<M: Mangler = DefaultMangler> {
    current: usize,
    fns: Vec<FnScope>,
    defs: DefineScope<M>,
}

impl<M: Mangler> ModScope<M> {
    pub fn new() -> Self {
        Self {
            current: 0,
            fns: vec![FnScope::new("main")],
            defs: DefineScope::new_with_main(),
        }
    }

    pub fn regist_fn(&mut self, name: String, sign: FnSign) -> TypeRes {
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

    // from FnScope
    pub fn spoce<T, F>(&mut self, f: F) -> Result<(ir::Statements, T)>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let scope = Default::default();

        self.scope_stack.push(scope);
        let t = f(self)?;
        let pool = self.scope_stack.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub fn solve_decalre(&mut self) -> Result<()> {
        todo!()
    }

    pub fn function_overload_declare(&self, fn_name: &str) -> Vec<TypeRes> {
        self.defs.fn_signs.get_unmangled(fn_name)
    }

    pub fn delcare<K>(&mut self, benches: Vec<BenchBuilder<M>>) -> terl::Result<GroupIdx>
    where
        K: DeclareKind,
    {
        // no-deref, or compiler error
        self.fns[self.current]
            .declare_map
            .new_group::<K, _, _>(&self.defs, benches)
    }

    pub fn load_stmts(&mut self, stmts: &[PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }
}

impl<M: Mangler> Scope for ModScope<M> {}

impl<M: Mangler> std::ops::Deref for ModScope<M> {
    type Target = FnScope;

    fn deref(&self) -> &Self::Target {
        self.fns.last().unwrap()
    }
}

impl<M: Mangler> std::ops::DerefMut for ModScope<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fns.last_mut().unwrap()
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

impl Scope for FnScope {
    fn to_ast_inner<A>(&mut self, s: &A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<Self>,
    {
        A::to_ast(s, span, self)
    }
}

/// usually be folded into other structs,like FnDef, If, While...
pub struct BasicScope {
    // defines
    pub vars: HashMap<String, VarDef>,
    // statements in scope
    pub stmts: ir::Statements,
}

impl Default for BasicScope {
    fn default() -> Self {
        Self {
            vars: Default::default(),
            stmts: Default::default(),
        }
    }
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

    pub(crate) fn this_scope(&mut self) -> &mut BasicScope {
        self.scope_stack.last_mut().unwrap()
    }

    pub(crate) fn push_stmt(&mut self, stmt: impl Into<ir::Statement>) {
        self.this_scope().stmts.push(stmt.into())
    }

    pub fn finish(mut self) -> ir::Statements {
        assert!(self.scope_stack.len() == 1, "unclosed parse!?");
        self.scope_stack.pop().unwrap().stmts
    }

    // pub fn mangle(&mut self, name: &str) {}

    pub fn push_compute<T, E>(&mut self, ty: T, init: E) -> ir::Variable
    where
        T: Into<ir::PrimitiveType>,
        E: Into<ir::OperateExpr>,
    {
        let name = format!("_{}", self.alloc_id);
        self.alloc_id += 1;

        let eval = init.into();
        let compute = ir::Compute {
            ty: ty.into(),
            name: name.clone(),
            eval,
        };
        self.this_scope().stmts.push(compute.into());
        ir::AtomicExpr::Variable(name)
    }
}
