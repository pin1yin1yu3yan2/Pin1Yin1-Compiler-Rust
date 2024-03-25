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
    pub overdrives: Vec<FnSign<'ast, 's>>,
    #[cfg(feature = "parser")]
    pub raw_defines: Vec<&'ast parse::FnDefine<'s>>,
    _p: PhantomData<&'ast &'s ()>,
}

impl<'ast, 's> FnDefinition<'ast, 's> {
    pub fn new(
        overdrives: Vec<FnSign<'ast, 's>>,
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
pub struct FnSign<'ast, 's> {
    pub mangle: String,
    pub ty: ast::TypeDefine,
    pub params: Vec<Parameter<'ast, 's>>,
}

#[derive(Debug, Clone)]
pub struct Parameter<'ast, 's> {
    pub name: String,
    #[cfg(feature = "parser")]
    pub var_def: VarDefinition<'ast, 's>,
}

impl<'ast, 's> std::ops::Deref for Parameter<'ast, 's> {
    type Target = VarDefinition<'ast, 's>;

    fn deref(&self) -> &Self::Target {
        &self.var_def
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
