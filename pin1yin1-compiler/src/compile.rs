use std::error::Error;

use crate::scope::{AllocVariable, Global, ImmediateValue, Scope, Variable};
use inkwell::{
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};
use pin1yin1_ast::ast;

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
        stmts: &[ast::Statement],
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

    pub fn get_var(&self, name: &str) -> &(dyn Variable<'ctx> + 'ctx) {
        self.global.get_var(name)
    }

    pub fn regist_var<V: Variable<'ctx> + 'ctx>(&mut self, name: String, val: V) {
        self.global.regist_var(name, val)
    }

    fn type_cast(&self, ty: &ast::TypeDefine) -> BasicTypeEnum<'ctx> {
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

    fn get_type(&self, expr: &ast::AtomicExpr) -> BasicTypeEnum {
        match expr {
            ast::AtomicExpr::Char(_) => self.context.i32_type().into(),
            ast::AtomicExpr::String(str) => self
                .context
                .i8_type()
                .array_type(str.as_bytes().len() as _)
                .into(),
            ast::AtomicExpr::Integer(_) => self.context.i64_type().into(),
            ast::AtomicExpr::Float(_) => self.context.f32_type().into(),
            ast::AtomicExpr::FnCall(_) => todo!(),
            ast::AtomicExpr::Variable(name) => {
                let get_var = &self.get_var(name);
                get_var.get_type()
            }
        }
    }

    fn atomic_epxr(&self, atomic: &ast::AtomicExpr) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        match atomic {
            ast::AtomicExpr::Char(c) => {
                // char -> u32
                Ok(self.context.i32_type().const_int(*c as _, false).into())
            }
            ast::AtomicExpr::String(s) => {
                let u8 = self.context.i8_type();
                let mut bytes = vec![];
                for byte in s.bytes() {
                    bytes.push(u8.const_int(byte as _, false));
                }
                // &str -> &[u8]
                Ok(u8.const_array(&bytes).into())
            }
            ast::AtomicExpr::Integer(i) => {
                Ok(self.context.i64_type().const_int(*i as _, false).into())
            }
            ast::AtomicExpr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            ast::AtomicExpr::Variable(v) => self.get_var(v).load(&self.builder),

            _ => todo!("initalizition is not supported now..."),
        }
    }

    fn generate_stmts(&mut self, stmts: &[ast::Statement]) -> Result<&mut Self, BuilderError> {
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

impl Compile for ast::Statement {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        match self {
            ast::Statement::FnDefine(f) => f.generate(state),
            ast::Statement::Compute(c) => c.generate(state),
            ast::Statement::VarDefine(v) => v.generate(state),
            ast::Statement::VarStore(v) => v.generate(state),
            ast::Statement::FnCall(f) => f.generate(state),
            ast::Statement::If(i) => i.generate(state),
            ast::Statement::While(w) => w.generate(state),
            ast::Statement::Return(r) => r.generate(state),
            ast::Statement::Block(b) => state.generate_stmts(b).map(|_| ()),
        }
    }
}
impl Compile for ast::FnDefine {
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

        state.scope(move |state| {
            let params = self
                .params
                .iter()
                .enumerate()
                .map(|(idx, param)| (param.name.clone(), fn_.get_nth_param(idx as _).unwrap()))
                .map(|(name, param)| (name, ImmediateValue { inner: param }));
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
impl Compile for ast::Compute {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        match &self.eval {
            ast::OperateExpr::Unary(_, _) => todo!(),
            ast::OperateExpr::Binary(op, l, r) => {
                let ty = state.get_type(l);
                let l = state.atomic_epxr(l)?;
                let r = state.atomic_epxr(r)?;
                use pin1yin1_ast::keywords::operators::Operators;
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
                        let imv = ImmediateValue { inner: val };
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
                        let imv = ImmediateValue { inner: val };
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
impl Compile for ast::VarDefine {
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
impl Compile for ast::VarStore {
    fn generate(&self, state: &mut CodeGen) -> Result<(), BuilderError> {
        let val = state.atomic_epxr(&self.val)?;
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
        use inkwell::values::BasicValue;
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
