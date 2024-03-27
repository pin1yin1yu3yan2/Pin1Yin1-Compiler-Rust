use crate::ast;
use crate::parse;
use crate::semantic;
use crate::semantic::Global;
use pin1yin1_parser::*;

pub trait Ast: ParseUnit {
    type Forward;

    /// divided [`PU`] into [`ParseUnit::Target`] and [`Span`] becase
    /// variants from [`crate::complex_pu`] isnot [`PU`], and the [`Span`]
    /// was stored in the enum
    fn to_ast<'ast>(
        s: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward>;
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

impl Ast for parse::Statement {
    type Forward = ();

    fn to_ast<'ast>(
        stmt: &'ast Self::Target,
        selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
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
        Result::Ok(())
    }
}

impl Ast for parse::FnCall {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        fn_call: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let (tys, vals): (Vec<_>, Vec<_>) = fn_call
            .args
            .args
            .iter()
            .try_fold(vec![], |mut args, expr| {
                let expr = _global.to_ast(expr)?;
                args.push((expr.ty, expr.val));
                Result::Ok(args)
            })?
            .into_iter()
            .unzip();

        let Some(fn_def) = _global.search_fn(&fn_call.fn_name) else {
            return _selection.throw("use of undefined function");
        };

        // TOOD: overdrive support
        let fn_def = &fn_def.overdrives[0];
        let params = &fn_def.params;

        if params.len() != vals.len() {
            return _selection.throw(format!(
                "function {} exprct {} arguments, but {} arguments passed in",
                fn_call.fn_name.0,
                params.len(),
                vals.len()
            ));
        }

        for arg_idx in 0..vals.len() {
            if tys[arg_idx] != params[arg_idx].ty {
                return fn_call.args.args[arg_idx].throw(format!(
                    "expected type {}, but found type {}",
                    params[arg_idx].ty, tys[arg_idx]
                ));
            }
        }

        let fn_call = ast::FnCall {
            name: fn_call.fn_name.0.clone(),
            args: vals.into_iter().collect(),
        };

        let ty = fn_def.ty.clone();
        let fn_call = ast::AtomicExpr::FnCall(fn_call);
        Result::Ok(ast::TypedExpr::new(ty, fn_call))
    }
}

impl Ast for parse::VarStore {
    type Forward = ();

    fn to_ast<'ast>(
        var_store: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let name = var_store.name.0.clone();
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
        _global.push_stmt(ast::VarStore { name, val: val.val });
        Result::Ok(())
    }
}

impl Ast for parse::FnDefine {
    type Forward = ();

    fn to_ast<'ast>(
        fn_define: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let fn_name = fn_define.name.0.clone();

        if let Some(exist) = _global.search_fn(&fn_name) {
            return exist.raw_defines[0]
                .name
                .throw("overdrive is not supported now...");
        }

        // do this step firstly to allow recursion
        // mangle should follow the `mangle rule` (not exist now)
        // the mangle is the unique id of the function because overdrive allow fns with same name but different sign
        let ret_ty: ast::TypeDefine = fn_define.ty.to_ast_ty()?;

        let params = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                let ty = pu.inner.ty.to_ast_ty()?;
                let name = pu.inner.name.0.clone();
                vec.push(ast::Parameter { ty, name });
                Result::Ok(vec)
            })?;

        let sign_params =
            params
                .iter()
                .cloned()
                .enumerate()
                .try_fold(Vec::new(), |mut vec, (idx, param)| {
                    let raw = &fn_define.params.params[idx].inner;
                    let param = semantic::Parameter {
                        name: param.name,
                        var_def: semantic::VarDefinition::new(param.ty, raw),
                        _p: std::marker::PhantomData,
                    };
                    vec.push(param);
                    Result::Ok(vec)
                })?;

        // TODO: `mangle rule`
        let fn_sign = semantic::FnSign {
            mangle: fn_name.clone(),
            ty: ret_ty.clone(),
            params: sign_params,
        };

        let fn_def = semantic::FnDefinition::new(vec![fn_sign], vec![fn_define]);
        _global.regist_fn(fn_name.clone(), fn_def);

        // generate ast
        let ty = ret_ty;
        let name = fn_name;

        let body = _global
            .fn_scope(name.clone(), |_global| {
                // _global.regist_params(params_iter);
                for stmt in &fn_define.codes.stmts {
                    _global.to_ast(stmt)?;
                }
                Result::Ok(())
            })?
            .0;
        _global.push_stmt(ast::Statement::FnDefine(ast::FnDefine {
            ty,
            name,
            params,
            body,
        }));

        Result::Ok(())
    }
}

impl Ast for parse::VarDefine {
    type Forward = ();

    fn to_ast<'ast>(
        var_define: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        // TODO: if ty exist
        let ty = var_define.ty.to_ast_ty()?;
        let name = var_define.name.0.clone();
        let init = match &var_define.init {
            Some(init) => {
                let init = _global.to_ast(&init.val)?;
                if init.ty != ty {
                    return _selection.throw(format!(
                        "tring to define a variable with type {} from type {}",
                        ty, init.ty
                    ));
                }
                Some(init.val)
            }
            None => None,
        };

        _global.regist_var(
            name.clone(),
            semantic::VarDefinition::new(ty.clone(), var_define),
        );

        _global.push_stmt(ast::VarDefine { ty, name, init });

        Result::Ok(())
    }
}

impl Ast for parse::If {
    type Forward = ();

    fn to_ast<'ast>(
        if_: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
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
                    return Result::Ok(());
                }
            }
        }
        _global.push_stmt(ast::Statement::If(ast::If {
            branches,
            else_: None,
        }));
        Result::Ok(())
    }
}

impl Ast for parse::While {
    type Forward = ();

    fn to_ast<'ast>(
        while_: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let cond = _global.to_ast(&while_.conds)?;
        let body = _global.to_ast(&while_.block)?;
        _global.push_stmt(ast::Statement::While(ast::While { cond, body }));
        Result::Ok(())
    }
}

impl Ast for parse::AtomicIf {
    type Forward = ast::IfBranch;

    fn to_ast<'ast>(
        atomic: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let cond = _global.to_ast(&atomic.conds)?;
        let body = _global.to_ast(&atomic.block)?;
        Result::Ok(ast::IfBranch { cond, body })
    }
}

impl Ast for parse::Return {
    type Forward = ();

    fn to_ast<'ast>(
        return_: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let val = match &return_.val {
            Some(val) => Some(_global.to_ast(val)?),
            None => None,
        };
        _global.push_stmt(ast::Statement::Return(ast::Return {
            val: val.map(|v| v.val),
        }));
        Result::Ok(())
    }
}

impl Ast for parse::CodeBlock {
    type Forward = ast::Statements;

    fn to_ast<'ast>(
        block: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        _global
            .spoce(|_global| {
                for stmt in &block.stmts {
                    _global.to_ast(stmt)?;
                }
                Result::Ok(())
            })
            .map(|(v, _)| v)
    }
}

// TODO: condition`s`
impl Ast for parse::Arguments {
    type Forward = ast::Condition;

    fn to_ast<'ast>(
        cond: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        let (compute, last_cond) = _global.spoce(|_global| {
            let mut last_cond = _global.to_ast(&cond.args[0])?;
            for arg in cond.args.iter().skip(1) {
                last_cond = _global.to_ast(arg)?;
            }
            Result::Ok(last_cond)
        })?;

        if last_cond.ty != ast::TypeDefine::bool() {
            return cond.args.last().unwrap().throw("condition must be boolean");
        }

        Result::Ok(ast::Condition {
            val: last_cond.val,
            compute,
        })
    }
}

impl Ast for parse::Expr {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        expr: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
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

                let define = _global.push_compute(ast::OperateExpr::binary(o.take(), l.val, r.val));
                Result::Ok(define.with_ty(ty))
            }
        }
    }
}

impl Ast for parse::AtomicExpr {
    type Forward = ast::TypedExpr;

    fn to_ast<'ast>(
        atomic: &'ast Self::Target,
        _selection: Span,
        _global: &mut Global<'ast>,
    ) -> Result<Self::Forward> {
        match atomic {
            parse::AtomicExpr::CharLiteral(char) => {
                Result::Ok(ast::AtomicExpr::Char(char.parsed).with_ty(ast::TypeDefine::char()))
            }
            parse::AtomicExpr::StringLiteral(str) => Result::Ok(
                ast::AtomicExpr::String(str.parsed.clone()).with_ty(ast::TypeDefine::string()),
            ),
            parse::AtomicExpr::NumberLiteral(n) => match n {
                parse::NumberLiteral::Float { number, .. } => {
                    Result::Ok(ast::AtomicExpr::Float(*number).with_ty(ast::TypeDefine::float()))
                }
                parse::NumberLiteral::Digit { number, .. } => Result::Ok(
                    ast::AtomicExpr::Integer(*number).with_ty(ast::TypeDefine::integer()),
                ),
            },
            parse::AtomicExpr::FnCall(fn_call) => {
                parse::FnCall::to_ast(fn_call, _selection, _global)
            }
            parse::AtomicExpr::Variable(var) => {
                let Some(def) = _global.search_var(var) else {
                    return _selection.throw("use of undefined variable");
                };

                Result::Ok(ast::AtomicExpr::Variable(var.0.clone()).with_ty(def.0.ty.clone()))
            }

            // here, this is incorrect because operators may be overdriven
            // all operator overdriven must be casted into function calling here but primitives
            parse::AtomicExpr::UnaryExpr(unary) => {
                let l = _global.to_ast(&unary.expr)?;
                let define = _global.push_compute(ast::OperateExpr::unary(*unary.operator, l.val));
                Result::Ok(define.with_ty(l.ty))
            }
            parse::AtomicExpr::BracketExpr(expr) => _global.to_ast(&expr.expr),
            parse::AtomicExpr::Initialization(_) => todo!("how to do???"),
        }
    }
}
