use crate::ast;
use crate::parse;
use crate::semantic::definition;
use crate::semantic::definition_pool::GlobalPool;
use pin1yin1_parser::*;

pub trait Ast<'s>: ParseUnit {
    type Forward;

    /// this function may return nothing because the ast will be put in [`LocalPool`]
    fn to_ast<'ast>(
        s: &'ast PU<'s, Self>,
        global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        Self::to_ast_inner(&**s, s.get_selection(), global)
    }

    /// divided [`PU`] into [`ParseUnit::Target`] and [`Selection`] becase
    /// variants from [`crate::complex_pu`] isnot [`PU`], and the [`Selection`]
    /// was stored in the enum
    fn to_ast_inner<'ast>(
        s: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward>;
}

pub struct TypedVar {
    val: String,
    ty: ast::TypeDefine,
}

impl TypedVar {
    fn new(val: String, ty: ast::TypeDefine) -> Self {
        Self { val, ty }
    }
}

impl From<ast::VarDefine> for TypedVar {
    fn from(value: ast::VarDefine) -> Self {
        Self::new(value.name, value.ty)
    }
}

impl<'s> Ast<'s> for parse::Statement<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        stmt: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match stmt {
            parse::Statement::FnCallStmt(fn_call) => {
                parse::FnCall::to_ast(&fn_call.inner, _global)?;
            }
            parse::Statement::VarStoreStmt(var_store) => {
                parse::VarStore::to_ast(&var_store.inner, _global)?;
            }
            parse::Statement::FnDefine(fn_define) => {
                parse::FnDefine::to_ast_inner(fn_define, _selection, _global)?;
            }
            parse::Statement::VarDefineStmt(var_define) => {
                parse::VarDefine::to_ast(&var_define.inner, _global)?;
            }
            parse::Statement::If(if_) => {
                parse::If::to_ast_inner(if_, _selection, _global)?;
            }
            parse::Statement::While(while_) => {
                parse::While::to_ast_inner(while_, _selection, _global)?;
            }
            parse::Statement::Return(return_) => {
                parse::Return::to_ast_inner(return_, _selection, _global)?;
            }
            parse::Statement::CodeBlock(block) => {
                parse::CodeBlock::to_ast_inner(block, _selection, _global)?;
            }
            parse::Statement::Comment(_) => {}
        }
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::FnCall<'s> {
    type Forward = TypedVar;

    fn to_ast_inner<'ast>(
        fn_call: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let args = fn_call
            .args
            .args
            .iter()
            .try_fold(vec![], |mut args, expr| {
                args.push(parse::Expr::to_ast(expr, _global)?);
                Result::Success(args)
            })?;

        let fn_def = Result::from_option(_global.search_fn(&fn_call.fn_name), || {
            _selection.throw("use of undefined function")
        })?;

        // TOOD: overdrive support
        let fn_def = &fn_def.overdrives[0];
        let params = &fn_def.params;

        if params.len() != args.len() {
            return _selection.throw(format!(
                "function {} exprct {} arguments, but {} arguments passed in",
                fn_call.fn_name.ident,
                params.len(),
                args.len()
            ));
        }

        for arg_idx in 0..args.len() {
            if args[arg_idx].ty != params[arg_idx] {
                return fn_call.args.args[arg_idx].throw(format!(
                    "expected type {}, but found type {}",
                    params[arg_idx], args[arg_idx].ty
                ));
            }
        }

        let fn_call = ast::FnCall {
            name: fn_call.fn_name.ident.clone(),
            args: args.into_iter().map(|tv| tv.val).collect(),
        };

        let ty = fn_def.ty.clone();
        let init = ast::Expr::FuncionCall(fn_call);

        Result::Success(
            _global
                .this_pool()
                .push_define(ast::VarDefine::new_alloc(ty, init)),
        )
    }
}

impl<'s> Ast<'s> for parse::VarStore<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        var_store: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let name = var_store.name.ident.clone();
        let val = parse::Expr::to_ast(&var_store.assign.value, _global)?;
        let Some(var_def) = _global.search_var(&name) else {
            return _selection.throw(format!("use of undefined variable {}", name));
        };
        if var_def.ty != val.ty {
            return _selection.throw(format!(
                "tring to assign to variable with type {} from type {}",
                var_def.ty, val.ty
            ));
        }
        _global
            .this_pool()
            .push_sotre(ast::VarStore { name, val: val.val });
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::FnDefine<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        fn_define: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let fn_name = fn_define.function.name.ident.clone();
        if let Some(exist) = _global.search_fn(&fn_name) {
            return exist.raw_defines[0]
                .function
                .throw("overdrive is not supported now...");
        }

        let ret_ty: ast::TypeDefine = fn_define.function.ty.to_ast_ty()?;

        let params = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                vec.push(ast::TypeDefine::try_from((*pu.inner.ty).clone())?);
                Result::Success(vec)
            })?;

        let fn_sign = definition::FnSign::new(ret_ty.clone(), params);

        _global.this_pool().fns.map.insert(
            fn_name.clone(),
            definition::FnDefinition::new(vec![fn_sign], vec![fn_define]),
        );
        // generate ast
        let ty = ret_ty;
        let name = fn_name;

        let params = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                let ty = ast::TypeDefine::try_from((*pu.inner.ty).clone())?;
                let name = pu.inner.name.ident.clone();
                vec.push(ast::Parameter { ty, name });
                Result::Success(vec)
            })?;

        // regist parameters
        for (idx, param) in params.iter().enumerate() {
            _global.regist_var(
                param.name.clone(),
                definition::VarDefinition::new(
                    param.ty.clone(),
                    &fn_define.params.params[idx].inner,
                ),
            )
        }

        let body = parse::CodeBlock::to_ast(&fn_define.codes, _global)?;
        _global
            .this_pool()
            .push_stmt(ast::Statement::FnDefine(ast::FnDefine {
                ty,
                name,
                params,
                body,
            }));

        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::VarDefine<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        var_define: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let ty = var_define.ty.to_ast_ty()?;
        let name = var_define.name.ident.clone();
        let init = match &var_define.init {
            Some(init) => {
                let init = parse::Expr::to_ast(&init.value, _global)?;
                if init.ty != ty {
                    return _selection.throw(format!(
                        "tring to define a variable with type {} from type {}",
                        ty, init.ty
                    ));
                }
                Some(ast::Expr::Variable(init.val))
            }
            None => None,
        };

        _global.regist_var(
            name.clone(),
            definition::VarDefinition::new(ty.clone(), var_define),
        );

        _global
            .this_pool()
            .push_define(ast::VarDefine { ty, name, init });

        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::If<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        if_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let mut branches = vec![parse::AtomicIf::to_ast(&if_.ruo4, _global)?];
        for chain in &if_.chains {
            match &**chain {
                parse::ChainIf::AtomicElseIf(atomic) => {
                    branches.push(parse::AtomicIf::to_ast(&atomic.ruo4, _global)?);
                }
                parse::ChainIf::AtomicElse(else_) => {
                    let else_ = parse::CodeBlock::to_ast(&else_.block, _global)?;
                    _global.this_pool().push_stmt(ast::Statement::If(ast::If {
                        branches,
                        else_: Some(else_),
                    }));
                    return Result::Success(());
                }
            }
        }
        _global.this_pool().push_stmt(ast::Statement::If(ast::If {
            branches,
            else_: None,
        }));
        return Result::Success(());
    }
}

impl<'s> Ast<'s> for parse::While<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        while_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let cond = parse::Arguments::to_ast(&while_.conds, _global)?;
        let body = parse::CodeBlock::to_ast(&while_.block, _global)?;
        _global
            .this_pool()
            .push_stmt(ast::Statement::While(ast::While { cond, body }));
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::AtomicIf<'s> {
    type Forward = ast::IfBranch;

    fn to_ast_inner<'ast>(
        atomic: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let cond = parse::Arguments::to_ast(&atomic.conds, _global)?;
        let body = parse::CodeBlock::to_ast(&atomic.block, _global)?;
        Result::Success(ast::IfBranch { cond, body })
    }
}

impl<'s> Ast<'s> for parse::Return<'s> {
    type Forward = ();

    fn to_ast_inner<'ast>(
        return_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let val = match &return_.val {
            Some(val) => Some(parse::Expr::to_ast(val, _global)?.val),
            None => None,
        };
        _global
            .this_pool()
            .push_stmt(ast::Statement::Return(ast::Return { val }));
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::CodeBlock<'s> {
    type Forward = ast::Statements;

    fn to_ast_inner<'ast>(
        block: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        _global
            .spoce(|_global| {
                for stmt in &block.stmts {
                    parse::Statement::to_ast(stmt, _global);
                }
                Result::Success(())
            })
            .map(|(v, _)| v)
    }
}

// TODO: condition`s`
impl<'s> Ast<'s> for parse::Arguments<'s> {
    type Forward = ast::Condition;

    fn to_ast_inner<'ast>(
        cond: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let (compute, last_cond) = _global.spoce(|_global| {
            let mut last_cond = parse::Expr::to_ast(&cond.args[0], _global)?;
            for arg in cond.args.iter().skip(1) {
                last_cond = parse::Expr::to_ast(arg, _global)?;
            }
            Result::Success(last_cond)
        })?;

        if last_cond.ty != ast::TypeDefine::bool() {
            return cond.args.last().unwrap().throw("condition must be boolean");
        }

        Result::Success(ast::Condition {
            val: last_cond.val,
            compute,
        })
    }
}

impl<'s> Ast<'s> for parse::Expr<'s> {
    type Forward = TypedVar;

    fn to_ast_inner<'ast>(
        expr: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match expr {
            parse::Expr::Atomic(atomic) => {
                parse::AtomicExpr::to_ast_inner(atomic, _selection, _global)
            }
            parse::Expr::Binary(l, o, r) => {
                let l = parse::Expr::to_ast(l, _global)?;
                let r = parse::Expr::to_ast(r, _global)?;
                if l.ty != r.ty {
                    return _selection.throw(format!(
                        "operator around different type: `{}` and `{}`!",
                        l.ty, r.ty
                    ));
                }

                let ty = if o.ty() == crate::keywords::operators::OperatorTypes::CompareOperator {
                    ast::TypeDefine::bool()
                } else {
                    l.ty.clone()
                };

                Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                    ty,
                    ast::Expr::binary(o.take(), l.val, r.val),
                )))
            }
        }
    }
}

impl<'s> Ast<'s> for parse::AtomicExpr<'s> {
    type Forward = TypedVar;

    fn to_ast_inner<'ast>(
        atomic: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut GlobalPool<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match atomic {
            parse::AtomicExpr::CharLiteral(char) => {
                Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                    ast::TypeDefine::char(),
                    ast::Expr::Char(char.parsed),
                )))
            }
            parse::AtomicExpr::StringLiteral(str) => {
                Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                    ast::TypeDefine::string(),
                    ast::Expr::String(str.parsed.clone()),
                )))
            }
            parse::AtomicExpr::NumberLiteral(n) => match n {
                parse::NumberLiteral::Float { number, .. } => {
                    Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                        ast::TypeDefine::float(),
                        ast::Expr::Float(*number),
                    )))
                }
                parse::NumberLiteral::Digit { number, .. } => {
                    Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                        ast::TypeDefine::integer(),
                        ast::Expr::Integer(*number),
                    )))
                }
            },
            parse::AtomicExpr::FnCall(fn_call) => {
                parse::FnCall::to_ast_inner(fn_call, _selection, _global)
            }
            parse::AtomicExpr::Variable(var) => {
                let def = Result::from_option(_global.search_var(var), || {
                    _selection.throw("use of undefined variable")
                })?;
                Result::Success(TypedVar::new(var.ident.clone(), def.ty.clone()))
            }
            parse::AtomicExpr::UnaryExpr(unary) => {
                let l = parse::AtomicExpr::to_ast(&unary.expr, _global)?;
                Result::Success(_global.this_pool().push_define(ast::VarDefine::new_alloc(
                    l.ty.clone(),
                    ast::Expr::unary(*unary.operator, l.val),
                )))
            }
            parse::AtomicExpr::BracketExpr(expr) => parse::Expr::to_ast(&expr.expr, _global),
            parse::AtomicExpr::Initialization(_) => todo!("how to do???"),
        }
    }
}
