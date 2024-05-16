use crate::scope::{AllocVariable, ComputeResult, Defines, FnScope, Variable};
use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, FunctionValue},
};
use py_ir::value as ir_value;
use py_ir::value::Value as IRValue;
use py_ir::{types as ir_types, ControlFlow};
use py_lex::SharedString;
use pyc::*;

pub struct ModuleGen<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub defines: Defines<'ctx>,
}

fn type_scast<'ctx>(context: &'ctx Context, ty: &ir_types::TypeDefine) -> BasicTypeEnum<'ctx> {
    use ir_types::*;

    match ty {
        TypeDefine::Primitive(ty) => match ty {
            PrimitiveType::Bool => context.bool_type().into(),
            PrimitiveType::I8 | PrimitiveType::U8 => context.i8_type().into(),
            PrimitiveType::I16 | PrimitiveType::U16 => context.i16_type().into(),
            PrimitiveType::I32 | PrimitiveType::U32 => context.i32_type().into(),
            PrimitiveType::I64 | PrimitiveType::U64 => context.i64_type().into(),
            PrimitiveType::I128 | PrimitiveType::U128 => context.i128_type().into(),
            PrimitiveType::Usize | PrimitiveType::Isize => context.i64_type().into(),
            PrimitiveType::F32 => context.f32_type().into(),
            PrimitiveType::F64 => context.f64_type().into(),
        },
        TypeDefine::Complex(_ty) => {
            todo!()
        }
    }
}

impl<'ctx> ModuleGen<'ctx> {
    fn type_cast(&self, ty: &ir_types::TypeDefine) -> BasicTypeEnum<'ctx> {
        type_scast(self.context, ty)
    }
}

impl CodeGenerator for ModuleGen<'_> {
    type Backend = crate::LLVMBackend;
}

impl CodeGen<py_ir::Item> for ModuleGen<'_> {
    fn generate(&mut self, cgu: &py_ir::Item) -> Result<(), BuilderError> {
        match cgu {
            py_ir::Item::FnDefine(cgu) => self.generate(cgu),
        }
    }
}

impl CodeGen<py_ir::FnDefine<IRValue>> for ModuleGen<'_> {
    fn generate(&mut self, cgu: &py_ir::FnDefine<IRValue>) -> Result<(), BuilderError> {
        let retty = self.type_cast(&cgu.ty);
        let param_ty = cgu
            .params
            .iter()
            .map(|param| self.type_cast(&param.ty).into())
            .collect::<Vec<_>>();
        let fn_ty = retty.fn_type(&param_ty, false);

        let fn_ = self.module.add_function(&cgu.name, fn_ty, None);
        self.defines.regist_fn(cgu.name.clone(), fn_);
        let entry = self.context.append_basic_block(fn_, "entry");
        self.builder.position_at_end(entry);

        let params = cgu
            .params
            .iter()
            .enumerate()
            .map(|(idx, param)| (param.name.clone(), fn_.get_nth_param(idx as _).unwrap()))
            .map(|(name, param)| (name, ComputeResult { val: param }));

        let mut fn_gen = FnGen {
            context: self.context,
            builder: &self.builder,
            defines: &mut self.defines,
            current_fn: fn_,
            fn_scope: FnScope::new(params),
        };

        fn_gen.generate(&cgu.body)?;

        Ok(())
    }
}

struct FnGen<'mg, 'ctx> {
    context: &'ctx Context,
    builder: &'mg Builder<'ctx>,
    defines: &'mg mut Defines<'ctx>,
    current_fn: FunctionValue<'ctx>,
    fn_scope: FnScope<'ctx>,
}

impl<'mg, 'ctx> FnGen<'mg, 'ctx> {
    fn type_cast(&self, ty: &ir_types::TypeDefine) -> BasicTypeEnum<'ctx> {
        type_scast(self.context, ty)
    }

    fn get_val(&self, name: &str) -> &(dyn Variable<'ctx> + 'ctx) {
        self.fn_scope
            .params
            .get(name)
            .map(|val| val as &dyn Variable)
            .or_else(|| {
                self.fn_scope
                    .vars
                    .iter()
                    .rev()
                    .find_map(|map| map.get(name))
                    .map(|val| &**val)
            })
            .unwrap()
    }

    pub fn regist_var<V: Variable<'ctx> + 'ctx>(&mut self, name: SharedString, val: V) {
        self.fn_scope
            .vars
            .last_mut()
            .unwrap()
            .insert(name, Box::new(val));
    }
}

impl<'ctx> std::ops::Deref for FnGen<'_, 'ctx> {
    type Target = Defines<'ctx>;

    fn deref(&self) -> &Self::Target {
        self.defines
    }
}

impl<'ctx> std::ops::DerefMut for FnGen<'_, 'ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.defines
    }
}

impl CodeGenerator for FnGen<'_, '_> {
    type Backend = crate::LLVMBackend;
}

impl<'ctx> FnGen<'_, 'ctx> {
    fn literal(
        &self,
        literal: &ir_value::Literal,
        ty: &ir_types::PrimitiveType,
    ) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        let ret = match literal {
            ir_value::Literal::Integer(int) if ty.is_integer() => match ty.width() {
                1 => self
                    .context
                    .bool_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                8 => self
                    .context
                    .i8_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                16 => self
                    .context
                    .i16_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                32 => self
                    .context
                    .i32_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                64 => self
                    .context
                    .i64_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                128 => self
                    .context
                    .i128_type()
                    .const_int(*int as _, ty.is_signed())
                    .into(),
                _ => unreachable!(),
            },
            ir_value::Literal::Float(float) if ty.is_float() => match ty.width() {
                32 => self.context.f32_type().const_float(*float).into(),
                64 => self.context.f64_type().const_float(*float).into(),
                _ => unreachable!(),
            },
            ir_value::Literal::Char(char) if ty == &ir_types::PrimitiveType::U32 => self
                .context
                .i32_type()
                .const_int(*char as _, ty.is_signed())
                .into(),
            _ => panic!("incorrect PrimitiveType are passed in"),
        };
        Ok(ret)
    }

    fn eval(&self, var: &IRValue) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        match var {
            IRValue::FnCall(fn_call) => {
                let fn_ = self.get_fn(&fn_call.fn_name);
                let args = fn_call.args.iter().try_fold(vec![], |mut vec, arg| {
                    vec.push(self.eval(arg)?.into());
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
            IRValue::Variable(variable) => self.get_val(variable).load(self.builder),
            IRValue::Literal(literal, ty) => self.literal(literal, ty),
        }
    }
}

impl CodeGen<py_ir::Statements<IRValue>> for FnGen<'_, '_> {
    fn generate(
        &mut self,
        cgu: &py_ir::Statements<IRValue>,
    ) -> Result<(), <Self::Backend as Backend>::Error> {
        self.fn_scope.vars.push(Default::default());
        for cgu in &**cgu {
            self.generate(cgu)?;
        }
        self.fn_scope.vars.pop();
        Ok(())
    }
}

/// same as the implementation for [`py_ir::Statements`] but no new scope created to make
/// condition val usable
impl CodeGen<py_ir::Condition<IRValue>> for FnGen<'_, '_> {
    fn generate(
        &mut self,
        cgu: &py_ir::Condition<IRValue>,
    ) -> Result<(), <Self::Backend as Backend>::Error> {
        for cgu in &*cgu.compute {
            self.generate(cgu)?;
        }
        Ok(())
    }
}

impl CodeGen<py_ir::Statement<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::Statement<IRValue>) -> Result<(), BuilderError> {
        match cgu {
            py_ir::Statement::Compute(cgu) => self.generate(cgu),
            py_ir::Statement::VarDefine(cgu) => self.generate(cgu),
            py_ir::Statement::VarStore(cgu) => self.generate(cgu),
            py_ir::Statement::If(cgu) => self.generate(cgu),
            py_ir::Statement::While(cgu) => self.generate(cgu),
            py_ir::Statement::Return(cgu) => self.generate(cgu),
            py_ir::Statement::Block(cgu) => self.generate(cgu),
        }
    }
}

impl CodeGen<py_ir::Compute<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::Compute<IRValue>) -> Result<(), BuilderError> {
        let builder = &self.builder;
        match &cgu.eval {
            py_ir::OperateExpr::Unary(op, val) => {
                let val = self.eval(val)?;
                let val = crate::primitive::unary_compute(builder, cgu.ty, *op, val, &cgu.name)?;
                self.regist_var(cgu.name.clone(), ComputeResult { val })
            }
            py_ir::OperateExpr::Binary(op, l, r) => {
                let l = self.eval(l)?;
                let r = self.eval(r)?;
                let val = crate::primitive::binary_compute(builder, cgu.ty, *op, l, r, &cgu.name)?;
                self.regist_var(cgu.name.clone(), ComputeResult { val })
            }
        };

        Ok(())
    }
}

/// alloca, eval, store
impl CodeGen<py_ir::VarDefine<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::VarDefine<IRValue>) -> Result<(), BuilderError> {
        let ty = self.type_cast(&cgu.ty);
        let pointer = self.builder.build_alloca(ty, &cgu.name)?;

        let val = AllocVariable { ty, pointer };

        if let Some(init) = &cgu.init {
            let init = self.eval(init)?;
            val.store(self.builder, init)?;
        }
        self.regist_var(cgu.name.clone(), val);
        Ok(())
    }
}

// eval, store
impl CodeGen<py_ir::VarStore<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::VarStore<IRValue>) -> Result<(), BuilderError> {
        let val = self.eval(&cgu.val)?;
        let s = self.get_val(&cgu.name);
        s.store(self.builder, val)?;
        Ok(())
    }
}

// call
impl CodeGen<py_ir::value::FnCall<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::value::FnCall<IRValue>) -> Result<(), BuilderError> {
        let fn_ = self.get_fn(&cgu.fn_name);
        let args = cgu.args.iter().try_fold(vec![], |mut vec, arg| {
            vec.push(self.eval(arg)?.into());
            Ok(vec)
        })?;

        self.builder.build_call(fn_, &args, &cgu.fn_name)?;
        Ok(())
    }
}

impl CodeGen<py_ir::If<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::If<IRValue>) -> Result<(), BuilderError> {
        let after_exist = !cgu.returned();

        // note: conds.last() is `else block`(if exist), and br into codes.last()
        let conds = (0..cgu.branches.len() + 1)
            .map(|_| self.context.append_basic_block(self.current_fn, ""))
            .collect::<Vec<_>>();
        let codes = (0..cgu.branches.len() + after_exist as usize)
            .map(|_| self.context.append_basic_block(self.current_fn, ""))
            .collect::<Vec<_>>();

        // jump to eval first condition
        self.builder.build_unconditional_branch(conds[0])?;

        let else_block = conds.last().copied().unwrap();
        let code_after = codes.last().copied().unwrap();

        for idx in 0..cgu.branches.len() {
            self.builder.position_at_end(conds[idx]);
            let condition = &cgu.branches[idx].cond;

            self.generate(condition)?;
            let cond_val = self.eval(&condition.val)?.into_int_value();
            // br <cond> <code> <else>
            self.builder
                .build_conditional_branch(cond_val, codes[idx], conds[idx + 1])?;

            // generate if body
            self.builder.position_at_end(codes[idx]);
            self.generate(&cgu.branches[idx].body)?;

            if after_exist && !cgu.branches[idx].returned() {
                self.builder.build_unconditional_branch(code_after)?;
            }
        }

        self.builder.position_at_end(else_block);
        if let Some(else_) = &cgu.else_ {
            self.generate(else_)?;
        }

        if after_exist {
            self.builder.build_unconditional_branch(code_after)?;
            self.builder.position_at_end(code_after);
        }

        Ok(())
    }
}

impl CodeGen<py_ir::While<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::While<IRValue>) -> Result<(), BuilderError> {
        let cond = self.context.append_basic_block(self.current_fn, "");
        let code = self.context.append_basic_block(self.current_fn, "");
        let after = self.context.append_basic_block(self.current_fn, "");

        self.builder.position_at_end(cond);

        self.generate(&cgu.cond)?;
        let cond_val = self.eval(&cgu.cond.val)?.into_int_value();
        self.builder
            .build_conditional_branch(cond_val, code, after)?;

        self.builder.position_at_end(code);
        self.generate(&cgu.body)?;
        self.builder.build_unconditional_branch(cond)?;

        self.builder.position_at_end(after);

        Ok(())
    }
}

impl CodeGen<py_ir::Return<IRValue>> for FnGen<'_, '_> {
    fn generate(&mut self, cgu: &py_ir::Return<IRValue>) -> Result<(), BuilderError> {
        let val = match &cgu.val {
            Some(val) => Some(self.eval(val)?),
            None => None,
        };
        self.builder
            .build_return(val.as_ref().map(|v| v as &dyn BasicValue))?;
        Ok(())
    }
}
