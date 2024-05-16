pub trait Backend {
    type Error: std::error::Error + 'static;

    type Config;

    type Module<'m>
    where
        Self: 'm;

    fn init(config: Self::Config) -> Self;

    fn module(&self, name: &str, items: &[py_ir::Item]) -> Result<Self::Module<'_>, Self::Error>;

    fn code(&self, module: &Self::Module<'_>) -> String;
}

pub trait CodeGenerator {
    type Backend: Backend;
}

pub trait CodeGen<CGU>: CodeGenerator {
    fn generate(&mut self, cgu: &CGU) -> Result<(), <Self::Backend as Backend>::Error>;
}
