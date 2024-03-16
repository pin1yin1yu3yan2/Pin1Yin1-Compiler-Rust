use super::definition::{FnDefinitions, VarDefinitions};

use crate::ast;

pub struct GlobalDefinitions<'ast, 's> {
    pools: Vec<LocalPool<'ast, 's>>,
    this: usize,
}

impl<'ast, 's> GlobalDefinitions<'ast, 's> {
    pub fn new() -> Self {
        let mut s = Self {
            pools: vec![],

            this: 0,
        };
        s.new_local();
        s
    }

    fn new_local_from(&mut self, parent: usize) -> &mut LocalPool<'ast, 's> {
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
        });
        &mut self.pools[new_id]
    }

    fn new_local(&mut self) -> &mut LocalPool<'ast, 's> {
        self.new_local_from(self.this)
    }

    fn this_pool(&mut self) -> &mut LocalPool<'ast, 's> {
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

/// TODO: performance optimization: use less clone() method
///
/// * take [`Statement`]'s ownership to avoid copying string
///
/// * let [`Parser`] output a better [`TypeDeclare`], like [`ast::TypeDefine`]
///
/// * ...
///
/// rebuild after pin1yin1 codes are able to run
#[cfg(feature = "parser")]
mod parse {
    struct TypedVar {
        pub value: String,
        pub type_: ast::TypeDefine,
    }

    impl TypedVar {
        fn new(val: String, type_: ast::TypeDefine) -> Self {
            Self { value: val, type_ }
        }
    }

    impl From<&ast::VarDefine> for TypedVar {
        fn from(value: &ast::VarDefine) -> Self {
            Self::new(value.name.clone(), value.type_.clone())
        }
    }

    use crate::{ast, parse::*, semantic::definition};
    use pin1yin1_parser::*;

    impl<'ast, 's> super::LocalPool<'ast, 's> {
        fn push_define(&mut self, define: ast::VarDefine) -> TypedVar {
            let tv = TypedVar::from(&define);
            self.push_stmt(ast::Statement::VarDefine(define));
            tv
        }

        fn push_sotre(&mut self, store: ast::VarStore) {
            self.push_stmt(ast::Statement::VarStore(store))
        }
    }

    impl<'ast, 's> super::GlobalDefinitions<'ast, 's> {
        pub fn load(
            &mut self,
            stmts: &'ast [PU<'s, crate::parse::Statement<'s>>],
        ) -> Result<'s, ()> {
            for stmt in stmts {
                match &**stmt {
                    Statement::FunctionCallStatement(fn_call) => {
                        self.fn_call(&fn_call.inner)?;
                    }
                    Statement::VariableStoreStatement(re_assign) => {
                        let name = re_assign.inner.name.ident.clone();
                        let value = self.expr(&re_assign.inner.assign.value)?;
                        let Some(var_def) = self.search_var(&name) else {
                            return re_assign
                                .inner
                                .throw(format!("undefined variable {}", name));
                        };
                        if var_def.type_ != value.type_ {
                            return re_assign.inner.throw(format!(
                                "tring to assign to variable with type {} from type {}",
                                var_def.type_, value.type_
                            ));
                        }

                        let value = value.value;
                        self.this_pool().push_sotre(ast::VarStore { name, value })
                    }
                    Statement::VariableDefineStatement(var_def) => {
                        let type_ = (*var_def.inner.type_).clone().into();
                        let name = var_def.inner.name.ident.clone();
                        let init = match &var_def.inner.init {
                            Some(init) => {
                                let init = self.expr(&init.value)?;
                                if init.type_ != type_ {
                                    return var_def.inner.throw(format!(
                                        "tring to define a variable with type {} from type {}",
                                        type_, init.type_
                                    ));
                                }
                                Some(ast::Expr::Variable(init.value))
                            }
                            None => None,
                        };
                        self.this_pool()
                            .push_define(ast::VarDefine { type_, name, init });
                    }
                    Statement::FunctionDefine(_) => todo!(),
                    Statement::CodeBlock(_) => todo!(),
                    Statement::If(_) => todo!(),
                    Statement::While(_) => todo!(),
                    Statement::Return(_) => todo!(),
                    Statement::Comment(_) => todo!(),
                }
            }
            Ok(())
        }

        fn fn_call(&mut self, fn_call: &PU<'s, FunctionCall>) -> Result<'s, TypedVar> {
            self.fn_call_inner(fn_call, fn_call.selection())
        }

        fn fn_call_inner(
            &mut self,
            fn_call: &FunctionCall<'s>,
            selection: &Selection<'s>,
        ) -> Result<'s, TypedVar> {
            let arguments = &fn_call.args.args;

            let mut args = vec![];
            for expr in arguments {
                let arg = self.expr(expr)?;
                args.push(arg)
            }

            let fn_def = self
                .search_fn(&fn_call.fn_name)
                .ok_or_else(|| selection.to_error("use of undefined function"))?;
            let fn_def = &fn_def.overdrives[0];
            let paramters = &fn_def.params;

            if paramters.len() != arguments.len() {
                // TODO: Error's TODO
                return selection.throw(format!(
                    "function {} exprct {} arguments, but {} arguments passed in",
                    fn_call.fn_name.ident,
                    paramters.len(),
                    arguments.len()
                ));
            }

            for idx in 0..arguments.len() {
                // TODO: Error's TODO
                if args[idx].type_ != paramters[idx] {
                    return arguments[idx].throw(format!(
                        "expected type {}, but found type {}",
                        paramters[idx], args[idx].type_
                    ));
                }
            }

            let fn_call = ast::FnCall {
                name: fn_call.fn_name.ident.clone(),
                args: args.into_iter().map(|tv| tv.value).collect(),
            };

            let type_define = fn_def.type_.clone();
            let init = ast::Expr::FuncionCall(fn_call);

            Ok(self
                .this_pool()
                .push_define(ast::VarDefine::new_alloc(type_define, init)))
        }

        fn expr(&mut self, expr: &PU<'s, Expr>) -> Result<'s, TypedVar> {
            match &**expr {
                Expr::Binary(l, o, r) => {
                    // TODO: operator overdrive
                    // only operators around same types are supported now
                    let l = self.expr(l)?;
                    let r = self.expr(r)?;
                    if l.type_ != r.type_ {
                        return expr.throw(format!(
                            "operator around different type: `{}` and `{}`!",
                            l.type_, r.type_
                        ));
                    }

                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        l.type_.clone(),
                        ast::Expr::binary(o.take(), l.value, r.value),
                    )))
                }
                Expr::Atomic(atomic) => self.atomic_expr_inner(atomic, expr.selection()),
            }
        }

        fn atomic_expr_inner(
            &mut self,
            atomic: &AtomicExpr<'s>,
            selection: &Selection<'s>,
        ) -> Result<'s, TypedVar> {
            match atomic {
                AtomicExpr::CharLiteral(char) => {
                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        ast::TypeDefine::char(),
                        ast::Expr::Char(char.parsed),
                    )))
                }
                AtomicExpr::StringLiteral(str) => {
                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        ast::TypeDefine::string(),
                        ast::Expr::String(str.parsed.clone()),
                    )))
                }
                AtomicExpr::NumberLiteral(n) => match n {
                    NumberLiteral::Float { number, .. } => {
                        Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                            ast::TypeDefine::float(),
                            ast::Expr::Float(*number),
                        )))
                    }
                    NumberLiteral::Digit { number, .. } => {
                        Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                            ast::TypeDefine::integer(),
                            ast::Expr::Integer(*number),
                        )))
                    }
                },
                AtomicExpr::Initialization(_) => todo!("how to do???"),
                AtomicExpr::FunctionCall(fn_call) => self.fn_call_inner(fn_call, selection),
                AtomicExpr::Variable(var) => {
                    let def = self.search_var(var).ok_or_else(|| {
                        selection.to_error(format!("use of undefined variable {}", &var.ident))
                    })?;
                    Ok(TypedVar::new(var.ident.clone(), def.type_.clone()))
                }
                AtomicExpr::UnaryExpr(unary) => {
                    let l = self.atomic_expr_inner(&unary.expr, unary.expr.selection())?;
                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        l.type_.clone(),
                        ast::Expr::unary(*unary.operator, l.value),
                    )))
                }
                AtomicExpr::BracketExpr(expr) => self.expr(&expr.expr),
            }
        }

        fn search_fn(&self, name: &str) -> Option<&definition::FnDefinition<'ast, 's>> {
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

        fn search_var(&self, name: &str) -> Option<&definition::VarDefinition<'ast, 's>> {
            let mut this = self.this;

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

impl Default for GlobalDefinitions<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Clone)]
pub struct LocalPool<'ast, 's> {
    // defines
    pub vars: VarDefinitions<'ast, 's>,
    pub fns: FnDefinitions<'ast, 's>,
    // statements in scope
    pub stmts: ast::Statements,
    //
    pub parent: usize,
    pub subs: Vec<usize>,
}

impl<'ast, 's> LocalPool<'ast, 's> {
    fn push_stmt(&mut self, stmt: impl Into<ast::Statement>) {
        self.stmts.push(stmt.into())
    }
}
