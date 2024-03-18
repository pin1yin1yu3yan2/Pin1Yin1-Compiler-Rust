use super::definition::{FnDefinitions, VarDefinitions};

use crate::ast;

pub struct GlobalPool<'ast, 's> {
    pub(crate) pools: Vec<LocalPool<'ast, 's>>,
    pub(crate) this: usize,
}

impl<'ast, 's> GlobalPool<'ast, 's> {
    pub fn new() -> Self {
        let mut s = Self {
            pools: vec![],
            this: 0,
        };
        s.new_local();
        s
    }

    pub(crate) fn new_local_from(&mut self, parent: usize) -> &mut LocalPool<'ast, 's> {
        let new_id = self.pools.len();
        if let Some(parent) = self.pools.get_mut(parent) {
            parent.subs.push(new_id);
        }
        self.pools.push(LocalPool {
            parent,
            stmts: vec![],
            subs: vec![],
            vars: Default::default(),
            fns: Default::default(),
            params: Default::default(),
        });
        &mut self.pools[new_id]
    }

    pub(crate) fn new_local(&mut self) -> &mut LocalPool<'ast, 's> {
        self.new_local_from(self.this)
    }

    pub(crate) fn this_pool(&mut self) -> &mut LocalPool<'ast, 's> {
        &mut self.pools[self.this]
    }

    fn finish_inner(&mut self, stmts: &mut ast::Statements, idx: usize) {
        stmts.append(&mut self.pools[idx].stmts);
        let idxs = self.pools[idx].subs.clone();
        for idx in idxs {
            self.finish_inner(stmts, idx);
        }
    }

    pub fn finish(mut self) -> ast::Statements {
        let mut stmts = vec![];
        self.finish_inner(&mut stmts, 0);
        stmts
    }
}

#[cfg(feature = "parser")]
mod parse {

    use crate::{ast, parse::*, semantic::definition};
    use pin1yin1_parser::*;

    impl<'ast, 's> super::LocalPool<'ast, 's> {
        pub(crate) fn push_stmt(&mut self, stmt: impl Into<ast::Statement>) {
            self.stmts.push(stmt.into())
        }

        pub(crate) fn push_define(&mut self, define: ast::VarDefine) -> TypedVar {
            let tv = TypedVar::from(define.clone());
            self.push_stmt(ast::Statement::VarDefine(define));
            tv
        }

        pub(crate) fn push_sotre(&mut self, store: ast::VarStore) {
            self.push_stmt(ast::Statement::VarStore(store))
        }
    }

    impl<'ast, 's> super::GlobalPool<'ast, 's> {
        pub fn load(&mut self, stmts: &'ast [PU<'s, Statement<'s>>]) -> Result<'s, ()> {
            for stmt in stmts {
                crate::parse::Statement::to_ast(stmt, self);
            }
            Result::Success(())
        }

        pub(crate) fn spoce<T, F>(&mut self, f: F) -> Result<'s, (ast::Statements, T)>
        where
            F: FnOnce(&mut Self) -> Result<'s, T>,
        {
            let this = self.this;
            self.new_local();

            let t = f(self)?;
            let stmts = std::mem::take(&mut self.this_pool().stmts);

            self.this = this;
            Result::Success((stmts, t))
        }

        pub(crate) fn regist_var(
            &mut self,
            name: String,
            def: definition::VarDefinition<'ast, 's>,
        ) {
            self.this_pool().vars.map.insert(name.clone(), def);
        }

        pub(crate) fn search_fn(&self, name: &str) -> Option<&definition::FnDefinition<'ast, 's>> {
            // overdrive is not supported now
            // so, the function serarching may be wrong(

            let mut this = self.this;
            loop {
                match self.pools[this].fns.map.get(name) {
                    Some(def) => return Some(def),
                    None => {
                        if this == 0 {
                            return None;
                        }
                        this = self.pools[this].parent
                    }
                }
            }
        }

        pub(crate) fn search_var(
            &self,
            name: &str,
        ) -> Option<&definition::VarDefinition<'ast, 's>> {
            let mut this = self.this;
            if let Some(def) = self.pools[this].params.map.get(name) {
                return Some(def);
            }
            loop {
                match self.pools[this].vars.map.get(name) {
                    Some(def) => return Some(def),
                    None => {
                        if this == 0 {
                            return None;
                        }
                        this = self.pools[this].parent
                    }
                }
            }
        }
    }
}

impl Default for GlobalPool<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Clone)]
pub struct LocalPool<'ast, 's> {
    // defines
    pub vars: VarDefinitions<'ast, 's>,
    pub fns: FnDefinitions<'ast, 's>,
    // this kind of var definitions are only allowed to be used in a LocalPool
    pub params: VarDefinitions<'ast, 's>,
    // statements in scope
    pub stmts: ast::Statements,
    //
    pub parent: usize,
    pub subs: Vec<usize>,
}
