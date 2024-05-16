use std::borrow::Cow;

pub trait Backend {
    type Error;

    type Config;

    type Module<'m>
    where
        Self: 'm;

    fn init(config: Self::Config) -> Self;

    fn module(&self, name: &str, items: &[py_ir::Item]) -> Result<Self::Module<'_>, Self::Error>;

    fn code<'m>(&self, module: &'m Self::Module<'_>) -> Cow<'m, str>;
}
