use std::collections::HashMap;

use inkwell::builder::{Builder, BuilderError};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};

/// this is not the most elegant way, but it works for now
#[derive(Debug, Clone)]
pub struct Global<'ctx> {
    scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> Global<'ctx> {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub fn this_scope(&mut self) -> &mut Scope<'ctx> {
        self.scopes.last_mut().unwrap()
    }

    pub fn get_var(&self, name: &str) -> &dyn Variable {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return var;
            }
            if let Some(params) = scope.params.as_ref() {
                return params.get(name).unwrap();
            }
        }
        unreachable!()
    }

    pub fn regist_var(&mut self, name: String, val: AllocVariable<'ctx>) {
        self.this_scope().vars.insert(name, val);
    }

    pub fn regist_params<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (String, ParamVariable<'ctx>)>,
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
#[derive(Default, Debug, Clone)]
pub struct Scope<'ctx> {
    vars: HashMap<String, AllocVariable<'ctx>>,
    params: Option<HashMap<String, ParamVariable<'ctx>>>,
}

pub trait Variable {
    fn load<'s: 'ctx, 'ctx>(
        &'s self,
        builder: &Builder<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, BuilderError>;
    fn store<'s: 'ctx, 'ctx>(
        &'s self,
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

impl Variable for AllocVariable<'_> {
    fn load<'s: 'ctx, 'ctx>(
        &'s self,
        builder: &Builder<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        builder
            .build_load(self.ty, self.pointer, "")
            .map(BasicValueEnum::from)
    }

    fn store<'s: 'ctx, 'ctx>(
        &'s self,
        builder: &Builder<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError> {
        builder.build_store(self.pointer, value).map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct ParamVariable<'ctx> {
    pub inner: BasicValueEnum<'ctx>,
}

impl Variable for ParamVariable<'_> {
    fn load<'s: 'ctx, 'ctx>(
        &'s self,
        _builder: &Builder<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, BuilderError> {
        Ok(self.inner)
    }

    fn store<'s: 'ctx, 'ctx>(
        &'s self,
        _builder: &Builder<'ctx>,
        _value: BasicValueEnum<'ctx>,
    ) -> Result<(), BuilderError> {
        unreachable!("this invalid operation should be flited in ast")
    }
}
