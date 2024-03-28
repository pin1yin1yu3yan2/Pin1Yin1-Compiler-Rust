use std::error::Error;

use crate::scope::{AllocVariable, ComputeResult, Global, Scope, Variable};
use inkwell::{
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};

use pyir::ir;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    position_block: BasicBlock<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    global: Global<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &str,
        stmts: &[ir::Statement],
    ) -> Result<Self, Box<dyn Error>> {
        let module = context.create_module(module);

        // `i32 main()`
        let i32 = context.i32_type();
        let no_return = i32.fn_type(&[], false);
        let main = module.add_function("main", no_return, None);
        let basic_block = context.append_basic_block(main, "entry");

        let builder = context.create_builder();
        builder.position_at_end(basic_block);

        let mut s = Self {
            execution_engine: module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)?,
            context,
            module,
            builder,
            position_block: basic_block,
            global: Default::default(),
        };

        s.generate_stmts(stmts)?;
        let zero = s.context.i32_type().const_int(0, false);
        s.builder.build_return(Some(&zero as &dyn BasicValue))?;

        Ok(s)
    }

    pub fn regist_var<V: Variable<'ctx> + 'ctx>(&mut self, name: String, val: V) {
        self.global.regist_var(name, val)
    }

    pub fn get_var(&self, name: &str) -> &(dyn Variable<'ctx> + 'ctx) {
        self.global.get_var(name)
    }

    pub fn get_fn(&self, name: &str) -> inkwell::values::FunctionValue<'ctx> {
        self.global.get_fn(name)
    }

    pub fn regist_fn(&mut self, name: String, val: inkwell::values::FunctionValue<'ctx>) {
        self.global.regist_fn(name, val)
    }

    fn type_cast(&self, ty: &ir::TypeDefine) -> BasicTypeEnum<'ctx> {
        if ty.decorators.is_some() {
            todo!("decorators is not supported now...")
        }
        let ty: &str = &ty.ty;
        match ty {
            "i64" => self.context.i64_type(),
            "i32" => self.context.i32_type(),
            "i16" => self.context.i16_type(),
            "i8" => self.context.i8_type(),
            "zi4" => self.context.i32_type(),
            _ => todo!("type {} is not supported now...", ty),
        }
        .into()
    }

    fn get_type(&self, expr: &ir::AtomicExpr) -> BasicTypeEnum {
        match expr {
            ir::AtomicExpr::Char(_) => self.context.i32_type().into(),
            ir::AtomicExpr::String(str) => self
                .context
                .i8_type()
                .array_type(str.as_bytes().len() as _)
                .into(),
            ir::AtomicExpr::Integer(_) => self.context.i64_type().into(),
            ir::AtomicExpr::Float(_) => self.context.f32_type().into(),
            ir::AtomicExpr::FnCall(_) => todo!(),
            ir::AtomicExpr::Variable(name) => {
                let get_var = &self.get_var(name);
                get_var.get_type()
            }
        }
    }

    fn atomic_epxr(&self, atomic: &ir::AtomicExpr) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        match atomic {
            ir::AtomicExpr::Char(c) => {
                // char -> u32
                Ok(self.context.i32_type().const_int(*c as _, false).into())
            }
            ir::AtomicExpr::String(s) => {
                let u8 = self.context.i8_type();
                let mut bytes = vec![];
                for byte in s.bytes() {
                    bytes.push(u8.const_int(byte as _, false));
                }
                // &str -> &[u8]
                Ok(u8.const_array(&bytes).into())
            }
            ir::AtomicExpr::Integer(i) => {
                Ok(self.context.i64_type().const_int(*i as _, false).into())
            }
            ir::AtomicExpr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            ir::AtomicExpr::Variable(v) => self.get_var(v).load(&self.builder),

            ir::AtomicExpr::FnCall(fn_call) => {
                let fn_ = self.get_fn(&fn_call.name);
                let args = fn_call.args.iter().try_fold(vec![], |mut vec, arg| {
                    vec.push(self.atomic_epxr(arg)?.into());
                    Ok(vec)
                })?;

                let val = self
                    .builder
                    .build_call(fn_, &args, &fn_call.name)?
                    .try_as_basic_value()
                    // TODO: `kong1` as return type
                    .unwrap_left();
                Ok(val)
            }
        }
    }

    fn generate_stmts(&mut self, stmts: &[ir::Statement]) -> Result<&mut Self, BuilderError> {
        for stmt in stmts {
            stmt.generate(self)?;
        }

        Ok(self)
    }

    fn scope<T, A>(&mut self, action: A) -> T
    where
        A: FnOnce(&mut Self) -> T,
    {
        self.global.scopes.push(Scope::default());
        let t = (action)(self);
        self.global.scopes.pop();
        t
    }

    pub fn llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

pub trait Compile {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError>;
}

impl Compile for ir::Statement {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        match self {
            ir::Statement::FnDefine(f) => f.generate(state),
            ir::Statement::Compute(c) => c.generate(state),
            ir::Statement::VarDefine(v) => v.generate(state),
            ir::Statement::VarStore(v) => v.generate(state),
            ir::Statement::FnCall(f) => f.generate(state),
            ir::Statement::If(i) => i.generate(state),
            ir::Statement::While(w) => w.generate(state),
            ir::Statement::Return(r) => r.generate(state),
            ir::Statement::Block(b) => state.generate_stmts(b).map(|_| ()),
        }
    }
}
impl Compile for ir::FnDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let start_block = state.position_block;

        let retty = state.type_cast(&self.ty);
        let param_ty = self
            .params
            .iter()
            .map(|param| state.type_cast(&param.ty).into())
            .collect::<Vec<_>>();
        let fn_ty = retty.fn_type(&param_ty, false);

        let fn_ = state.module.add_function(&self.name, fn_ty, None);
        state.regist_fn(self.name.clone(), fn_);

        state.scope(move |state| {
            let params = self
                .params
                .iter()
                .enumerate()
                .map(|(idx, param)| (param.name.clone(), fn_.get_nth_param(idx as _).unwrap()))
                .map(|(name, param)| (name, ComputeResult { inner: param }));
            state.global.regist_params(params);
            let entry = state.context.append_basic_block(fn_, "entry");
            state.builder.position_at_end(entry);
            state.generate_stmts(&self.body)?;
            Ok(())
        })?;

        state.builder.position_at_end(start_block);
        Ok(())
    }
}
impl Compile for ir::Compute {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        match &self.eval {
            ir::OperateExpr::Unary(_, _) => todo!(),
            ir::OperateExpr::Binary(op, l, r) => {
                let ty = state.get_type(l);
                let l = state.atomic_epxr(l)?;
                let r = state.atomic_epxr(r)?;
                use pyir::ops::Operators;
                match op {
                    Operators::Add => {
                        let val: BasicValueEnum = if ty.is_int_type() {
                            state
                                .builder
                                .build_int_add(l.into_int_value(), r.into_int_value(), &self.name)?
                                .into()
                        } else if ty.is_float_type() {
                            state
                                .builder
                                .build_float_add(
                                    l.into_float_value(),
                                    r.into_float_value(),
                                    &self.name,
                                )?
                                .into()
                        } else {
                            todo!()
                        };
                        let imv = ComputeResult { inner: val };
                        state.regist_var(self.name.clone(), imv);
                    }
                    Operators::Sub => {
                        let val: BasicValueEnum = if ty.is_int_type() {
                            state
                                .builder
                                .build_int_sub(l.into_int_value(), r.into_int_value(), &self.name)?
                                .into()
                        } else if ty.is_float_type() {
                            state
                                .builder
                                .build_float_sub(
                                    l.into_float_value(),
                                    r.into_float_value(),
                                    &self.name,
                                )?
                                .into()
                        } else {
                            todo!()
                        };
                        let imv = ComputeResult { inner: val };
                        state.regist_var(self.name.clone(), imv);
                    }

                    _ => unimplemented!("OP: {op:?}"),
                }
            }
        };

        Ok(())
    }
}
/// alloca, eval, store
impl Compile for ir::VarDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let ty = state.type_cast(&self.ty);
        let pointer = state.builder.build_alloca(ty, &self.name)?;

        let val = AllocVariable { ty, pointer };

        if let Some(init) = &self.init {
            let init = state.atomic_epxr(init)?;
            val.store(&state.builder, init)?;
        }
        state.regist_var(self.name.clone(), val);
        Ok(())
    }
}
// eval, store
impl Compile for ir::VarStore {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let val = state.atomic_epxr(&self.val)?;
        let s = state.get_var(&self.name);
        s.store(&state.builder, val)?;
        Ok(())
    }
}
// call
impl Compile for ir::FnCall {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let fn_ = state.get_fn(&self.name);
        let args = self.args.iter().try_fold(vec![], |mut vec, arg| {
            vec.push(state.atomic_epxr(arg)?.into());
            Ok(vec)
        })?;

        // droped
        state.builder.build_call(fn_, &args, &self.name)?;
        Ok(())
    }
}
impl Compile for ir::If {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
impl Compile for ir::While {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
impl Compile for ir::Return {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let val = match &self.val {
            Some(val) => Some(state.atomic_epxr(val)?),
            None => None,
        };
        state
            .builder
            .build_return(val.as_ref().map(|v| v as &dyn BasicValue))?;
        Ok(())
    }
}
