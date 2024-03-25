use crate::ast;
use crate::parse;
use crate::semantic;
use crate::semantic::Global;
use pin1yin1_parser::*;

pub trait Ast<'s>: ParseUnit {
    type Forward;

    /// divided [`PU`] into [`ParseUnit::Target`] and [`Selection`] becase
    /// variants from [`crate::complex_pu`] isnot [`PU`], and the [`Selection`]
    /// was stored in the enum
    fn to_ast<'ast>(
        s: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward>;
}

// pub struct TypedVar {
//     val: String,
//     ty: ast::TypeDefine,
// }

// impl TypedVar {
//     pub(crate) fn new(val: String, ty: ast::TypeDefine) -> Self {
//         Self { val, ty }
//     }
// }

// impl From<ast::VarDefine> for TypedVar {
//     fn from(value: ast::VarDefine) -> Self {
//         Self::new(value.name, value.ty)
//     }
// }

impl<'s> Ast<'s> for parse::Statement<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        stmt: &'ast Self::Target<'s>,
        selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match stmt {
            parse::Statement::FnCallStmt(fn_call) => {
                _global.to_ast(&fn_call.inner)?;
            }
            parse::Statement::VarStoreStmt(var_store) => {
                _global.to_ast(&var_store.inner)?;
            }
            parse::Statement::FnDefine(fn_define) => {
                parse::FnDefine::to_ast(fn_define, selection, _global)?;
            }
            parse::Statement::VarDefineStmt(var_define) => {
                _global.to_ast(&var_define.inner)?;
            }
            parse::Statement::If(if_) => {
                _global.to_ast_inner::<parse::If>(if_, selection)?;
            }
            parse::Statement::While(while_) => {
                _global.to_ast_inner::<parse::While>(while_, selection)?;
            }
            parse::Statement::Return(return_) => {
                _global.to_ast_inner::<parse::Return>(return_, selection)?;
            }
            parse::Statement::CodeBlock(block) => {
                _global.to_ast_inner::<parse::CodeBlock>(block, selection)?;
            }
            parse::Statement::Comment(_) => {}
        }
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::FnCall<'s> {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        fn_call: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let args = fn_call
            .args
            .args
            .iter()
            .try_fold(vec![], |mut args, expr| {
                args.push(_global.to_ast(expr)?);
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
            args: args.into_iter().collect(),
        };

        let ty = fn_def.ty.clone();
        let init = ast::AtomicExpr::FnCall(fn_call);
        let define = _global.alloc_var(ty, init);
        Result::Success(_global.push_define(define))
    }
}

impl<'s> Ast<'s> for parse::VarStore<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        var_store: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let name = var_store.name.ident.clone();
        // function parameters are inmutable

        let val = _global.to_ast(&var_store.assign.val)?;
        let Some((var_def, mutable)) = _global.search_var(&name) else {
            return _selection.throw(format!("use of undefined variable {}", name));
        };
        if !mutable {
            return _selection.throw(format!("cant assign to a immmutable variable {}", name));
        }
        if var_def.ty != val.ty {
            return _selection.throw(format!(
                "tring to assign to variable with type {} from type {}",
                var_def.ty, val.ty
            ));
        }
        _global.push_stmt(ast::VarStore { name, val });
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::FnDefine<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        fn_define: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let fn_name = fn_define.function.name.ident.clone();
        if let Some(exist) = _global.search_fn(&fn_name) {
            return exist.raw_defines[0]
                .function
                .throw("overdrive is not supported now...");
        }

        let ret_ty: ast::TypeDefine = fn_define.function.ty.to_ast_ty()?;

        let types = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                vec.push(ast::TypeDefine::try_from((*pu.inner.ty).clone())?);
                Result::Success(vec)
            })?;

        // do this step firstly to allow recursion
        // mangle should follow the `mangle rule` (not exist now)
        // the mangle is the unique id of the function because overdrive allow fns with same name but different sign

        // TODO: `mangle rule`
        let fn_sign = semantic::FnSign::new(fn_name.clone(), ret_ty.clone(), types.clone());
        let fn_def = semantic::FnDefinition::new(vec![fn_sign], vec![fn_define]);
        _global.regist_fn(fn_name.clone(), fn_def);

        // generate ast
        let ty = ret_ty;
        let name = fn_name;

        let params = types
            .into_iter()
            .enumerate()
            .try_fold(Vec::new(), |mut vec, (idx, ty)| {
                let name = fn_define.params.params[idx].inner.name.ident.clone();
                vec.push(ast::Parameter { ty, name });
                Result::Success(vec)
            })?;

        // regist parameters
        let params_iter = params.iter().enumerate().map(|(idx, param)| {
            (
                param.name.clone(),
                semantic::VarDefinition::new(param.ty.clone(), &fn_define.params.params[idx].inner),
            )
        });

        // use this because funtion cant access out variables

        // TODO: global variables support
        let body = _global
            .fn_scope(|_global| {
                _global.regist_params(params_iter);
                for stmt in &fn_define.codes.stmts {
                    _global.to_ast(stmt)?;
                }
                Result::Success(())
            })?
            .0;
        _global.push_stmt(ast::Statement::FnDefine(ast::FnDefine {
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

    fn to_ast<'ast>(
        var_define: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let ty = var_define.ty.to_ast_ty()?;
        let name = var_define.name.ident.clone();
        let init = match &var_define.init {
            Some(init) => {
                let init = _global.to_ast(&init.val)?;
                if init.ty != ty {
                    return _selection.throw(format!(
                        "tring to define a variable with type {} from type {}",
                        ty, init.ty
                    ));
                }
                Some(init.val.into())
            }
            None => None,
        };

        _global.regist_var(
            name.clone(),
            semantic::VarDefinition::new(ty.clone(), var_define),
        );

        _global.push_define(ast::VarDefine { ty, name, init });

        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::If<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        if_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let mut branches = vec![_global.to_ast(&if_.ruo4)?];
        for chain in &if_.chains {
            match &**chain {
                parse::ChainIf::AtomicElseIf(atomic) => {
                    branches.push(_global.to_ast(&atomic.ruo4)?);
                }
                parse::ChainIf::AtomicElse(else_) => {
                    let else_ = _global.to_ast(&else_.block)?;
                    _global.push_stmt(ast::Statement::If(ast::If {
                        branches,
                        else_: Some(else_),
                    }));
                    return Result::Success(());
                }
            }
        }
        _global.push_stmt(ast::Statement::If(ast::If {
            branches,
            else_: None,
        }));
        return Result::Success(());
    }
}

impl<'s> Ast<'s> for parse::While<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        while_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let cond = _global.to_ast(&while_.conds)?;
        let body = _global.to_ast(&while_.block)?;
        _global.push_stmt(ast::Statement::While(ast::While { cond, body }));
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::AtomicIf<'s> {
    type Forward = ast::IfBranch;

    fn to_ast<'ast>(
        atomic: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let cond = _global.to_ast(&atomic.conds)?;
        let body = _global.to_ast(&atomic.block)?;
        Result::Success(ast::IfBranch { cond, body })
    }
}

impl<'s> Ast<'s> for parse::Return<'s> {
    type Forward = ();

    fn to_ast<'ast>(
        return_: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let val = match &return_.val {
            Some(val) => Some(_global.to_ast(val)?),
            None => None,
        };
        _global.push_stmt(ast::Statement::Return(ast::Return { val }));
        Result::Success(())
    }
}

impl<'s> Ast<'s> for parse::CodeBlock<'s> {
    type Forward = ast::Statements;

    fn to_ast<'ast>(
        block: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        _global
            .spoce(|_global| {
                for stmt in &block.stmts {
                    _global.to_ast(stmt)?;
                }
                Result::Success(())
            })
            .map(|(v, _)| v)
    }
}

// TODO: condition`s`
impl<'s> Ast<'s> for parse::Arguments<'s> {
    type Forward = ast::Condition;

    fn to_ast<'ast>(
        cond: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        let (compute, last_cond) = _global.spoce(|_global| {
            let mut last_cond = _global.to_ast(&cond.args[0])?;
            for arg in cond.args.iter().skip(1) {
                last_cond = _global.to_ast(arg)?;
            }
            Result::Success(last_cond)
        })?;

        if last_cond.ty != ast::TypeDefine::bool() {
            return cond.args.last().unwrap().throw("condition must be boolean");
        }

        Result::Success(ast::Condition {
            val: last_cond,
            compute,
        })
    }
}

impl<'s> Ast<'s> for parse::Expr<'s> {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        expr: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match expr {
            parse::Expr::Atomic(atomic) => parse::AtomicExpr::to_ast(atomic, _selection, _global),
            parse::Expr::Binary(l, o, r) => {
                let l = _global.to_ast(l)?;
                let r = _global.to_ast(r)?;

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

                let define =
                    _global.alloc_var(ty, ast::OperateExpr::binary(o.take(), l.val, r.val));
                Result::Success(_global.push_define(define))
            }
        }
    }
}

impl<'s> Ast<'s> for parse::AtomicExpr<'s> {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        atomic: &'ast Self::Target<'s>,
        _selection: Selection<'s>,
        _global: &mut Global<'ast, 's>,
    ) -> Result<'s, Self::Forward> {
        match atomic {
            parse::AtomicExpr::CharLiteral(char) => {
                let define =
                    _global.alloc_var(ast::TypeDefine::char(), ast::AtomicExpr::Char(char.parsed));
                Result::Success(_global.push_define(define))
            }
            parse::AtomicExpr::StringLiteral(str) => {
                let define = _global.alloc_var(
                    ast::TypeDefine::string(),
                    ast::AtomicExpr::String(str.parsed.clone()),
                );
                Result::Success(_global.push_define(define))
            }
            parse::AtomicExpr::NumberLiteral(n) => {
                let defint = match n {
                    parse::NumberLiteral::Float { number, .. } => {
                        _global.alloc_var(ast::TypeDefine::float(), ast::AtomicExpr::Float(*number))
                    }
                    parse::NumberLiteral::Digit { number, .. } => _global.alloc_var(
                        ast::TypeDefine::integer(),
                        ast::AtomicExpr::Integer(*number),
                    ),
                };
                Result::Success(_global.push_define(defint))
            }
            parse::AtomicExpr::FnCall(fn_call) => {
                parse::FnCall::to_ast(fn_call, _selection, _global)
            }
            parse::AtomicExpr::Variable(var) => {
                let def = Result::from_option(_global.search_var(var), || {
                    _selection.throw("use of undefined variable")
                })
                .map(|(def, _m)| def)?;
                Result::Success(
                    ast::AtomicExpr::Variable(var.ident.clone()).with_ty(def.ty.clone()),
                )
            }

            // here, this is incorrect because operators may be overdriven
            // all operator overdriven must be casted into function calling here but primitives
            parse::AtomicExpr::UnaryExpr(unary) => {
                let l = _global.to_ast(&unary.expr)?;
                let define = _global.alloc_var(
                    l.ty.clone(),
                    ast::OperateExpr::unary(*unary.operator, l.val),
                );
                Result::Success(_global.push_define(define))
            }
            parse::AtomicExpr::BracketExpr(expr) => _global.to_ast(&expr.expr),
            parse::AtomicExpr::Initialization(_) => todo!("how to do???"),
        }
    }
}
