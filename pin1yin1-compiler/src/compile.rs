use std::error::Error;

use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::BasicTypeEnum,
    values::BasicValueEnum,
};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    global: Global<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module: &str) -> Result<Self, Box<dyn Error>> {
        let module = context.create_module(module);

        // `void main()`
        let void = context.void_type();
        let no_return = void.fn_type(&[], false);
        let main = module.add_function("main", no_return, None);
        let basic_block = context.append_basic_block(main, "entry");

        let builder = context.create_builder();
        builder.position_at_end(basic_block);

        Ok(Self {
            execution_engine: module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)?,
            builder,
            context,
            module,
            global: Default::default(),
        })
    }

    pub fn get_var(&self, name: &str) -> &dyn Variable {
        self.global.get_var(name)
    }

    pub fn regist_params<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (String, crate::scope::ParamVariable<'ctx>)>,
    {
        self.global.regist_params(iter)
    }

    pub fn regist_var(&mut self, name: String, val: AllocVariable<'ctx>) {
        self.global.regist_var(name, val)
    }

    fn type_cast(&self, ty: &ast::TypeDefine) -> BasicTypeEnum<'ctx> {
        if !ty.decorators.is_empty() {
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

    fn atomic_epxr(&self, atomic: &ast::AtomicExpr) -> Result<BasicValueEnum, BuilderError> {
        let r = match atomic {
            ast::AtomicExpr::Char(c) => {
                // char -> u32
                self.context.i32_type().const_int(*c as _, false).into()
            }
            ast::AtomicExpr::String(s) => {
                let u8 = self.context.i8_type();
                let mut bytes = vec![];
                for byte in s.bytes() {
                    bytes.push(u8.const_int(byte as _, false));
                }
                // &str -> &[u8]
                u8.const_array(&bytes).into()
            }
            ast::AtomicExpr::Integer(i) => self.context.i64_type().const_int(*i as _, false).into(),
            ast::AtomicExpr::Float(f) => self.context.f64_type().const_float(*f).into(),
            ast::AtomicExpr::Variable(v) => self.get_var(v).load(&self.builder)?,

            _ => todo!("initalizition is not supported now..."),
        };
        Ok(r)
    }

    fn eval(&self, expr: &ast::OperateExpr) -> Result<BasicValueEnum, BuilderError> {
        match expr {
            ast::OperateExpr::Unary(_, _) => todo!(),
            ast::OperateExpr::Binary(_, _, _) => todo!(),
            ast::OperateExpr::NoOp(val) => self.atomic_epxr(val),
        }
    }
}

pub trait Compile {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError>;
}

use pin1yin1_ast::ast;

use crate::scope::{AllocVariable, Global, Variable};

impl Compile for ast::Statement {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        match self {
            ast::Statement::FnDefine(f) => f.generate(state),
            ast::Statement::VarDefine(v) => v.generate(state),
            ast::Statement::VarStore(v) => v.generate(state),
            ast::Statement::FnCall(f) => f.generate(state),
            ast::Statement::If(i) => i.generate(state),
            ast::Statement::While(w) => w.generate(state),
            ast::Statement::Return(r) => r.generate(state),
            ast::Statement::Block(b) => {
                for s in b {
                    s.generate(state)?;
                }
                Ok(())
            }
        }
    }
}
impl Compile for ast::FnDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}

/// alloca, eval, store
impl Compile for ast::VarDefine {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let ty = state.type_cast(&self.ty);
        let pointer = state.builder.build_alloca(ty, &self.name)?;

        let val = AllocVariable { ty, pointer };

        if let Some(init) = &self.init {
            let init = state.eval(init)?;
            val.store(&state.builder, init)?;
        }
        state.regist_var(self.name.clone(), val);
        Ok(())
    }
}
// eval, store
impl Compile for ast::VarStore {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let val = state.atomic_epxr(&self.val.val)?;
        let s = state.get_var(&self.name);
        s.store(&state.builder, val)?;
        Ok(())
    }
}
// call
impl Compile for ast::FnCall {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
impl Compile for ast::If {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
impl Compile for ast::While {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
impl Compile for ast::Return {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        todo!()
    }
}
