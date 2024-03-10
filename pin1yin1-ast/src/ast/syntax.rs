use super::types::TypeDeclare;
use super::*;
use crate::keywords::syntax;

pub struct VariableDefine<'s> {
    pub type_: Token<'s, TypeDeclare<'s>>,
    pub ident: Token<'s, Ident<'s>>,
}

pub struct VariableAssign<'s> {
    pub deng3: Token<'s, syntax::Symbol>,
}

pub struct Statement<'s> {
    pub x: &'s (),
}

pub struct CodeBlocks<'s> {
    pub start: Token<'s, syntax::Symbol>,
    pub stmts: Vec<Statement<'s>>,
    pub end: Token<'s, syntax::Symbol>,
}

pub struct DefineFunction {}
