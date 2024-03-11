use super::*;
use crate::keywords::syntax;

#[derive(Debug, Clone)]
pub struct VariableDefine<'s> {
    pub type_: Token<'s, types::TypeDeclare<'s>>,
    pub ident: Token<'s, Ident<'s>>,
}

impl ParseUnit for VariableDefine<'_> {
    type Target<'t> = VariableDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let type_ = p.parse::<types::TypeDeclare>()?;
        let ident = p.parse::<Ident>()?;
        p.finish(VariableDefine { type_, ident })
    }
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
