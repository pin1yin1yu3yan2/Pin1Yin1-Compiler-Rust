use super::mangle::Mangle;
use super::*;
use crate::parse;
use either::Either;
use py_declare::mir::IntoIR;
use py_declare::*;
use py_ir as ir;
use py_lex::PU;
use terl::*;

pub trait Generator<Item> {
    type Forward;

    fn generate(&mut self, item: &Item) -> Self::Forward;
}

impl<M: Mangle> Generator<parse::Item> for Defines<M> {
    type Forward = Result<Option<ir::Item<ir::Variable>>, Either<Error, Vec<Error>>>;

    fn generate(&mut self, item: &parse::Item) -> Self::Forward {
        match item {
            parse::Item::FnDefine(fn_define) => self.generate(fn_define).map(Into::into).map(Some),
            parse::Item::Comment(..) => Ok(None),
        }
    }
}

impl<M: Mangle> Generator<parse::FnDefine> for Defines<M> {
    type Forward = Result<ir::FnDefine<ir::Variable>, Either<Error, Vec<Error>>>;

    fn generate(&mut self, fn_define: &parse::FnDefine) -> Self::Forward {
        let name = fn_define.name.to_string();

        let ty = fn_define.ty.to_mir_ty().map_err(Either::Left)?;

        let params = fn_define
            .params
            .params
            .iter()
            .try_fold(Vec::new(), |mut vec, pu| {
                let ty = pu.ty.to_mir_ty()?;
                let name = pu.name.to_string();
                let param = defs::Parameter { name, ty };
                vec.push(param);
                Result::Ok(vec)
            })
            .map_err(Either::Left)?;

        let fn_sign = defs::FnSign {
            retty_span: fn_define.retty_span,
            sign_span: fn_define.sign_span,
            ty: ty.clone(),
            params: params.clone(),
        };

        let mangled_fn_name = self.mangler.mangle_fn(&fn_define.name, &fn_sign);
        if let Some(previous) = self.defs.try_get_mangled(&mangled_fn_name) {
            let previous_define = previous
                .sign_span
                .make_message(format!("funcion {} has been definded here", name));
            let mut err = fn_sign
                .sign_span
                .make_error(format!("double define for function {}", name))
                .append(previous_define);
            if previous.ty == fn_sign.ty {
                err += format!("note: if you want to overload funcion {}, you can define them with different parameters",name)
            } else {
                err += "note: overload which only return type is differnet is not allowed";
                err += format!("note: if you want to overload funcion {}, you can define them with different parameters",name);
            }
            return Err(Either::Left(err));
        }

        let fn_scope = FnScope::new(
            &mangled_fn_name,
            fn_define
                .params
                .params
                .iter()
                .zip(fn_sign.params.iter())
                .map(|(raw, param)| (raw.get_span(), param)),
        );
        let scopes = BasicScopes::default();

        self.defs
            .new_fn(name.clone(), mangled_fn_name.clone(), fn_sign);
        let mut statement_transmuter = StatementTransmuter::new(&mut self.defs, fn_scope, scopes);

        let body = statement_transmuter
            .generate(&fn_define.codes)
            .map_err(Either::Left)?;
        if !body.returned {
            return Err(Either::Left(
                fn_define
                    .sign_span
                    .make_error(format!("function `{}` is never return", name)),
            ));
        }

        statement_transmuter
            .fn_scope
            .declare_map
            .declare_all()
            .map_err(Either::Right)?;

        let mir_fn = mir::FnDefine {
            ty,
            body,
            params,
            name: mangled_fn_name,
        };
        Ok(mir_fn.into_ir(&statement_transmuter.fn_scope.declare_map))
    }
}

struct StatementTransmuter<'w> {
    pub defs: &'w mut Defs,
    pub fn_scope: FnScope,
    pub scopes: BasicScopes,
    stmts: mir::Statements,
}

impl<'w> StatementTransmuter<'w> {
    pub fn new(defs: &mut Defs, fn_scope: FnScope, scopes: BasicScopes) -> StatementTransmuter<'_> {
        StatementTransmuter {
            defs,
            fn_scope,
            scopes,
            stmts: Default::default(),
        }
    }

    #[inline]
    pub fn push_stmt(&mut self, stmt: impl Into<mir::Statement>) {
        self.stmts.push(stmt.into());
    }

    #[inline]
    pub fn take_stmts(&mut self) -> mir::Statements {
        std::mem::take(&mut self.stmts)
    }

    #[inline]
    pub fn replace_stmts(&mut self, new: mir::Statements) -> mir::Statements {
        std::mem::replace(&mut self.stmts, new)
    }

    #[inline]
    pub fn search_value(&mut self, name: &str) -> Option<defs::VarDef> {
        self.fn_scope
            .search_parameter(name)
            .or_else(|| self.scopes.search_variable(name))
    }

    #[inline]
    pub fn in_new_basic_scope<R>(&mut self, active: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes.push(Default::default());
        let r = active(self);
        self.scopes.pop();
        r
    }
}

impl Generator<parse::Statement> for StatementTransmuter<'_> {
    type Forward = Result<Option<mir::Statement>>;

    fn generate(&mut self, stmt: &parse::Statement) -> Self::Forward {
        match stmt {
            parse::Statement::FnCallStmt(stmt) => {
                let result = self.generate(&****stmt)?;
                let temp_name = self.fn_scope.temp_name();
                let var_define = mir::VarDefine {
                    ty: result.ty,
                    name: temp_name,
                    init: Some(result),
                };
                Ok(var_define.into())
            }
            parse::Statement::VarStoreStmt(stmt) => self.generate(&****stmt).map(Into::into),
            parse::Statement::VarDefineStmt(stmt) => self.generate(&****stmt).map(Into::into),
            parse::Statement::If(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::While(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::Return(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::CodeBlock(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::Comment(..) => return Ok(None),
        }
        .map(Some)
    }
}

impl Generator<parse::FnCall> for StatementTransmuter<'_> {
    type Forward = Result<mir::Variable>;

    fn generate(&mut self, fn_call: &parse::FnCall) -> Self::Forward {
        let args = fn_call.args.iter().try_fold(vec![], |mut args, expr| {
            args.push(self.generate(expr)?);
            Result::Ok(args)
        })?;

        let Some(overloads) = self.defs.get_unmangled(&fn_call.fn_name) else {
            return Err(fn_call.make_error(format!("call undefinded function {}", fn_call.fn_name)));
        };

        let overload_len_filter =
            filters::FnParamLen::new(Some(&fn_call.fn_name), args.len(), fn_call.get_span());

        let args_spans = fn_call
            .args
            .iter()
            .map(|pu| pu.get_span())
            .collect::<Vec<_>>();
        let bench_builders = overloads
            .iter()
            .map(|overload| {
                let mut bench_builder = BenchBuilder::new(Type::Overload(overload.clone()));
                bench_builder.filter_self(self.defs, &overload_len_filter);

                if bench_builder.is_ok() {
                    for ((param, arg), span) in overload.params.iter().zip(&args).zip(&args_spans) {
                        let filter = filters::TypeEqual::new(&param.ty, *span);
                        let declare_map = &mut self.fn_scope.declare_map;
                        bench_builder =
                            bench_builder.new_depend::<Directly, _>(declare_map, arg.ty, &filter);
                    }
                }
                bench_builder
            })
            .collect();
        let overload = self
            .fn_scope
            .declare_map
            .new_group(GroupBuilder::new(fn_call.get_span(), bench_builders));

        let val = mir::AtomicExpr::FnCall(mir::FnCall { args });
        let fn_call = mir::Variable::new(val, overload);

        Ok(fn_call)
    }
}

impl Generator<parse::VarStore> for StatementTransmuter<'_> {
    type Forward = Result<mir::VarStore>;

    fn generate(&mut self, var_store: &parse::VarStore) -> Self::Forward {
        let name = var_store.name.to_string();
        let val = self.generate(&var_store.assign.val)?;

        let val_at = var_store.assign.val.get_span();

        let Some(var_def) = self.search_value(&name) else {
            return Err(val_at.make_error(format!("use of undefined variable {}", name)));
        };
        if !var_def.mutable {
            return Err(val_at.make_error(format!("cant assign to a immmutable variable {}", name)));
        }

        self.fn_scope
            .declare_map
            .merge_group(val_at, var_def.ty, val.ty);

        Ok(mir::VarStore { name, val })
    }
}

impl Generator<parse::VarDefine> for StatementTransmuter<'_> {
    type Forward = Result<mir::VarDefine>;

    fn generate(&mut self, var_define: &parse::VarDefine) -> Self::Forward {
        let name = var_define.name.to_string();
        let ty = var_define.ty.to_mir_ty()?;
        let ty = self
            .fn_scope
            .declare_map
            .new_static_group(var_define.ty.get_span(), std::iter::once(ty.into()));
        let init = match &var_define.init {
            Some(var_assign) => {
                let init = self.generate(&var_assign.val)?;
                let at = var_assign.val.get_span();
                self.fn_scope.declare_map.merge_group(at, ty, init.ty);
                Some(init)
            }
            None => None,
        };
        self.scopes
            .regist_variable(&name, defs::VarDef { ty, mutable: true });

        Ok(mir::VarDefine { ty, name, init })
    }
}

impl Generator<parse::If> for StatementTransmuter<'_> {
    type Forward = Result<mir::If>;

    fn generate(&mut self, if_: &parse::If) -> Self::Forward {
        let branches = if_
            .branches
            .iter()
            .try_fold(vec![], |mut branches, branch| {
                branches.push(self.generate(branch)?);
                Ok(branches)
            })?;
        let else_ = match &if_.else_ {
            Some(else_) => Some(self.generate(&else_.block)?),
            None => None,
        };
        Ok(mir::If { branches, else_ })
    }
}

impl Generator<parse::While> for StatementTransmuter<'_> {
    type Forward = Result<mir::While>;

    fn generate(&mut self, while_: &parse::While) -> Self::Forward {
        let cond = self.generate(&while_.conds)?;
        let body = self.generate(&while_.block)?;
        Ok(mir::While { cond, body })
    }
}

impl Generator<parse::IfBranch> for StatementTransmuter<'_> {
    type Forward = Result<mir::IfBranch>;

    fn generate(&mut self, branch: &parse::IfBranch) -> Self::Forward {
        let cond = self.generate(&branch.conds)?;
        let body = self.generate(&branch.body)?;
        Ok(mir::IfBranch { cond, body })
    }
}

impl Generator<parse::Return> for StatementTransmuter<'_> {
    type Forward = Result<mir::Return>;

    fn generate(&mut self, ret: &parse::Return) -> Self::Forward {
        let val = match &ret.val {
            Some(expr) => {
                let val = self.generate(expr)?;
                let mangled_fn = self.defs.get_mangled(&self.fn_scope.fn_name);
                self.fn_scope
                    .declare_map
                    .declare_type(expr.get_span(), val.ty, &mangled_fn.ty);
                Some(val)
            }
            None => None,
        };
        Ok(mir::Return { val })
    }
}

impl Generator<parse::CodeBlock> for StatementTransmuter<'_> {
    type Forward = Result<mir::Statements>;

    fn generate(&mut self, item: &parse::CodeBlock) -> Self::Forward {
        self.in_new_basic_scope(|g| {
            let current_scope = g.take_stmts();
            for stmt in &item.stmts {
                if let Some(stmt) = g.generate(stmt)? {
                    g.push_stmt(stmt);
                }
            }
            Ok(g.replace_stmts(current_scope))
        })
    }
}

impl Generator<parse::Conditions> for StatementTransmuter<'_> {
    type Forward = Result<mir::Condition>;

    fn generate(&mut self, conds: &parse::Conditions) -> Self::Forward {
        let (compute, val) = self.in_new_basic_scope(|g| {
            let mut last_condition = g.generate(&conds[0])?;
            for arg in conds.iter().skip(1) {
                last_condition = g.generate(arg)?;
            }
            Ok((g.take_stmts(), last_condition))
        })?;

        // type check
        let bool = ir::PrimitiveType::Bool.into();
        let last_cond_span = conds.last().unwrap().get_span();
        self.fn_scope
            .declare_map
            .declare_type(last_cond_span, val.ty, &bool);
        Ok(mir::Condition { val, compute })
    }
}

impl Generator<parse::Expr> for StatementTransmuter<'_> {
    type Forward = Result<mir::Variable>;

    fn generate(&mut self, expr: &parse::Expr) -> Self::Forward {
        match expr {
            parse::Expr::Atomic(atomic) => self.generate(atomic),
            parse::Expr::Binary(l, o, r) => {
                let l = self.generate(&**l)?;
                let r = self.generate(&**r)?;

                // TODO: operator overload(type system)
                // TODO: primitive operators
                // TODO: operator -> function call

                // now, we suppert primitive operators only, so they should be same type
                self.fn_scope
                    .declare_map
                    .merge_group(expr.get_span(), l.ty, r.ty);
                use py_ir::PrimitiveType;
                use py_lex::ops::OperatorTypes::CompareOperator;

                // for compare operators(like == != < >), the result will be a boolean value,
                // not parameters' type
                let param_ty = l.ty;
                let result_ty = if o.op_ty() == CompareOperator {
                    self.fn_scope
                        .declare_map
                        .new_static_group(expr.get_span(), [PrimitiveType::Bool.into()])
                } else {
                    l.ty
                };

                let eval = mir::OperateExpr::binary(o.take(), l, r);
                let temp_name = self.fn_scope.temp_name();
                self.push_stmt(mir::Compute {
                    ty: param_ty,
                    name: temp_name.clone(),
                    eval,
                });
                Ok(mir::Variable {
                    val: mir::AtomicExpr::Variable(temp_name),
                    ty: result_ty,
                })
            }
        }
    }
}

impl Generator<PU<parse::AtomicExpr>> for StatementTransmuter<'_> {
    type Forward = Result<mir::Variable>;

    fn generate(&mut self, expr: &PU<parse::AtomicExpr>) -> Self::Forward {
        let literal = match &**expr {
            // atomics
            parse::AtomicExpr::CharLiteral(char) => ir::Literal::Char(char.parsed),
            parse::AtomicExpr::NumberLiteral(n) => match n {
                parse::NumberLiteral::Float { number, .. } => ir::Literal::Float(*number),
                parse::NumberLiteral::Digit { number, .. } => ir::Literal::Integer(*number),
            },

            parse::AtomicExpr::StringLiteral(_str) => {
                // TODO: init for array
                todo!("a VarDefine statement will be generate...")
            }
            parse::AtomicExpr::FnCall(fn_call) => return self.generate(fn_call),
            parse::AtomicExpr::Variable(name) => {
                let Some(def) = self.search_value(name) else {
                    return Err(expr.make_error("use of undefined variable"));
                };

                let variable =
                    mir::Variable::new(mir::AtomicExpr::Variable(name.to_string()), def.ty);
                return Ok(variable);
            }

            // here, this is incorrect because operators may be overload
            // all operator overload must be casted into function calling here but primitives
            parse::AtomicExpr::UnaryExpr(unary) => {
                let l = self.generate(&*unary.expr)?;
                let name = self.fn_scope.temp_name();

                let ty = l.ty;
                let compute = mir::Compute {
                    ty,
                    name: name.clone(),
                    eval: mir::OperateExpr::unary(*unary.operator, l),
                };
                self.push_stmt(compute);

                return Ok(mir::Variable {
                    ty,
                    val: mir::AtomicExpr::Variable(name),
                });
            }
            parse::AtomicExpr::BracketExpr(expr) => {
                let expr: &parse::Expr = &expr.expr;
                return self.generate(expr);
            }
            parse::AtomicExpr::Initialization(_) => todo!("how to do???"),
        };

        let ty = self.fn_scope.declare_map.new_group({
            let benches = mir::Variable::literal_benches(&literal);
            GroupBuilder::new(expr.get_span(), benches)
        });
        Ok(mir::Variable::new(literal.into(), ty))
    }
}
