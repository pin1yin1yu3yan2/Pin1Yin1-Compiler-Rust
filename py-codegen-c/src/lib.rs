mod translate;

pub struct CBackend;

use std::fmt::Write;

use translate::Translate;

#[derive(Clone, Copy, PartialEq)]
pub enum Buffer {
    C,
    H,
}

pub struct FileModule {
    name: String,
    buffer: Buffer,
    c_file: String,
    h_file: String,
    label_idx: usize,
}

struct Label(String);

impl FileModule {
    pub fn new(name: String) -> Self {
        const HEADER_FILES: &str = "#include <math.h>\n#include <stdbool.h>\n#include <stdint.h>\n";
        Self {
            name,
            buffer: Buffer::C,
            c_file: String::from(HEADER_FILES),
            h_file: String::from(HEADER_FILES),
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

    fn swap_to(&mut self, target: Buffer) {
        if self.buffer != target {
            std::mem::swap(&mut self.c_file, &mut self.h_file);
            self.buffer = target;
        }
    }

    pub fn write_header_file(
        &mut self,
        writer: impl FnOnce(&mut Self) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.swap_to(Buffer::H);
        writer(self)?;
        self.swap_to(Buffer::C);
        Ok(())
    }

    pub fn write_source_file(
        &mut self,
        writer: impl FnOnce(&mut Self) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.swap_to(Buffer::C);
        writer(self)
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

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn c_file(&self) -> &str {
        &self.c_file
    }

    #[inline]
    pub fn h_file(&self) -> &str {
        &self.h_file
    }
}

impl std::fmt::Write for FileModule {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.c_file.write_str(s)
    }
}

impl py_codegen::Backend for CBackend {
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
