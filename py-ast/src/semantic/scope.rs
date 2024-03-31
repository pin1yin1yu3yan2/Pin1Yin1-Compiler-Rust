use super::defs::*;
use super::Ast;
use super::Type;
use crate::ir;
use crate::{parse::*, semantic::defs};
use std::collections::HashMap;
use terl::*;

pub struct GlobalScope<'ast> {
    // this kind of variables can be accessed cross fn define
    pub(crate) fns: FnDefs<'ast>,
    pub(crate) tys: DeclareMap<Type>,
    pub(crate) pools: Vec<Scope<'ast>>,
}

impl<'ast> GlobalScope<'ast> {
    pub fn new() -> Self {
        let pools = vec![Scope::new()];

        Self {
            pools,
            tys: Default::default(),
            fns: Default::default(),
        }
    }

    pub(crate) fn this_pool(&mut self) -> &mut Scope<'ast> {
        self.pools.last_mut().unwrap()
    }

    pub(crate) fn push_stmt(&mut self, stmt: impl Into<ir::Statement>) {
        self.this_pool().stmts.push(stmt.into())
    }

    pub fn finish(mut self) -> ir::Statements {
        assert!(self.pools.len() == 1, "un closed parse!?");
        self.pools.pop().unwrap().stmts
    }

    // pub fn mangle(&mut self, name: &str) {}

    pub fn push_compute<T, E>(&mut self, ty: T, init: E) -> ir::Variable
    where
        T: Into<ir::PrimitiveType>,
        E: Into<ir::OperateExpr>,
    {
        let name = format!("_{}", self.this_pool().alloc_id);
        self.this_pool().alloc_id += 1;

        let eval = init.into();
        let compute = ir::Compute {
            ty: ty.into(),
            name: name.clone(),
            eval,
        };
        self.this_pool().stmts.push(compute.into());
        ir::AtomicExpr::Variable(name)
    }
}

impl<'ast> Default for GlobalScope<'ast> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ast> GlobalScope<'ast> {
    pub fn load(&mut self, stmts: &'ast [PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }

    pub(crate) fn spoce<T, F>(&mut self, f: F) -> Result<(ir::Statements, T)>
    where
        F: FnOnce(&mut GlobalScope<'ast>) -> Result<T>,
    {
        let mut scope = Scope::new();
        scope.alloc_id = self.this_pool().alloc_id;

        self.pools.push(scope);
        let t = f(self)?;
        let pool = self.pools.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub(crate) fn fn_scope<T, F>(&mut self, fn_name: String, f: F) -> Result<(ir::Statements, T)>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let mut scope = Scope::new();
        scope.fn_name = Some(fn_name);
        self.pools.push(scope);
        let t = f(self)?;
        let pool = self.pools.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub(crate) fn regist_var(&mut self, name: String, def: defs::VarDef<'ast>) {
        self.this_pool().vars.map.insert(name, def);
    }

    pub(crate) fn regist_fn(&mut self, name: String, def: defs::FnDef<'ast>) {
        self.fns.map.insert(name, def);
    }

    pub(crate) fn search_fn(&self, name: &str) -> Option<&defs::FnDef<'ast>> {
        // overload is not supported now :(
        // so, the function serarching may be wrong(
        // because the function ignore the function parameters
        // the calling should select the right function with the function's parameters
        self.fns.map.get(name)
    }

    // .1: mutable
    pub(crate) fn search_var(&self, name: &str) -> Option<(&defs::VarDef<'ast>, bool)> {
        for pool in self.pools.iter().rev() {
            if let Some(def) = pool.vars.map.get(name) {
                return Some((def, true));
            }

            if let Some(fn_name) = &pool.fn_name {
                let fn_def = self.search_fn(fn_name).unwrap();
                return fn_def.overloads[0]
                    .params
                    .iter()
                    .find(|param| param.name == name)
                    .map(|param| (&param.var_def, false));
            }
        }
        None
    }

    pub fn to_ast_inner<A: Ast>(&mut self, s: &'ast A::Target, span: Span) -> Result<A::Forward> {
        A::to_ast(s, span, self)
    }

    pub fn to_ast<A: Ast>(&mut self, pu: &'ast PU<A>) -> Result<A::Forward> {
        self.to_ast_inner::<A>(&**pu, pu.get_span())
    }
}

#[derive(Debug, Clone)]
// TODO
pub struct Mangle;

#[derive(Default)]
pub struct Scope<'ast> {
    // defines
    pub vars: VarDefs<'ast>,
    // TODO: static/const variable
    // this kind of var definitions are only allowed to be used in a LocalPool
    pub fn_name: Option<String>,
    // statements in scope
    pub stmts: ir::Statements,
    // a mangle for functions, variable, etc
    // TODO: no_mangle
    pub mangle: HashMap<String, Mangle>,
    // a counter
    pub alloc_id: usize,
}

impl<'ast> Scope<'ast> {
    pub fn new() -> Self {
        Self::default()
    }
}
