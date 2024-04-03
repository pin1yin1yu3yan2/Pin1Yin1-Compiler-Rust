use super::declare::*;
use super::mangle::*;
use super::*;
use crate::ir;
use crate::parse::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use terl::*;

/// Marker
pub trait Scope<'ast>: Sized {
    fn to_ast_inner<A>(&mut self, s: &'ast A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<'ast, Self>;

    fn to_ast<A>(&mut self, pu: &'ast PU<A>) -> Result<A::Forward>
    where
        A: Ast<'ast, Self>,
    {
        self.to_ast_inner::<A>(&**pu, pu.get_span())
    }
}

pub struct ModScope<'ast, M: Mangler = DefaultMangler> {
    current: ManglePrefix,
    fns: HashMap<String, FnDef<'ast>>,
    _p: PhantomData<M>,
}

impl<'ast, M: Mangler> Scope<'ast> for ModScope<'ast, M> {
    fn to_ast_inner<A>(&mut self, s: &'ast A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<'ast, Self>,
    {
        todo!()
    }
}

/// a scope that represents a fn's local scope
#[derive(Default)]
pub struct FnScope<'ast> {
    pub fn_name: String,
    // a counter
    pub alloc_id: usize,
    pub scope_stack: Vec<BasicScope<'ast>>,
    pub declare_map: DeclareMap,
    pub parameters: HashMap<String, VarDef<'ast>>, // TODO: Nested definitions
}

impl<'ast> Scope<'ast> for FnScope<'ast> {
    fn to_ast_inner<A>(&mut self, s: &'ast A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<'ast, Self>,
    {
        A::to_ast(s, span, self)
    }
}

/// usually be folded into other structs,like FnDef, If, While...
pub struct BasicScope<'ast> {
    // defines
    pub vars: HashMap<String, VarDef<'ast>>,
    // statements in scope
    pub stmts: ir::Statements,
}

impl<'ast> Default for BasicScope<'ast> {
    fn default() -> Self {
        Self {
            vars: Default::default(),
            stmts: Default::default(),
        }
    }
}

impl<'ast> FnScope<'ast> {
    pub fn new(fn_name: impl ToString) -> Self {
        Self {
            fn_name: fn_name.to_string(),
            scope_stack: Default::default(),
            alloc_id: Default::default(),
            declare_map: Default::default(),
            parameters: Default::default(),
        }
    }

    pub(crate) fn this_scope(&mut self) -> &mut BasicScope<'ast> {
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

    pub fn load(&mut self, stmts: &'ast [PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }

    pub(crate) fn spoce<T, F>(&mut self, f: F) -> Result<(ir::Statements, T)>
    where
        F: FnOnce(&mut FnScope<'ast>) -> Result<T>,
    {
        let scope = Default::default();

        self.scope_stack.push(scope);
        let t = f(self)?;
        let pool = self.scope_stack.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub(crate) fn regist_var(&mut self, name: String, def: defs::VarDef<'ast>) {
        self.this_scope().vars.insert(name, def);
    }

    // pub(crate) fn regist_fn(&mut self, name: String, def: defs::FnDef<'ast>) {
    //     self.fns.map.insert(name, def);
    // }

    // pub(crate) fn search_fn(&self, name: &str) -> Option<&defs::FnDef<'ast>> {
    //     self.fns.map.get(name)
    // }

    // .1: mutable
    pub(crate) fn search_var(&self, name: &str) -> Option<(&defs::VarDef<'ast>, bool)> {
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
