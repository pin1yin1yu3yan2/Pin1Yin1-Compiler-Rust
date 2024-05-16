mod codegen;
mod primitive;
mod scope;

use std::borrow::Cow;

use codegen::CodeGen;
pub use inkwell;
use inkwell::{context::Context, module::Module};

pub struct LLVMBackend {
    context: Context,
}

impl LLVMBackend {
    pub fn new() -> Self {
        Self {
            context: Context::create(),
        }
    }
}

impl Default for LLVMBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl pyc::Backend for LLVMBackend {
    type Error = inkwell::builder::BuilderError;
    type Config = ();
    type Module<'ctx> = Module<'ctx>;

    fn init(_config: Self::Config) -> Self {
        Self::new()
    }

    fn module(&self, name: &str, items: &[py_ir::Item]) -> Result<Module<'_>, Self::Error> {
        let mut mod_gen = codegen::ModuleGen {
            context: &self.context,
            builder: self.context.create_builder(),
            module: self.context.create_module(name),
            defines: Default::default(),
        };
        for item in items {
            mod_gen.generate(item)?;
        }
        Ok(mod_gen.module)
    }

    // fn code<'s>(&'s self, module: &'s Self::Module<'s>) -> Cow<'s, str> {
    //     module.to_string().into()
    // }
    fn code<'m>(&self, module: &'m Self::Module<'_>) -> Cow<'m, str> {
        module.to_string().into()
    }
}
