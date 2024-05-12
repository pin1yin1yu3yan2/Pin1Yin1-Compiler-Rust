use std::collections::HashMap;

use inkwell::builder::{Builder, BuilderError};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use py_lex::SharedString;

/// this is not the most elegant way, but it works for now

pub struct Global<'ctx> {
    pub fns: HashMap<SharedString, FunctionValue<'ctx>>,
    pub scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> Global<'ctx> {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            fns: Default::default(),
        }
    }

    pub fn this_scope(&mut self) -> &mut Scope<'ctx> {
        self.scopes.last_mut().unwrap()
    }

    pub fn get_var(&self, name: &str) -> &(dyn Variable<'ctx> + 'ctx) {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return &**var;
            }
            if let Some(params) = scope.params.as_ref() {
                return params.get(name).unwrap();
            }
        }
        unreachable!("{name}")
    }

    pub fn get_fn(&self, name: &str) -> FunctionValue<'ctx> {
        *self.fns.get(name).unwrap()
    }

    pub fn regist_var<V: Variable<'ctx> + 'ctx>(&mut self, name: SharedString, val: V) {
        self.this_scope().vars.insert(name, Box::new(val));
    }

    pub fn regist_fn(&mut self, name: SharedString, val: FunctionValue<'ctx>) {
        self.fns.insert(name, val);
    }

    pub fn regist_params<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (SharedString, ComputeResult<'ctx>)>,
    {
        assert!(self.this_scope().params.is_none());
        self.this_scope().params = Some(iter.into_iter().collect())
    }
}

impl<'ctx> Default for Global<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

/// scope is still necessary bacause variable may be shadowed in scope
#[derive(Default)]
pub struct Scope<'ctx> {
    vars: HashMap<SharedString, Box<dyn Variable<'ctx> + 'ctx>>,
    params: Option<HashMap<SharedString, ComputeResult<'ctx>>>,
}

pub trait Variable<'ctx> {
    fn load(&self, builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>, BuilderError>;
    fn store(
        &self,
        builder: &Builder<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError>;
    fn get_type(&self) -> BasicTypeEnum<'ctx>;
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
    fn get_type(&self) -> BasicTypeEnum<'ctx> {
        self.ty
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
        unreachable!("this invalid operation should be filtered in ast")
    }

    fn get_type(&self) -> BasicTypeEnum<'ctx> {
        self.val.get_type()
    }
}
