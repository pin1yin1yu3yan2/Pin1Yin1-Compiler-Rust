use super::mangle::Mangler;
use super::*;
use crate::parse;
use py_declare::*;
use terl::*;

pub trait Ast<M: Mangler>: ParseUnit {
    type Forward;

    /// divided [`PU`] into [`ParseUnit::Target`] and [`Span`] becase
    /// variants from [`crate::complex_pu`] isnot [`PU`], and the [`Span`]
    /// was stored in the enum
    fn to_ast(s: &Self::Target, span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward>;
}

impl<M: Mangler> Ast<M> for parse::Item {
    type Forward = ();

    fn to_ast(stmt: &Self::Target, span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        match stmt {
            parse::Item::FnDefine(fn_define) => {
                scope.to_ast_inner::<parse::FnDefine>(fn_define, span)?;
            }
            parse::Item::Comment(_) => {}
        }
        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::FnDefine {
    type Forward = ();

    fn to_ast(
        fn_define: &Self::Target,
        span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let name = fn_define.name.to_string();

        let ty: mir::TypeDefine = fn_define.ty.to_ast_ty()?;

        let params = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                let ty = pu.ty.to_ast_ty()?;
                let name = pu.name.to_string();
                let param = defs::Param { name, ty };
                vec.push(param);
                Result::Ok(vec)
            })?;

        let fn_sign = defs::FnSign {
            loc: span,
            ty: ty.clone(),
            params,
        };

        // generate ast
        scope.create_fn(name, fn_sign, &fn_define.params, |scope| {
            // scope.regist_params(params_iter);
            for stmt in &fn_define.codes.stmts {
                scope.to_ast(stmt)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::Statement {
    type Forward = ();

    fn to_ast(stmt: &Self::Target, span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        match stmt {
            parse::Statement::FnCallStmt(fn_call) => {
                let fn_call = scope.to_ast(fn_call)?;
                // so that a meanless FnDefine Statement will be generate,
                // avoiding implement IntoIR for FnCall
                scope.fn_call_stmt(fn_call);
            }
            parse::Statement::VarStoreStmt(var_store) => {
                scope.to_ast(var_store)?;
            }
            parse::Statement::VarDefineStmt(var_define) => {
                scope.to_ast(var_define)?;
            }
            parse::Statement::If(if_) => {
                scope.to_ast_inner::<parse::If>(if_, span)?;
            }
            parse::Statement::While(while_) => {
                scope.to_ast_inner::<parse::While>(while_, span)?;
            }
            parse::Statement::Return(return_) => {
                scope.to_ast_inner::<parse::Return>(return_, span)?;
            }
            parse::Statement::CodeBlock(block) => {
                scope.to_ast_inner::<parse::CodeBlock>(block, span)?;
            }
            parse::Statement::Comment(_) => {}
        }
        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::FnCall {
    type Forward = mir::Variable;

    fn to_ast(
        fn_call: &Self::Target,
        span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let args = fn_call
            .args
            .args
            .iter()
            .try_fold(vec![], |mut args, expr| {
                args.push(scope.to_ast(expr)?);
                Result::Ok(args)
            })?;

        let overload = scope.new_declare_group(|map, defs| {
            // TODO: call non-exist function error
            let overloads = defs.get_unmangled(&fn_call.fn_name).unwrap();
            let pre_filter = filters::FnParamLen::new(Some(&fn_call.fn_name), args.len(), span);
            let bench_builders = overloads
                .iter()
                .map(|overload| {
                    let mut bench_builder = BenchBuilder::new(Type::Overload(overload.clone()));
                    bench_builder.filter_self(defs, &pre_filter);

                    if bench_builder.is_ok() {
                        for (param, arg) in overload.params.iter().zip(args.iter()) {
                            let filter = filters::TypeEqual::new(&param.ty, span);
                            bench_builder =
                                bench_builder.new_depend::<Directly, _>(map, defs, arg.ty, &filter);
                        }
                    }

                    bench_builder
                })
                .collect();

            GroupBuilder::new(span, bench_builders)
        });

        let val = mir::AtomicExpr::FnCall(mir::FnCall { args });
        let fn_call = mir::Variable::new(val, overload);

        Ok(fn_call)
    }
}

impl<M: Mangler> Ast<M> for parse::VarStore {
    type Forward = ();

    fn to_ast(
        var_store: &Self::Target,
        span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let name = var_store.name.to_string();
        // function parameters are inmutable

        let val = scope.to_ast(&var_store.assign.val)?;
        let Some(var_def) = scope.search_var(&name) else {
            return Err(span.make_error(format!("use of undefined variable {}", name)));
        };
        if !var_def.mutable {
            return Err(span.make_error(format!("cant assign to a immmutable variable {}", name)));
        }

        scope.merge_group(span, var_def.ty, val.ty);
        scope.push_stmt(mir::VarStore { name, val });

        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::VarDefine {
    type Forward = ();

    fn to_ast(
        var_define: &Self::Target,
        span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        // TODO: testfor if  ty exist
        let ty = var_define.ty.to_ast_ty()?;
        let name = var_define.name.to_string();

        let init = match &var_define.init {
            Some(init) => Some(scope.to_ast(&init.val)?),
            None => None,
        };

        let def = defs::VarDef {
            ty: scope.new_static_group(span, std::iter::once(ty.clone().into())),

            mutable: true,
        };
        let stmt = mir::VarDefine { ty, name, init };
        scope.regist_var(stmt, def, span);

        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::If {
    type Forward = ();

    fn to_ast(if_: &Self::Target, _span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        let mut branches = vec![scope.to_ast(&if_.ruo4)?];
        for chain in &if_.chains {
            match &**chain {
                parse::ChainIf::AtomicElseIf(atomic) => {
                    branches.push(scope.to_ast(&atomic.ruo4)?);
                }
                parse::ChainIf::AtomicElse(else_) => {
                    let else_ = scope.to_ast(&else_.block)?;
                    scope.push_stmt(mir::Statement::If(mir::If {
                        branches,
                        else_: Some(else_),
                    }));
                    return Ok(());
                }
            }
        }
        scope.push_stmt(mir::Statement::If(mir::If {
            branches,
            else_: None,
        }));
        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::While {
    type Forward = ();

    fn to_ast(
        while_: &Self::Target,
        _span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let cond = scope.to_ast(&while_.conds)?;
        let body = scope.to_ast(&while_.block)?;
        scope.push_stmt(mir::Statement::While(mir::While { cond, body }));
        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::AtomicIf {
    type Forward = mir::IfBranch;

    fn to_ast(
        atomic: &Self::Target,
        _span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let cond = scope.to_ast(&atomic.conds)?;
        let body = scope.to_ast(&atomic.block)?;
        Result::Ok(mir::IfBranch { cond, body })
    }
}

impl<M: Mangler> Ast<M> for parse::Return {
    type Forward = ();

    fn to_ast(
        return_: &Self::Target,
        _span: Span,
        scope: &mut ModScope<M>,
    ) -> Result<Self::Forward> {
        let val = match &return_.val {
            Some(val) => Some(scope.to_ast(val)?),
            None => None,
        };
        scope.push_stmt(mir::Statement::Return(mir::Return { val }));
        Ok(())
    }
}

impl<M: Mangler> Ast<M> for parse::CodeBlock {
    type Forward = mir::Statements;

    fn to_ast(block: &Self::Target, _span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        scope
            .spoce(|scope| {
                for stmt in &block.stmts {
                    scope.to_ast(stmt)?;
                }
                Ok(())
            })
            .map(|(v, _)| v)
    }
}

impl<M: Mangler> Ast<M> for parse::Arguments {
    type Forward = mir::Condition;

    fn to_ast(cond: &Self::Target, _span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        let (compute, last_cond) = scope.spoce(|scope| {
            let mut last_cond = scope.to_ast(&cond.args[0])?;
            for arg in cond.args.iter().skip(1) {
                last_cond = scope.to_ast(arg)?;
            }
            Result::Ok(last_cond)
        })?;

        let span = cond.args.last().unwrap().get_span();
        scope.assert_type_is(span, last_cond.ty, &mir::PrimitiveType::Bool.into());

        Result::Ok(mir::Condition {
            val: last_cond,
            compute,
        })
    }
}

impl<M: Mangler> Ast<M> for parse::Expr {
    type Forward = mir::Variable;

    fn to_ast(expr: &Self::Target, span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        match expr {
            parse::Expr::Atomic(atomic) => parse::AtomicExpr::to_ast(atomic, span, scope),
            parse::Expr::Binary(l, o, r) => {
                let l = scope.to_ast(l)?;
                let r = scope.to_ast(r)?;

                // TODO: operator overload(type system)
                // TODO: primitive operators
                // TODO: operator -> function call

                // now, we suppert primitive operators only, so they should be same type
                scope.merge_group(span, l.ty, r.ty);

                let op = mir::OperateExpr::binary(o.take(), l, r);
                Result::Ok(scope.push_compute(op))
            }
        }
    }
}

impl<M: Mangler> Ast<M> for parse::AtomicExpr {
    type Forward = mir::Variable;

    fn to_ast(atomic: &Self::Target, span: Span, scope: &mut ModScope<M>) -> Result<Self::Forward> {
        let atomic = match atomic {
            // atomics
            parse::AtomicExpr::CharLiteral(char) => mir::AtomicExpr::Char(char.parsed),
            parse::AtomicExpr::StringLiteral(str) => mir::AtomicExpr::String(str.parsed.clone()),
            parse::AtomicExpr::NumberLiteral(n) => match n {
                parse::NumberLiteral::Float { number, .. } => mir::AtomicExpr::Float(*number),
                parse::NumberLiteral::Digit { number, .. } => mir::AtomicExpr::Integer(*number),
            },

            parse::AtomicExpr::FnCall(fn_call) => {
                return parse::FnCall::to_ast(fn_call, span, scope)
            }
            parse::AtomicExpr::Variable(var) => {
                let Some(def) = scope.search_var(var) else {
                    return Err(span.make_error("use of undefined variable"));
                };

                let variable = mir::Variable {
                    val: mir::AtomicExpr::Variable(var.to_string()),
                    ty: def.ty,
                };
                return Ok(variable);
            }

            // here, this is incorrect because operators may be overloadn
            // all operator overloadn must be casted into function calling here but primitives
            parse::AtomicExpr::UnaryExpr(unary) => {
                let l = scope.to_ast(&unary.expr)?;
                let expr = scope.push_compute(mir::OperateExpr::unary(*unary.operator, l));
                return Ok(expr);
            }
            parse::AtomicExpr::BracketExpr(expr) => return scope.to_ast(&expr.expr),
            parse::AtomicExpr::Initialization(_) => todo!("how to do???"),
        };

        let ty = scope.new_declare_group(|_, _| {
            let benches = mir::Variable::literal_benches(&atomic);
            GroupBuilder::new(span, benches)
        });
        Ok(mir::Variable::new(atomic, ty))
    }
}
