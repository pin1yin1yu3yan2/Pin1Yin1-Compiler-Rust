use crate::ast;
use std::{collections::HashMap, marker::PhantomData};

#[cfg(feature = "parser")]
use crate::parse;

#[derive(Default, Debug, Clone)]
pub struct FnDefinitions<'ast, 's> {
    pub map: HashMap<String, FnDefinition<'ast, 's>>,
}

#[derive(Default, Debug, Clone)]
pub struct FnDefinition<'ast, 's> {
    /// functions have same names but different signatures
    pub overdrives: Vec<FnSign>,
    #[cfg(feature = "parser")]
    pub raw_defines: Vec<Box<&'ast parse::FunctionDefine<'s>>>,
    _p: PhantomData<&'ast &'s ()>,
}

#[derive(Debug, Clone)]
pub struct FnSign {
    pub type_: ast::TypeDefine,
    pub params: Vec<ast::TypeDefine>,
}

#[derive(Default, Debug, Clone)]
pub struct VarDefinitions<'ast, 's> {
    pub map: HashMap<String, VarDefinition<'ast, 's>>,
}

#[derive(Debug, Clone)]
pub struct VarDefinition<'ast, 's> {
    pub type_: ast::TypeDefine,
    #[cfg(feature = "parser")]
    pub raw_define: &'ast parse::VariableDefine<'s>,
    _p: PhantomData<&'ast &'s ()>,
}
