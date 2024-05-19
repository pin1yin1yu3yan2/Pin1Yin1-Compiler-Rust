mod translate;

pub struct CBackend;

use std::fmt::Write;

use translate::Translate;

pub struct FileModule {
    name: String,
    text: String,
    label_idx: usize,
}

struct Label(String);

impl FileModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            text: String::from("#include <stdint.h>\n#include <stdbool.h>\n#include <math.h>\n"),
            label_idx: 0,
        }
    }

    fn label(&mut self) -> Label {
        let label = format!("L{}", self.label_idx);
        self.label_idx += 1;
        Label(label)
    }

    fn goto(&mut self, label: &Label) -> Result<(), std::fmt::Error> {
        write!(self, "goto {};", label.0)
    }

    fn eol(&mut self) -> Result<(), std::fmt::Error> {
        self.write_char(';')
    }

    fn if_else(
        &mut self,
        cond: &py_ir::value::Value,
        then: &Label,
        or: &Label,
    ) -> Result<(), std::fmt::Error> {
        self.write_str("if(")?;
        self.translate(cond)?;
        self.write_char(')')?;
        self.goto(then)?;
        self.write_str("else ")?;
        self.goto(or)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Write for FileModule {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.text.write_str(s)
    }
}

impl pyc::Backend for CBackend {
    type Error = std::fmt::Error;

    type Config = ();

    type Module<'m> = FileModule
    where
        Self: 'm;

    fn init(_config: Self::Config) -> Self {
        CBackend
    }

    fn module(&self, name: &str, items: &[py_ir::Item]) -> Result<Self::Module<'_>, Self::Error> {
        let mut module = FileModule::new(name.to_string());
        for item in items {
            module.translate(item)?;
        }
        Ok(module)
    }
}
