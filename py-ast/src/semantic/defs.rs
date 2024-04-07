use terl::Span;

use crate::ir;

pub struct FnDef {
    // Vec is enough
    pub overloads: Vec<FnSign>,
}

impl FnDef {
    pub fn new(overloads: Vec<FnSign>) -> Self {
        Self { overloads }
    }
}

pub struct FnSign {
    /// mangled name (the real name which be used in symbol table)
    pub mangle: String,
    /// return type of the function must be cleared (will change in future versions)
    pub ty: ir::TypeDefine,
    pub params: Vec<Param>,
    pub loc: Span,
}

pub struct Param {
    pub name: String,
    pub var_def: VarDef,
}

impl std::ops::Deref for Param {
    type Target = VarDef;

    fn deref(&self) -> &Self::Target {
        &self.var_def
    }
}

pub struct VarDef {
    pub ty: ir::TypeDefine,
    pub loc: Span,
}

impl VarDef {
    pub fn new(ty: ir::TypeDefine, loc: Span) -> Self {
        Self { ty, loc }
    }
}
