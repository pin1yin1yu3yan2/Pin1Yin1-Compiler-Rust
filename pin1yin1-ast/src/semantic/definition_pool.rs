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
        pub ty: ast::TypeDefine,
    }

    impl TypedVar {
        fn new(val: String, ty: ast::TypeDefine) -> Self {
            Self { value: val, ty }
        }
    }

    impl From<&ast::VarDefine> for TypedVar {
        fn from(value: &ast::VarDefine) -> Self {
            Self::new(value.name.clone(), value.ty.clone())
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
                        self.fn_call(&fn_call.inner).map(|_| ())
                    }
                    Statement::VariableStoreStatement(re_assign) => {
                        self.var_store(&re_assign.inner)
                    }

                    Statement::VariableDefineStatement(var_def) => self.var_define(&var_def.inner),
                    Statement::CodeBlock(code_block) => self.code_block(&code_block.stmts),
                    Statement::FunctionDefine(fn_def) => self.fn_define(fn_def),
                    Statement::If(if_) => self.if_(if_),
                    Statement::While(while_) => self.while_(while_),
                    Statement::Return(return_) => self.return_(return_),
                    Statement::Comment(..) => Ok(()),
                }?;
            }
            Ok(())
        }

        fn fn_define(&mut self, fn_define: &'ast FunctionDefine<'s>) -> Result<'s, ()> {
            let fn_name = fn_define.function.name.ident.clone();

            if let Some(exist) = self.search_fn(&fn_name) {
                return Err(exist.raw_defines[0]
                    .function
                    .failed_error("overdrive is not supported now..."));
            }

            let ret_t = (*fn_define.function.ty).clone().into();
            let params = fn_define
                .params
                .params
                .iter()
                .map(|pu| (*pu.ty).clone().into())
                .collect::<Vec<_>>();
            let fn_sign = definition::FnSign::new(ret_t, params);

            self.this_pool().fns.map.insert(
                fn_name,
                definition::FnDefinition::new(vec![fn_sign], vec![fn_define]),
            );

            Ok(())
        }

        fn return_(&mut self, return_: &Return<'s>) -> Result<'s, ()> {
            let val = match &return_.val {
                Some(val) => Some(self.expr(val)?.value),
                None => None,
            };
            self.this_pool()
                .push_stmt(ast::Statement::Return(ast::Return { val }));
            Ok(())
        }

        fn while_(&mut self, while_: &'ast While<'s>) -> Result<'s, ()> {
            let cond = self.condition(&while_.conds)?;
            let body = self.spoce(|s| s.load(&while_.block.stmts))?.0;
            self.this_pool()
                .push_stmt(ast::Statement::While(ast::While { cond, body }));
            Ok(())
        }

        fn if_(&mut self, if_: &'ast If<'s>) -> Result<'s, ()> {
            let mut branches = vec![self.atomic_if(&if_.ruo4)?];

            for chain in &if_.chains {
                match &**chain {
                    ChainIf::AtomicElseIf(atomic_if) => {
                        branches.push(self.atomic_if(&atomic_if.ruo4)?)
                    }
                    ChainIf::AtomicElse(else_) => {
                        let else_ = self.spoce(|s| s.load(&else_.block.stmts))?.0;
                        self.this_pool().push_stmt(ast::Statement::If(ast::If {
                            branches,
                            else_: Some(else_),
                        }));
                        return Ok(());
                    }
                }
            }

            self.this_pool().push_stmt(ast::Statement::If(ast::If {
                branches,
                else_: None,
            }));
            Ok(())
        }

        fn atomic_if(
            &mut self,
            atomic_if: &'ast PU<'s, AtomicIf<'s>>,
        ) -> Result<'s, ast::IfBranch> {
            let cond = self.condition(&atomic_if.conds)?;
            let body = self.spoce(|s| s.load(&atomic_if.block.stmts))?.0;
            Ok(ast::IfBranch { cond, body })
        }

        // TODO: condition`s`
        fn condition(&mut self, cond: &Arguments<'s>) -> Result<'s, ast::Condition> {
            let this = self.this;

            self.new_local();
            let new = self.this;
            let mut last_cond = self.expr(&cond.args[0])?;
            for arg in cond.args.iter().skip(1) {
                last_cond = self.expr(arg)?;
            }

            self.this = this;

            if last_cond.ty != ast::TypeDefine::bool() {
                return Err(cond
                    .args
                    .last()
                    .unwrap()
                    .failed_error("condition must be a boolen!"));
            }

            let stmts = self.pools[new].stmts.drain(..).collect::<Vec<_>>();

            Ok(ast::Condition {
                value: last_cond.value,
                compute: stmts,
            })
        }

        fn code_block(
            &mut self,
            code_block: &'ast [PU<'s, crate::parse::Statement<'s>>],
        ) -> Result<'s, ()> {
            let stmts = self.spoce(|s| s.load(code_block))?;
            let stmt = ast::Statement::Block(stmts.0);
            self.this_pool().push_stmt(stmt);
            Ok(())
        }

        fn spoce<T, F>(&mut self, f: F) -> Result<'s, (ast::Statements, T)>
        where
            F: FnOnce(&mut Self) -> Result<'s, T>,
        {
            let this = self.this;
            self.new_local();

            let t = f(self)?;
            let stmts = std::mem::take(&mut self.this_pool().stmts);

            self.this = this;
            Ok((stmts, t))
        }

        fn var_define(&mut self, var_define: &'ast PU<'s, VariableDefine<'s>>) -> Result<'s, ()> {
            let ty = (*var_define.ty).clone().into();
            let name = var_define.name.ident.clone();
            let init = match &var_define.init {
                Some(init) => {
                    let init = self.expr(&init.value)?;
                    if init.ty != ty {
                        return Err(var_define.failed_error(format!(
                            "tring to define a variable with type {} from type {}",
                            ty, init.ty
                        )));
                    }
                    Some(ast::Expr::Variable(init.value))
                }
                None => None,
            };

            self.this_pool().vars.map.insert(
                name.clone(),
                definition::VarDefinition::new(ty.clone(), var_define),
            );
            self.this_pool()
                .push_define(ast::VarDefine { ty, name, init });

            Ok(())
        }

        fn var_store(&mut self, var_store: &PU<'s, VariableStore<'s>>) -> Result<'s, ()> {
            let name = var_store.name.ident.clone();
            let value = self.expr(&var_store.assign.value)?;
            let Some(var_def) = self.search_var(&name) else {
                return Err(var_store.failed_error(format!("undefined variable {}", name)));
            };
            if var_def.ty != value.ty {
                return Err(var_store.failed_error(format!(
                    "tring to assign to variable with type {} from type {}",
                    var_def.ty, value.ty
                )));
            }

            let value = value.value;
            self.this_pool().push_sotre(ast::VarStore { name, value });
            Ok(())
        }

        fn fn_call(&mut self, fn_call: &PU<'s, FunctionCall>) -> Result<'s, TypedVar> {
            self.fn_call_inner(fn_call, &fn_call.get_selection())
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
                .ok_or_else(|| selection.failed_error("use of undefined function"))?;
            let fn_def = &fn_def.overdrives[0];
            let paramters = &fn_def.params;

            if paramters.len() != arguments.len() {
                // TODO: Error's TODO
                return Err(selection.failed_error(format!(
                    "function {} exprct {} arguments, but {} arguments passed in",
                    fn_call.fn_name.ident,
                    paramters.len(),
                    arguments.len()
                )));
            }

            for idx in 0..arguments.len() {
                // TODO: Error's TODO
                if args[idx].ty != paramters[idx] {
                    return Err(arguments[idx].failed_error(format!(
                        "expected type {}, but found type {}",
                        paramters[idx], args[idx].ty
                    )));
                }
            }

            let fn_call = ast::FnCall {
                name: fn_call.fn_name.ident.clone(),
                args: args.into_iter().map(|tv| tv.value).collect(),
            };

            let type_define = fn_def.ty.clone();
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
                    if l.ty != r.ty {
                        return Err(expr.failed_error(format!(
                            "operator around different type: `{}` and `{}`!",
                            l.ty, r.ty
                        )));
                    }

                    let ty = if o.ty() == crate::keywords::operators::OperatorTypes::CompareOperator
                    {
                        ast::TypeDefine::bool()
                    } else {
                        l.ty.clone()
                    };

                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        ty,
                        ast::Expr::binary(o.take(), l.value, r.value),
                    )))
                }
                Expr::Atomic(atomic) => self.atomic_expr_inner(atomic, &expr.get_selection()),
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
                        selection.failed_error(format!("use of undefined variable {}", &var.ident))
                    })?;
                    Ok(TypedVar::new(var.ident.clone(), def.ty.clone()))
                }
                AtomicExpr::UnaryExpr(unary) => {
                    let l = self.atomic_expr_inner(&unary.expr, &unary.expr.get_selection())?;
                    Ok(self.this_pool().push_define(ast::VarDefine::new_alloc(
                        l.ty.clone(),
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
