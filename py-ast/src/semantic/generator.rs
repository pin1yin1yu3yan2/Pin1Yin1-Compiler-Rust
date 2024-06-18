use super::mangle::Mangle;
use super::*;
use crate::parse;
use either::Either;
use py_declare::mir::IntoIR;
use py_declare::*;
use py_lex::PU;
use terl::*;

py_ir::custom_ir_variable!(pub IR<py_ir::value::Value>);

pub trait Generator<Item: ?Sized> {
    type Forward;

    fn generate(&mut self, item: &Item) -> Self::Forward;
}

type Errors = Either<Error, Vec<Error>>;

type ItemsGenerateResult = Result<Vec<Item>, Either<Vec<Error>, Vec<Vec<Error>>>>;

struct FallibleResults<T, E> {
    inner: Result<Vec<T>, Vec<E>>,
}

impl<T, E> FallibleResults<T, E> {
    fn new() -> Self {
        Self { inner: Ok(vec![]) }
    }

    fn add_ok(&mut self, item: T) -> Result<(), T> {
        match &mut self.inner {
            Ok(state) => {
                state.push(item);
                Ok(())
            }
            _ => Err(item),
        }
    }

    fn add_err(&mut self, err: E) {
        match &mut self.inner {
            Ok(_) => self.inner = Err(vec![err]),
            Err(errs) => errs.push(err),
        }
    }

    fn add_result(&mut self, result: Result<T, E>) {
        match result {
            Ok(ok) => _ = self.add_ok(ok),
            Err(err) => self.add_err(err),
        }
    }

    fn take(self) -> Result<Vec<T>, Vec<E>> {
        self.inner
    }
}

fn fn_define_task<'d, M: Mangle>(
    define: &mut Defines<M>,
    fn_define: &'d parse::FnDefine,
) -> Result<impl FnOnce(&'d Defines<M>) -> Result<FnDefine, Vec<Error>>, Error> {
    let ty = fn_define.ty.to_mir_ty()?;

    let params = fn_define
        .params
        .iter()
        .try_fold(Vec::new(), |mut vec, pu| {
            let name = pu.name.to_string();
            let ty = pu.ty.to_mir_ty()?;
            vec.push(defs::Parameter { name, ty });
            Result::Ok(vec)
        })?;

    let fn_sign = defs::FnSign::new(
        ty.clone(),
        params.clone(),
        fn_define.retty_span,
        fn_define.sign_span,
    );

    let mangled_name = define.regist_fn(fn_define, fn_sign)?;

    Ok(|define: &Defines<M>| -> Result<FnDefine, Vec<Error>> {
        let mut statement_transmuter = {
            let scopes = BasicScopes::default();
            let spans = fn_define.params.iter().map(WithSpan::get_span);
            let fn_scope = FnScope::new(&mangled_name, params.iter(), spans);
            StatementGenerator::new(&define.defs, fn_scope, scopes)
        };

        let body = match statement_transmuter.generate(&fn_define.codes) {
            Err(error) => Err(vec![error]),
            Ok(body) if !body.returned => {
                let reason = format!("function `{}` is never return", fn_define.name);
                let error = fn_define.sign_span.make_error(reason);
                Err(vec![error])
            }

            Ok(body) => Ok(body),
        }?;

        statement_transmuter.fn_scope.declare_map.declare_all()?;

        let export = fn_define.export.is_some();
        let mir_fn = mir::FnDefine {
            export,
            ty,
            body,
            params,
            name: mangled_name,
        };
        Ok(mir_fn.into_ir(&statement_transmuter.fn_scope.declare_map))
    })
}

#[cfg(feature = "parallel")]
mod parallel {
    use super::*;

    impl<M: Mangle> Generator<[parse::Item]> for Defines<M> {
        type Forward = ItemsGenerateResult;

        fn generate(&mut self, items: &[parse::Item]) -> Self::Forward {
            let mut tasks = FallibleResults::new();
            for item in items {
                let next = match item {
                    parse::Item::FnDefine(fn_define) => fn_define,
                    parse::Item::Comment(_) => continue,
                };
                match fn_define_task(self, next).map(Box::new) {
                    Ok(task) => {
                        let _ = tasks.add_ok(task);
                    }
                    Err(err) => {
                        tasks.add_err(err);
                    }
                };
            }

            use rayon::prelude::*;

            let (result_s, result_r) = std::sync::mpsc::channel();

            let state = std::thread::spawn(move || {
                let mut items = FallibleResults::new();

                while let Ok(result) = result_r.recv() {
                    items.add_result(result);
                }
                items.take()
            });

            tasks
                .take()
                .map_err(Either::Left)?
                .into_par_iter()
                .map(|task| (task, result_s.clone()))
                .for_each(|(task, result_s)| {
                    _ = result_s.send(task(self).map(Into::into));
                });
            // drop them, or collection thread will never return
            drop(result_s);

            state.join().unwrap().map_err(Either::Right)
        }
    }
}

#[cfg(not(feature = "parallel"))]
mod normal {
    use super::*;
    impl<M: Mangle> Generator<[parse::Item]> for Defines<M> {
        type Forward = ItemsGenerateResult;

        fn generate(&mut self, items: &[parse::Item]) -> Self::Forward {
            let tasks = items
                .iter()
                .fold(FallibleResults::new(), |mut result, item| {
                    match item {
                        parse::Item::FnDefine(fn_define) => {
                            result.add_result(fn_define_task(self, fn_define))
                        }
                        parse::Item::Comment(_) => {}
                    };
                    result
                })
                .take()
                .map_err(Either::Left)?;

            let mut state = FallibleResults::new();
            for task in tasks {
                match task(self) {
                    Ok(fn_define) => {
                        _ = state.add_ok(py_ir::Item::from(fn_define));
                    }
                    Err(err) => {
                        state.add_err(err);
                    }
                };
            }

            state.take().map_err(Either::Right)
        }
    }
}

impl<M: Mangle> Generator<parse::Item> for Defines<M> {
    type Forward = Result<Option<Item>, Errors>;

    fn generate(&mut self, item: &parse::Item) -> Self::Forward {
        match item {
            parse::Item::FnDefine(fn_define) => self.generate(fn_define).map(Into::into).map(Some),
            parse::Item::Comment(..) => Ok(None),
        }
    }
}

impl<M: Mangle> Generator<parse::FnDefine> for Defines<M> {
    type Forward = Result<FnDefine, Errors>;

    fn generate(&mut self, fn_define: &parse::FnDefine) -> Self::Forward {
        fn_define_task(self, fn_define).map_err(Either::Left)?(self).map_err(Either::Right)
    }
}

struct StatementGenerator<'w> {
    pub defs: &'w Defs,
    pub fn_scope: FnScope,
    pub scopes: BasicScopes,
    stmts: mir::Statements,
}

struct VarDeineLoc(usize);

impl<'w> StatementGenerator<'w> {
    fn new(defs: &Defs, fn_scope: FnScope, scopes: BasicScopes) -> StatementGenerator<'_> {
        StatementGenerator {
            defs,
            fn_scope,
            scopes,
            stmts: Default::default(),
        }
    }

    #[inline]
    fn push_stmt(&mut self, stmt: impl Into<mir::Statement>) {
        self.stmts.push(stmt.into());
    }

    fn temp_var_define<I>(
        &mut self,
        param_ty: GroupIdx,
        result_ty: GroupIdx,
        init: I,
    ) -> ValueHandle
    where
        I: Into<mir::AssignValue>,
    {
        let init = mir::Undeclared::new(init.into(), param_ty);
        let temp_name = self.fn_scope.temp_name();
        let loc = self.push_var_define(mir::VarDefine {
            ty: param_ty,
            name: temp_name.clone(),
            init: Some(init),
            is_temp: true,
        });
        let handle = mir::Undeclared::new(mir::Value::Variable(temp_name), result_ty);
        ValueHandle::new(loc, handle)
    }

    fn push_var_define(&mut self, var_define: mir::VarDefine) -> VarDeineLoc {
        let loc = VarDeineLoc(self.stmts.len());
        self.stmts.push(mir::Statement::VarDefine(var_define));
        loc
    }

    fn rename_var_define(&mut self, loc: VarDeineLoc, new_name: &str) {
        match &mut self.stmts[loc.0] {
            py_ir::Statement::VarDefine(define) => {
                define.name = new_name.to_owned();
                define.is_temp = false;
            }
            _ => unreachable!(),
        }
    }

    fn take_stmts(&mut self) -> mir::Statements {
        std::mem::take(&mut self.stmts)
    }

    fn replace_stmts(&mut self, new: mir::Statements) -> mir::Statements {
        std::mem::replace(&mut self.stmts, new)
    }

    fn search_value(&mut self, name: &str) -> Option<defs::VarDef> {
        self.fn_scope
            .search_parameter(name)
            .or_else(|| self.scopes.search_variable(name))
    }

    fn in_new_basic_scope<R>(&mut self, active: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes.push(Default::default());
        let r = active(self);
        self.scopes.pop();
        r
    }
}

impl Generator<parse::Statement> for StatementGenerator<'_> {
    type Forward = Result<Option<mir::Statement>>;

    fn generate(&mut self, stmt: &parse::Statement) -> Self::Forward {
        match stmt {
            parse::Statement::VarStoreStmt(stmt) => self.generate(&****stmt).map(Into::into),
            parse::Statement::If(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::While(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::Return(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::CodeBlock(stmt) => self.generate(&**stmt).map(Into::into),
            parse::Statement::VarDefineStmt(stmt) => match self.generate(&****stmt)? {
                Some(var_define) => Ok(var_define.into()),
                None => return Ok(None),
            },
            parse::Statement::FnCallStmt(stmt) => {
                self.generate(&****stmt)?;
                return Ok(None);
            }
            parse::Statement::Comment(..) => return Ok(None),
        }
        .map(Some)
    }
}

struct ValueHandle {
    loc: Option<VarDeineLoc>,
    handle: mir::Undeclared<mir::Value>,
}

impl std::ops::Deref for ValueHandle {
    type Target = mir::Undeclared<mir::Value>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl ValueHandle {
    fn new(loc: VarDeineLoc, handle: mir::Undeclared<mir::Value>) -> Self {
        Self {
            loc: Some(loc),
            handle,
        }
    }
}

impl From<mir::Undeclared<mir::Value>> for ValueHandle {
    fn from(handle: mir::Undeclared<mir::Value>) -> Self {
        Self { loc: None, handle }
    }
}

impl Generator<parse::FnCall> for StatementGenerator<'_> {
    type Forward = Result<ValueHandle>;

    fn generate(&mut self, fn_call: &parse::FnCall) -> Self::Forward {
        let args = fn_call.args.iter().try_fold(vec![], |mut args, expr| {
            args.push(self.generate(expr)?.handle);
            Result::Ok(args)
        })?;

        let Some(overloads) = self.defs.get_unmangled(&fn_call.fn_name) else {
            return Err(fn_call.make_error(format!("call undefinded function {}", fn_call.fn_name)));
        };

        let args_spans = fn_call
            .args
            .iter()
            .map(|pu| pu.get_span())
            .collect::<Vec<_>>();

        let overload_len_filter =
            filters::FnParamLen::new(Some(&fn_call.fn_name), args.len(), fn_call.get_span());

        let branch_builder = |overload: &Overload| {
            let mut branch_builder = BranchesBuilder::new(Type::Overload(overload.clone()));
            if branch_builder.filter_self(self.defs, &overload_len_filter) {
                // length of overload.params are equal to arg's
                for ((param, arg), span) in overload.params.iter().zip(&args).zip(&args_spans) {
                    let filter = filters::TypeEqual::new(&param.ty, *span);
                    let declare_map = &mut self.fn_scope.declare_map;
                    branch_builder = branch_builder.new_depend::<Directly, _>(
                        declare_map,
                        self.defs,
                        arg.ty,
                        &filter,
                    );
                }
            }
            branch_builder
        };

        let branch_builders = overloads.iter().map(branch_builder).collect();
        let overload = self
            .fn_scope
            .declare_map
            .build_group(GroupBuilder::new(fn_call.get_span(), branch_builders));

        Ok(self.temp_var_define(overload, overload, mir::FnCall { args }))
    }
}

impl Generator<parse::VarStore> for StatementGenerator<'_> {
    type Forward = Result<mir::VarStore>;

    fn generate(&mut self, var_store: &parse::VarStore) -> Self::Forward {
        let name = var_store.name.to_string();
        let val = self.generate(&var_store.assign.val)?.handle;

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

impl Generator<parse::VarDefine> for StatementGenerator<'_> {
    type Forward = Result<Option<mir::VarDefine>>;

    fn generate(&mut self, var_define: &parse::VarDefine) -> Self::Forward {
        let ty = var_define.ty.to_mir_ty()?;
        let ty = self
            .fn_scope
            .declare_map
            .new_static_group(var_define.ty.get_span(), std::iter::once(ty.into()));
        self.scopes
            .regist_variable(&var_define.name, defs::VarDef { ty, mutable: true });

        let init = match &var_define.init {
            Some(var_assign) => {
                let init = self.generate(&var_assign.val)?;

                if let Some(loc) = init.loc {
                    self.rename_var_define(loc, &var_define.name);
                    return Ok(None);
                }
                let at = var_assign.val.get_span();
                self.fn_scope.declare_map.merge_group(at, ty, init.ty);

                Some(mir::Undeclared::new(init.handle.val.into(), init.handle.ty))
            }
            None => None,
        };

        let name = var_define.name.to_string();
        Ok(Some(mir::VarDefine {
            ty,
            name,
            init,
            is_temp: false,
        }))
    }
}

impl Generator<parse::If> for StatementGenerator<'_> {
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

impl Generator<parse::While> for StatementGenerator<'_> {
    type Forward = Result<mir::While>;

    fn generate(&mut self, while_: &parse::While) -> Self::Forward {
        let cond = self.generate(&while_.conds)?;
        let body = self.generate(&while_.block)?;
        Ok(mir::While { cond, body })
    }
}

impl Generator<parse::IfBranch> for StatementGenerator<'_> {
    type Forward = Result<mir::IfBranch>;

    fn generate(&mut self, branch: &parse::IfBranch) -> Self::Forward {
        let cond = self.generate(&branch.conds)?;
        let body = self.generate(&branch.body)?;
        Ok(mir::IfBranch { cond, body })
    }
}

impl Generator<parse::Return> for StatementGenerator<'_> {
    type Forward = Result<mir::Return>;

    fn generate(&mut self, ret: &parse::Return) -> Self::Forward {
        let val = match &ret.val {
            Some(expr) => {
                let val = self.generate(expr)?;
                let mangled_fn = self.defs.get_mangled(&self.fn_scope.fn_name);
                self.fn_scope
                    .declare_map
                    .declare_type(expr.get_span(), val.ty, &mangled_fn.ty);
                Some(val.handle)
            }
            None => None,
        };
        Ok(mir::Return { val })
    }
}

impl Generator<parse::CodeBlock> for StatementGenerator<'_> {
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

impl Generator<parse::Conditions> for StatementGenerator<'_> {
    type Forward = Result<mir::Condition>;

    fn generate(&mut self, conds: &parse::Conditions) -> Self::Forward {
        let (compute, val) = self.in_new_basic_scope(|g| {
            let mut last_condition = g.generate(&conds[0])?;
            for arg in conds.iter().skip(1) {
                last_condition = g.generate(arg)?;
            }
            Ok((g.take_stmts(), last_condition.handle))
        })?;

        // type check
        let bool = py_ir::types::PrimitiveType::Bool.into();
        let last_cond_span = conds.last().unwrap().get_span();
        self.fn_scope
            .declare_map
            .declare_type(last_cond_span, val.ty, &bool);
        Ok(mir::Condition { val, compute })
    }
}

impl Generator<parse::Expr> for StatementGenerator<'_> {
    type Forward = Result<ValueHandle>;

    fn generate(&mut self, expr: &parse::Expr) -> Self::Forward {
        let mut vals = Vec::new();
        for item in expr.iter() {
            match item {
                parse::ExprItem::AtomicExpr(atomic) => vals.push(self.generate(atomic)?),
                parse::ExprItem::Operators(op) => match op.associativity() {
                    py_lex::ops::OperatorAssociativity::Binary => {
                        let r = vals.pop().unwrap();
                        let l = vals.pop().unwrap();
                        self.fn_scope
                            .declare_map
                            .merge_group(expr.get_span(), l.ty, r.ty);
                        use py_ir::types::PrimitiveType;
                        use py_lex::ops::OperatorTypes::CompareOperator;

                        // for compare operators(like == != < >), the result will be a boolean value,
                        // not parameters' type
                        let param_ty = l.ty;
                        let result_ty = if op.op_ty() == CompareOperator {
                            self.fn_scope
                                .declare_map
                                .new_static_group(expr.get_span(), [PrimitiveType::Bool.into()])
                        } else {
                            l.ty
                        };

                        let init = mir::Operate::Binary(**op, l.handle, r.handle);
                        vals.push(self.temp_var_define(param_ty, result_ty, init));
                    }
                    py_lex::ops::OperatorAssociativity::Unary => {
                        let v = vals.pop().unwrap();
                        let ty = v.ty;

                        let init = mir::Operate::Unary(**op, v.handle);
                        vals.push(self.temp_var_define(ty, ty, init));
                    }

                    py_lex::ops::OperatorAssociativity::None => unreachable!(),
                },
            }
        }
        vals.pop().ok_or_else(|| unreachable!())
    }
}

impl Generator<PU<parse::AtomicExpr>> for StatementGenerator<'_> {
    type Forward = Result<ValueHandle>;

    fn generate(&mut self, atomic: &PU<parse::AtomicExpr>) -> Self::Forward {
        let literal = match &**atomic {
            // atomics
            // 解析
            parse::AtomicExpr::CharLiteral(char) => py_ir::value::Literal::Char(char.parsed),
            parse::AtomicExpr::NumberLiteral(n) => match n {
                parse::NumberLiteral::Float(number) => py_ir::value::Literal::Float(*number),
                parse::NumberLiteral::Digit(number) => py_ir::value::Literal::Integer(*number),
            },

            parse::AtomicExpr::StringLiteral(_str) => {
                // TODO: init for array
                todo!("a VarDefine statement will be generate...")
            }
            parse::AtomicExpr::FnCall(fn_call) => return self.generate(fn_call),
            parse::AtomicExpr::Variable(name) => {
                let Some(def) = self.search_value(name) else {
                    return Err(atomic.make_error("use of undefined variable"));
                };

                let val = mir::Value::Variable(name.to_string());
                return Ok(mir::Undeclared::new(val, def.ty).into());
            }
            parse::AtomicExpr::Array(ref _array) => {
                // elements in arrray must be same type
                todo!()
            }
        };

        let ty = self.fn_scope.declare_map.build_group({
            let branches = mir::Undeclared::literal_branches(&literal);
            GroupBuilder::new(atomic.get_span(), branches)
        });
        Ok(mir::Undeclared::new(literal.into(), ty).into())
    }
}
