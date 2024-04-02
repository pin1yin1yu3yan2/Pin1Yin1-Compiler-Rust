use crate::ir;
use std::marker::PhantomData;

use crate::parse;

pub struct FnDef<'ast> {
    pub overloads: Vec<FnSign<'ast>>,
}

impl<'ast> FnDef<'ast> {
    pub fn new(overloads: Vec<FnSign<'ast>>) -> Self {
        Self { overloads }
    }
}

pub struct FnSign<'ast> {
    pub mangle: String,
    pub ty: ir::TypeDefine,
    pub params: Vec<Param<'ast>>,
    pub raw: &'ast parse::FnDefine,
}

pub struct Param<'ast> {
    pub name: String,
    pub var_def: VarDef<'ast>,
    pub _p: PhantomData<&'ast ()>,
}

impl<'ast> std::ops::Deref for Param<'ast> {
    type Target = VarDef<'ast>;

    fn deref(&self) -> &Self::Target {
        &self.var_def
    }
}

pub struct VarDef<'ast> {
    pub ty: ir::TypeDefine,

    pub raw_define: &'ast parse::VarDefine,
    _p: PhantomData<&'ast ()>,
}

impl<'ast> VarDef<'ast> {
    pub fn new(ty: ir::TypeDefine, raw_define: &'ast parse::VarDefine) -> Self {
        Self {
            ty,

            raw_define,
            _p: PhantomData,
        }
    }
}
