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

use py_ir::ir;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

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

        let builder = context.create_builder();

        let mut s = Self {
            execution_engine: module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)?,
            context,
            module,
            builder,

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
        match ty {
            ir::TypeDefine::Primitive(ty) => match ty {
                ir::PrimitiveType::Bool => self.context.bool_type().into(),
                ir::PrimitiveType::I8 | ir::PrimitiveType::U8 => self.context.i8_type().into(),
                ir::PrimitiveType::I16 | ir::PrimitiveType::U16 => self.context.i16_type().into(),
                ir::PrimitiveType::I32 | ir::PrimitiveType::U32 => self.context.i32_type().into(),
                ir::PrimitiveType::I64 | ir::PrimitiveType::U64 => self.context.i64_type().into(),
                ir::PrimitiveType::I128 | ir::PrimitiveType::U128 => {
                    self.context.i128_type().into()
                }
                ir::PrimitiveType::Usize | ir::PrimitiveType::Isize => {
                    self.context.i64_type().into()
                }
                ir::PrimitiveType::F32 => self.context.f32_type().into(),
                ir::PrimitiveType::F64 => self.context.f64_type().into(),
            },
            ir::TypeDefine::Complex(_ty) => {
                todo!()
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
                let fn_ = self.get_fn(&fn_call.fn_name);
                let args = fn_call.args.iter().try_fold(vec![], |mut vec, arg| {
                    vec.push(self.atomic_epxr(arg)?.into());
                    Ok(vec)
                })?;

                let val = self
                    .builder
                    .build_call(fn_, &args, &fn_call.fn_name)?
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
            ir::Statement::If(i) => i.generate(state),
            ir::Statement::While(w) => w.generate(state),
            ir::Statement::Return(r) => r.generate(state),
            ir::Statement::Block(b) => state.generate_stmts(b).map(|_| ()),
        }
    }
}
impl Compile for ir::FnDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
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
                .map(|(name, param)| (name, ComputeResult { val: param }));
            state.global.regist_params(params);
            let entry = state.context.append_basic_block(fn_, "entry");
            state.builder.position_at_end(entry);
            state.generate_stmts(&self.body)?;
            Ok(())
        })?;

        Ok(())
    }
}
impl Compile for ir::Compute {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let builder = &state.builder;
        match &self.eval {
            ir::OperateExpr::Unary(op, val) => {
                let val = state.atomic_epxr(val)?;
                let val = crate::primitive::unary_compute(builder, self.ty, *op, val, &self.name)?;
                state.regist_var(self.name.to_owned(), ComputeResult { val })
            }
            ir::OperateExpr::Binary(op, l, r) => {
                let l = state.atomic_epxr(l)?;
                let r = state.atomic_epxr(r)?;
                let val =
                    crate::primitive::binary_compute(builder, self.ty, *op, l, r, &self.name)?;
                state.regist_var(self.name.to_owned(), ComputeResult { val })
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
        let fn_ = state.get_fn(&self.fn_name);
        let args = self.args.iter().try_fold(vec![], |mut vec, arg| {
            vec.push(state.atomic_epxr(arg)?.into());
            Ok(vec)
        })?;

        // droped
        state.builder.build_call(fn_, &args, &self.fn_name)?;
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
