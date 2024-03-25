use std::collections::HashMap;

use super::definition::{FnDefinitions, VarDefinitions};

use crate::ast;

pub struct Global<'ast, 's> {
    // this kind of variables can be accessed cross fn define
    pub(crate) fns: FnDefinitions<'ast, 's>,
    pub(crate) pools: Vec<Scope<'ast, 's>>,
}

impl<'ast, 's> Global<'ast, 's> {
    pub fn new() -> Self {
        let pools = vec![Scope::new()];

        Self {
            pools,
            fns: Default::default(),
        }
    }

    pub(crate) fn this_pool(&mut self) -> &mut Scope<'ast, 's> {
        self.pools.last_mut().unwrap()
    }

    pub(crate) fn push_stmt(&mut self, stmt: impl Into<ast::Statement>) {
        self.this_pool().stmts.push(stmt.into())
    }

    pub fn finish(mut self) -> ast::Statements {
        assert!(self.pools.len() == 1, "un closed parse!?");
        self.pools.pop().unwrap().stmts
    }

    // pub fn mangle(&mut self, name: &str) {}

    pub fn push_compute<E>(&mut self, init: E) -> ast::Variable
    where
        E: Into<ast::OperateExpr>,
    {
        let name = format!("_{}", self.this_pool().alloc_id);
        self.this_pool().alloc_id += 1;

        let eval = init.into();
        let compute = ast::Compute {
            name: name.clone(),
            eval,
        };
        self.this_pool().stmts.push(compute.into());
        ast::AtomicExpr::Variable(name)
    }
}

impl<'ast, 's> Default for Global<'ast, 's> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "parser")]
mod parse {
    use super::*;
    use crate::{parse::*, semantic::definition};
    use pin1yin1_parser::*;

    impl<'ast, 's> Global<'ast, 's> {
        pub fn load(&mut self, stmts: &'ast [PU<'s, Statement<'s>>]) -> Result<'s, ()> {
            for stmt in stmts {
                self.to_ast(stmt)?;
            }
            Result::Success(())
        }

        pub(crate) fn spoce<T, F>(&mut self, f: F) -> Result<'s, (ast::Statements, T)>
        where
            F: FnOnce(&mut Self) -> Result<'s, T>,
        {
            let mut scope = Scope::new();
            scope.alloc_id = self.this_pool().alloc_id;

            self.pools.push(scope);
            let t = f(self)?;
            let pool = self.pools.pop().unwrap();

            Result::Success((pool.stmts, t))
        }

        pub(crate) fn fn_scope<T, F>(
            &mut self,
            fn_name: String,
            f: F,
        ) -> Result<'s, (ast::Statements, T)>
        where
            F: FnOnce(&mut Self) -> Result<'s, T>,
        {
            let mut scope = Scope::new();
            scope.fn_name = Some(fn_name);
            self.pools.push(scope);
            let t = f(self)?;
            let pool = self.pools.pop().unwrap();

            Result::Success((pool.stmts, t))
        }

        pub(crate) fn regist_var(
            &mut self,
            name: String,
            def: definition::VarDefinition<'ast, 's>,
        ) {
            self.this_pool().vars.map.insert(name, def);
        }

        pub(crate) fn regist_fn(&mut self, name: String, def: definition::FnDefinition<'ast, 's>) {
            self.fns.map.insert(name, def);
        }

        pub(crate) fn search_fn(&self, name: &str) -> Option<&definition::FnDefinition<'ast, 's>> {
            // overdrive is not supported now :(
            // so, the function serarching may be wrong(
            // because the function ignore the function parameters
            // the calling should select the right function with the function's parameters
            self.fns.map.get(name)
        }

        // .1: mutable
        pub(crate) fn search_var(
            &self,
            name: &str,
        ) -> Option<(&definition::VarDefinition<'ast, 's>, bool)> {
            for pool in self.pools.iter().rev() {
                if let Some(def) = pool.vars.map.get(name) {
                    return Some((def, true));
                }

                if let Some(fn_name) = &pool.fn_name {
                    let fn_def = self.search_fn(fn_name).unwrap();
                    return fn_def.overdrives[0]
                        .params
                        .iter()
                        .find(|param| param.name == name)
                        .map(|param| (&param.var_def, false));
                }
            }
            None
        }

        pub fn to_ast_inner<A: Ast<'s>>(
            &mut self,
            s: &'ast A::Target<'s>,
            selection: Selection<'s>,
        ) -> Result<'s, A::Forward> {
            A::to_ast(s, selection, self)
        }

        pub fn to_ast<A: Ast<'s>>(&mut self, pu: &'ast PU<'s, A>) -> Result<'s, A::Forward> {
            self.to_ast_inner::<A>(&**pu, pu.get_selection())
        }
    }
}

#[derive(Debug, Clone)]
// TODO
pub struct Mangle;

#[derive(Default, Debug, Clone)]
pub struct Scope<'ast, 's> {
    // defines
    pub vars: VarDefinitions<'ast, 's>,
    // TODO: static/const variable
    // this kind of var definitions are only allowed to be used in a LocalPool
    pub fn_name: Option<String>,
    // statements in scope
    pub stmts: ast::Statements,
    // a mangle for functions, variable, etc
    // TODO: no_mangle
    pub mangle: HashMap<String, Mangle>,
    // a counter
    pub alloc_id: usize,
}

impl<'ast, 's> Scope<'ast, 's> {
    pub fn new() -> Self {
        Self::default()
    }
}
