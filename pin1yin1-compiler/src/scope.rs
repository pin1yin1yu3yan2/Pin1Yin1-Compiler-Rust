use inkwell::builder::{Builder, BuilderError};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};

// use pin1yin1_ast::semantic::Global as AstGlobal;
// use pin1yin1_ast::semantic::Scope as AstScope;

pub struct Scope {}
pub struct Global {
    scopes: Vec<Scope>,
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
pub struct AllocVariable<'ctx> {
    pub ty: BasicTypeEnum<'ctx>,
    pub pointer: PointerValue<'ctx>,
}

pub struct ParamVariable {}

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
