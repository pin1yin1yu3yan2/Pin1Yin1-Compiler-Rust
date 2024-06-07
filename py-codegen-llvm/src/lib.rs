mod codegen;
mod operators;
mod scope;

use std::error::Error;

use codegen::CodeGen;
pub use inkwell;
use inkwell::{context::Context, module::Module};

pub struct LLVMBackend {
    context: Context,
}

impl py_codegen::Backend for LLVMBackend {
    type Error = Box<dyn Error>;
    type Config = ();
    type Module<'ctx> = Module<'ctx>;

    fn init(_config: Self::Config) -> Self {
        Self {
            context: Context::create(),
        }
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

        mod_gen.module.verify().map_err(|e| e.to_string())?;

        Ok(mod_gen.module)
    }
}
