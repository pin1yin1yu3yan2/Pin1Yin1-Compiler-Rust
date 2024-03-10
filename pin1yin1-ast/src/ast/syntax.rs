use super::types::TypeDeclare;
use super::*;
use crate::keywords::syntax;

pub struct DefineVariable<'s> {
    type_: TypeDeclare<'s>,
    ident: Ident<'s>,
}

pub struct VariableInitialize {}

pub struct Statement<'s> {
    pub x: &'s (),
}

pub struct CodeBlocks<'s> {
    pub start: Token<'s, syntax::Symbol>,
    pub stmts: Vec<Statement<'s>>,
    pub end: Token<'s, syntax::Symbol>,
}

pub struct DefineFunction {}
