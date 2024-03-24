use std::collections::HashMap;

use super::definition::{FnDefinitions, VarDefinitions};

use crate::ast;

pub struct Global<'ast, 's> {
    pub(crate) pools: Vec<Scope<'ast, 's>>,
}

impl<'ast, 's> Global<'ast, 's> {
    pub fn new() -> Self {
        let pools = vec![Scope::new()];

        Self { pools }
    }

    pub(crate) fn this_pool(&mut self) -> &mut Scope<'ast, 's> {
        self.pools.last_mut().unwrap()
    }

    pub(crate) fn push_stmt(&mut self, stmt: impl Into<ast::Statement>) {
        self.this_pool().stmts.push(stmt.into())
    }

    pub(crate) fn push_define(&mut self, define: ast::VarDefine) -> crate::parse::TypedVar {
        let typed_var = crate::parse::TypedVar::new(define.name.clone(), define.ty.clone());
        self.push_stmt(define);
        typed_var
    }

    pub fn finish(mut self) -> ast::Statements {
        assert!(self.pools.len() == 1, "un closed parse!?");
        self.pools.pop().unwrap().stmts
    }

    // pub fn mangle(&mut self, name: &str) {}

    pub fn alloc_var(
        &mut self,
        ty: ast::TypeDefine,
        init: impl Into<Option<ast::Expr>>,
    ) -> ast::VarDefine {
        let def = ast::VarDefine {
            ty,
            init: init.into(),
            name: self.this_pool().alloc_id.to_string(),
        };
        self.this_pool().alloc_id += 1;
        def
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

        pub(crate) fn fn_scope<T, F>(&mut self, f: F) -> Result<'s, (ast::Statements, T)>
        where
            F: FnOnce(&mut Self) -> Result<'s, T>,
        {
            self.pools.push(Scope::new());
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

        pub(crate) fn regist_params<I>(&mut self, defs: I)
        where
            I: IntoIterator<Item = (String, definition::VarDefinition<'ast, 's>)>,
        {
            assert!(self.this_pool().params.is_none());

            let defs = definition::VarDefinitions {
                map: defs.into_iter().collect(),
            };
            self.this_pool().params = Some(defs);
        }

        pub(crate) fn regist_fn(&mut self, name: String, def: definition::FnDefinition<'ast, 's>) {
            self.this_pool().fns.map.insert(name, def);
        }

        pub(crate) fn search_fn(&self, name: &str) -> Option<&definition::FnDefinition<'ast, 's>> {
            // overdrive is not supported now :(
            // so, the function serarching may be wrong(
            // because the function ignore the function parameters
            // the calling should select the right function with the function's parameters

            for pool in self.pools.iter().rev() {
                if let Some(def) = pool.fns.map.get(name) {
                    return Some(def);
                }
            }
            None
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

                if pool.params.is_some() {
                    return pool
                        .params
                        .as_ref()
                        .unwrap()
                        .map
                        .get(name)
                        .map(|def| (def, false));
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
    // this kind of variables can be accessed cross fn define
    pub fns: FnDefinitions<'ast, 's>,
    // this kind of var definitions are only allowed to be used in a LocalPool
    pub params: Option<VarDefinitions<'ast, 's>>,
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
