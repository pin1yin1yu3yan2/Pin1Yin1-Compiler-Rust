use super::declare::*;
use super::mangle::*;
use super::*;
use crate::ir;
use crate::parse::*;
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

pub struct ModScope<M: Mangler = DefaultMangler> {
    prefex: Vec<ManglePrefix>,
    // mangled
    currt_fn: String,
    // name: unmangled
    fns: HashMap<String, FnDef>,
    _m: PhantomData<M>,
}

impl<M: Mangler> ModScope<M> {
    pub fn new() -> Self {
        let main_sign = FnSign {
            mangle: String::new(),
            ty: ir::TypeDefine::Primitive(ir::PrimitiveType::I32),
            params: vec![],
            // no location
            loc: Span::new(0, 0),
        };

        let main = FnDef {
            overloads: vec![main_sign],
        };

        let fns = [(String::from("main"), main)].into_iter().collect();

        Self {
            prefex: vec![],
            currt_fn: String::from("main"),
            fns,
            _m: PhantomData,
        }
    }

    pub fn mangle(&self, item: MangleItem) -> String {
        let unit = MangleUnit {
            prefix: std::borrow::Cow::Borrowed(&self.prefex),
            item,
        };
        M::mangle(unit)
    }
}

impl<M: Mangler> Scope for ModScope<M> {}

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
    pub fn_name: String,
    // a counter
    pub alloc_id: usize,
    pub scope_stack: Vec<BasicScope>,
    pub declare_map: DeclareMap,
    pub parameters: HashMap<String, VarDef>, // TODO: Nested definitions
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
            parameters: Default::default(),
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

    pub fn load(&mut self, stmts: &[PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }

    pub(crate) fn spoce<T, F>(&mut self, f: F) -> Result<(ir::Statements, T)>
    where
        F: FnOnce(&mut FnScope) -> Result<T>,
    {
        let scope = Default::default();

        self.scope_stack.push(scope);
        let t = f(self)?;
        let pool = self.scope_stack.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub(crate) fn regist_var(&mut self, name: String, def: defs::VarDef) {
        self.this_scope().vars.insert(name, def);
    }

    // pub(crate) fn regist_fn(&mut self, name: String, def: defs::FnDef<>) {
    //     self.fns.map.insert(name, def);
    // }

    // pub(crate) fn search_fn(&self, name: &str) -> Option<&defs::FnDef<>> {
    //     self.fns.map.get(name)
    // }

    // .1: mutable
    pub(crate) fn search_var(&self, name: &str) -> Option<(&defs::VarDef, bool)> {
        if let Some(param) = self.parameters.get(name) {
            return Some((param, false));
        }

        for pool in self.scope_stack.iter().rev() {
            if let Some(def) = pool.vars.get(name) {
                return Some((def, true));
            }
        }
        None
    }
}
