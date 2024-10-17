use std::collections::HashMap;

use inkwell::builder::{Builder, BuilderError};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

/// this is not the most elegant way, but it works for now
pub struct Defines<'ctx> {
    pub fns: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> Defines<'ctx> {
    pub fn new() -> Self {
        Self {
            fns: Default::default(),
        }
    }

    pub fn get_fn(&self, name: &str) -> FunctionValue<'ctx> {
        *self.fns.get(name).unwrap()
    }

    pub fn regist_fn(&mut self, name: String, val: FunctionValue<'ctx>) {
        self.fns.insert(name, val);
    }
}

impl Default for Defines<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// scope is still necessary bacause variable may be shadowed in scope
#[derive(Default)]
pub struct FnScope<'ctx> {
    pub vars: Vec<HashMap<String, Box<dyn Variable<'ctx> + 'ctx>>>,
    pub params: HashMap<String, ComputeResult<'ctx>>,
}

impl<'ctx> FnScope<'ctx> {
    pub fn new<I>(params: I) -> Self
    where
        I: IntoIterator<Item = (String, ComputeResult<'ctx>)>,
    {
        Self {
            // CodeGen for Statemnts will create a template map
            // so, its not necessary to create a map while creating FnScope
            vars: vec![],
            params: params.into_iter().collect(),
        }
    }
}

pub trait Variable<'ctx> {
    fn load(&self, builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>, BuilderError>;
    fn store(
        &self,
        builder: &Builder<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError>;
}

/// variables from allocation, like heap/stack variables

#[derive(Debug, Clone)]
pub struct AllocVariable<'ctx> {
    pub ty: BasicTypeEnum<'ctx>,
    pub pointer: PointerValue<'ctx>,
}

impl<'ctx> Variable<'ctx> for AllocVariable<'ctx> {
    fn load(&self, builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        builder
            .build_load(self.ty, self.pointer, "")
            .map(BasicValueEnum::from)
    }
    fn store(
        &self,
        builder: &Builder<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError> {
        builder.build_store(self.pointer, value).map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct ComputeResult<'ctx> {
    pub val: BasicValueEnum<'ctx>,
}

impl<'ctx> Variable<'ctx> for ComputeResult<'ctx> {
    fn load(&self, _builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        Ok(self.val)
    }

    fn store(
        &self,
        _builder: &Builder<'ctx>,
        _value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError> {
        unreachable!("this invalid operation should be filtered in mir")
    }
}
