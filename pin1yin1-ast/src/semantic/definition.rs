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
    ///
    /// unsupport now
    pub overdrives: Vec<FnSign>,
    #[cfg(feature = "parser")]
    pub raw_defines: Vec<&'ast parse::FnDefine<'s>>,
    _p: PhantomData<&'ast &'s ()>,
}

impl<'ast, 's> FnDefinition<'ast, 's> {
    pub fn new(
        overdrives: Vec<FnSign>,
        #[cfg(feature = "parser")] raw_defines: Vec<&'ast parse::FnDefine<'s>>,
    ) -> Self {
        Self {
            overdrives,
            #[cfg(feature = "parser")]
            raw_defines,
            _p: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnSign {
    pub mangle: String,
    pub ty: ast::TypeDefine,
    pub params: Vec<ast::TypeDefine>,
}

impl FnSign {
    pub fn new(mangle: String, ty: ast::TypeDefine, params: Vec<ast::TypeDefine>) -> Self {
        Self { mangle, ty, params }
    }
}

#[derive(Default, Debug, Clone)]
pub struct VarDefinitions<'ast, 's> {
    pub map: HashMap<String, VarDefinition<'ast, 's>>,
}

#[derive(Debug, Clone)]
pub struct VarDefinition<'ast, 's> {
    pub ty: ast::TypeDefine,
    #[cfg(feature = "parser")]
    pub raw_define: &'ast parse::VarDefine<'s>,
    _p: PhantomData<&'ast &'s ()>,
}

impl<'ast, 's> VarDefinition<'ast, 's> {
    pub fn new(
        ty: ast::TypeDefine,
        #[cfg(feature = "parser")] raw_define: &'ast parse::VarDefine<'s>,
    ) -> Self {
        Self {
            ty,
            #[cfg(feature = "parser")]
            raw_define,
            _p: PhantomData,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TypeDefinitions {
    pub map: HashMap<String, TypeDefinition>,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {}
