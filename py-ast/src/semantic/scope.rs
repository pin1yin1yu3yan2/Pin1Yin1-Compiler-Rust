use super::mangle::*;
use super::*;
use crate::parse::*;
use py_declare::mir::IntoIR;
use py_declare::*;
use py_ir::ir;

use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;

use terl::*;

pub struct ModScope<M: Mangler = DefaultMangler> {
    mir_fns: Vec<FnScope>,
    ir_fns: Vec<CompiledFnScope>,

    prefix: Vec<ManglePrefix>,
    pub(crate) defs: Defs,
    _p: PhantomData<M>,
}

impl<M: Mangler> ModScope<M> {
    pub fn new_with_main() -> Self {
        Self {
            prefix: vec![],
            mir_fns: vec![FnScope::new("main", std::iter::empty())],
            ir_fns: vec![],
            defs: Defs::new_with_main(),
            _p: PhantomData,
        }
    }

    fn mangle_unit<'m>(&'m self, item: MangleItem<'m>) -> MangleUnit {
        MangleUnit {
            prefix: Cow::Borrowed(&self.prefix),
            item,
        }
    }

    pub fn mangle(&self, item: MangleItem) -> String {
        let unit = self.mangle_unit(item);
        M::mangle(unit)
    }

    pub fn mangle_ty(&self, ty: &mir::TypeDefine) -> MangleUnit {
        match ty {
            mir::TypeDefine::Primitive(pty) => self.mangle_unit(MangleItem::Type {
                ty: Cow::Owned(pty.to_string()),
            }),
            mir::TypeDefine::Complex(_) => todo!(),
        }
    }

    pub fn mangle_fn(&self, name: &str, sign: &defs::FnSign) -> String {
        let params = sign
            .params
            .iter()
            .map(|param| self.mangle_ty(&param.ty))
            .collect::<Vec<_>>();
        self.mangle(MangleItem::Fn {
            name: Cow::Borrowed(name),
            params,
        })
    }

    pub fn create_fn<F>(
        &mut self,
        name: String,
        sign: defs::FnSign,
        raw: &Parameters,
        fn_scope: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let mangled = self.mangle_fn(&name, &sign);

        let mir_fn = FnScope::new(
            mangled.clone(),
            raw.params
                .iter()
                .zip(sign.params.iter())
                .map(|(raw, param)| (raw.get_span(), param)),
        );

        self.mir_fns.push(mir_fn);
        self.defs.new_fn(name.to_owned(), mangled, sign);

        fn_scope(self)?;

        let scope = self.mir_fns.pop().unwrap();
        self.ir_fns.push(scope.into());

        Ok(())
    }

    pub fn regist_var(&mut self, stmt: mir::VarDefine, def: defs::VarDef, stmt_span: terl::Span) {
        if let Some(ref init) = stmt.init {
            let init_group = init.ty;
            self.assert_type_is(stmt_span, init_group, &stmt.ty);
        }

        self.current_fn_mut()
            .this_scope()
            .vars
            .insert(stmt.name.clone(), def);
        self.push_stmt(stmt);
    }

    pub fn assert_type_is(
        &mut self,
        stmt_span: terl::Span,
        val_ty: GroupIdx,
        expect_ty: &mir::TypeDefine,
    ) {
        self.mir_fns
            .last_mut()
            .unwrap()
            .declare_map
            .declare_type(stmt_span, val_ty, expect_ty)
    }

    pub fn search_var(&self, name: &str) -> Option<defs::VarDef> {
        let fn_scope = self.current_fn();

        if let Some(ty) = fn_scope.parameters.get(name) {
            return Some(defs::VarDef {
                ty: *ty,
                mutable: false,
            });
        }

        for scope in fn_scope.scope_stack.iter().rev() {
            if let Some(var_def) = scope.vars.get(name) {
                return Some(var_def.clone());
            }
        }

        None
    }

    fn current_fn(&self) -> &FnScope {
        self.mir_fns.last().unwrap()
    }

    fn current_fn_mut(&mut self) -> &mut FnScope {
        self.mir_fns.last_mut().unwrap()
    }

    // from FnScope
    pub fn spoce<T, F>(&mut self, f: F) -> Result<(mir::Statements, T)>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let scope = Default::default();

        self.current_fn_mut().scope_stack.push(scope);
        let t = f(self)?;
        let pool = self.current_fn_mut().scope_stack.pop().unwrap();

        Result::Ok((pool.stmts, t))
    }

    pub fn new_declare_group<B>(&mut self, builder: B) -> GroupIdx
    where
        B: FnOnce(&mut DeclareMap, &Defs) -> GroupBuilder,
    {
        let builder = builder(
            &mut self.mir_fns.last_mut().unwrap().declare_map,
            &self.defs,
        );
        self.current_fn_mut().declare_map.new_group(builder)
    }

    pub fn new_static_group<I>(&mut self, at: terl::Span, items: I) -> GroupIdx
    where
        I: IntoIterator<Item = Type>,
    {
        self.current_fn_mut()
            .declare_map
            .new_static_group(at, items)
    }

    pub fn merge_group(&mut self, stmt_span: terl::Span, to: GroupIdx, from: GroupIdx) {
        self.current_fn_mut()
            .declare_map
            .merge_group(stmt_span, to, from)
    }

    pub fn push_stmt(&mut self, stmt: impl Into<mir::Statement>) {
        self.current_fn_mut().this_scope().stmts.push(stmt.into());
    }

    pub fn push_compute(&mut self, eval: mir::OperateExpr) -> mir::Variable {
        let ty = match &eval {
            mir::OperateExpr::Unary(_, t) | mir::OperateExpr::Binary(_, _, t) => t.ty,
        };
        let name = self.current_fn_mut().alloc_name();

        self.push_stmt(mir::Compute {
            ty,
            name: name.clone(),
            eval,
        });
        mir::Variable {
            val: mir::AtomicExpr::Variable(name),
            ty,
        }
    }

    pub fn fn_call_stmt(&mut self, var: mir::Variable) {
        let temp = self.current_fn_mut().alloc_name();
        let called = var.ty;
        let mir::AtomicExpr::FnCall(fn_call) = var.val else {
            unreachable!()
        };

        self.push_stmt(mir::FnCallStmt {
            temp,
            called,
            args: fn_call,
        })
    }

    pub fn load_stmts(&mut self, stmts: &[PU<Statement>]) -> Result<()> {
        for stmt in stmts {
            self.to_ast(stmt)?;
        }
        Result::Ok(())
    }

    pub fn load_fns(&mut self, fn_defs: &[PU<FnDefine>]) -> Result<()> {
        for fn_def in fn_defs {
            self.to_ast(fn_def)?;
        }
        Result::Ok(())
    }

    pub fn to_ast_inner<A>(&mut self, s: &A::Target, span: Span) -> Result<A::Forward>
    where
        A: Ast<M>,
    {
        A::to_ast(s, span, self)
    }

    pub fn to_ast<A>(&mut self, pu: &PU<A>) -> Result<A::Forward>
    where
        A: Ast<M>,
    {
        self.to_ast_inner::<A>(&**pu, pu.get_span())
    }

    pub fn finish(self) -> Result<Vec<ir::FnDefine>, Vec<terl::Error>> {
        let mut errors = vec![];
        let mut defs = vec![];

        let other = self.mir_fns.into_iter().map(|fn_def| fn_def.into());
        for compiled in self.ir_fns.into_iter().chain(other) {
            if compiled.errors.is_empty() {
                let fn_def = self.defs.get_mangled(&compiled.fn_name);
                let params = fn_def
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| ir::Parameter {
                        ty: param.ty,
                        name: param.name,
                    })
                    .collect();
                let fn_def = ir::FnDefine {
                    ty: fn_def.ty.clone(),
                    name: compiled.fn_name,
                    params,
                    body: compiled.stmts,
                };
                defs.push(fn_def);
            } else {
                errors.extend(compiled.errors)
            }
        }
        if errors.is_empty() {
            Ok(defs)
        } else {
            Err(errors)
        }
    }
}

/// a scope that represents a fn's local scope
///
/// [`DeclareMap`] is used to picking overloads, declare var's types etc
///
/// un processed ast move into this struct and then become `mir`, mir misses
/// a part of type information, and fn_call is not point to monomorphic fn
///
/// these message will be filled by [`DeclareMap`], or a [`Error`] will be thrown
#[derive(Default)]
pub struct FnScope {
    // mangled
    pub fn_name: String,
    // a counter
    pub alloc_id: usize,
    pub parameters: HashMap<String, GroupIdx>,
    pub scope_stack: Vec<BasicScope>,
    pub declare_map: DeclareMap,
}

impl FnScope {
    pub fn new<'p, I>(fn_name: impl ToString, params: I) -> Self
    where
        I: IntoIterator<Item = (Span, &'p defs::Param)>,
    {
        let mut declare_map = DeclareMap::default();
        let parameters = params
            .into_iter()
            .map(|(at, param)| {
                (
                    param.name.clone(),
                    declare_map.new_static_group(at, std::iter::once(param.ty.clone().into())),
                )
            })
            .collect();

        Self {
            fn_name: fn_name.to_string(),
            parameters,
            scope_stack: vec![Default::default()],
            alloc_id: Default::default(),
            declare_map,
        }
    }

    fn solve_declare(&mut self) -> Vec<Error> {
        self.declare_map.declare_all()
    }

    fn this_scope(&mut self) -> &mut BasicScope {
        self.scope_stack.last_mut().unwrap()
    }

    fn alloc_name(&mut self) -> String {
        (format!(" {}", self.alloc_id), self.alloc_id += 1).0
    }
}

/// usually be folded into other structs,like FnDef, If, While...
#[derive(Default)]
pub struct BasicScope {
    // defines
    pub vars: HashMap<String, defs::VarDef>,
    // statements in scope
    pub stmts: mir::Statements,
}

pub struct CompiledFnScope {
    // mangled
    pub fn_name: String,
    pub stmts: ir::Statements,
    pub errors: Vec<terl::Error>,
}

impl From<FnScope> for CompiledFnScope {
    fn from(mut scope: FnScope) -> Self {
        let errors = scope.solve_declare();
        let fn_name = scope.fn_name;
        let stmts = if errors.is_empty() {
            assert!(scope.scope_stack.len() == 1, "unclosed parse!?");
            let basic_scope = scope.scope_stack.pop().unwrap();
            basic_scope.stmts.into_ir(&scope.declare_map)
        } else {
            vec![]
        };
        Self {
            fn_name,
            stmts,
            errors,
        }
    }
}
