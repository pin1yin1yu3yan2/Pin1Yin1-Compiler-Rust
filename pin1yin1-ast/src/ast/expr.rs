use crate::keywords::syntax;

use super::*;

#[derive(Debug, Clone)]
pub struct CharLiteral<'s> {
    pub zi4: Token<'s, syntax::Symbol>,
    pub unparsed: Token<'s, String>,
    pub parsed: char,
}

impl ParseUnit for CharLiteral<'_> {
    type Target<'t> = CharLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let zi4 = p.parse::<syntax::Symbol>()?;
        if *zi4 != syntax::Symbol::Char {
            return Err(None);
        }
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral<'s> {
    pub chuan4: Token<'s, syntax::Symbol>,
    pub unparsed: Token<'s, String>,
    pub parsed: String,
}

// pub enum Literal<'s> {}

// pub struct Expr<'s> {}
