use pin1yin1_parser::{ParseUnit, Result, Token};

use crate::keywords::{syntax, types};

#[derive(Debug, Clone, Copy)]
pub struct TypeWidthDeclare<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub width: Token<'s, usize>,
}

impl ParseUnit for TypeWidthDeclare<'_> {
    type Target<'t> = TypeWidthDeclare<'t>;

    fn select(_selector: &mut pin1yin1_parser::Selector) {
        types::BasicExtenWord::select(_selector);
        usize::select(_selector);
    }

    fn generate<'s>(selections: &pin1yin1_parser::Selections<'s>) -> Result<'s, Self::Target<'s>> {
        if selections.len() != 2 {
            selections.throw("usage: kuan1 <width>")?;
        }
        let keyword = selections[0].parse::<types::BasicExtenWord>()?;
        let width = selections[1].parse::<usize>()?;
        Ok(TypeWidthDeclare { keyword, width })
    }
}

#[cfg(test)]
mod tests {
    use pin1yin1_parser::Parser;

    use super::*;

    #[test]
    fn test_name() {
        {
            let chars = "kuan1 64".chars().collect::<Vec<_>>();
            let mut parser = Parser::new(&chars);
            dbg!(TypeWidthDeclare::parse(&mut parser)).ok();
        }
    }
}

pub struct TypeSignDeclare<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub signed: bool,
}

impl ParseUnit for TypeSignDeclare<'_> {
    type Target<'t> = TypeSignDeclare<'t>;

    fn select(_selector: &mut pin1yin1_parser::Selector) {
        todo!()
    }

    fn generate<'s>(selections: &pin1yin1_parser::Selections<'s>) -> Result<'s, Self::Target<'s>> {
        todo!()
    }
}
pub struct TypeDeclare<'s> {
    pub array: Option<Token<'s, types::BasicExtenWord>>,
    pub width: Option<Token<'s, TypeWidthDeclare<'s>>>,
    pub sign: Option<Token<'s, TypeSignDeclare<'s>>>,
    // reference, rvr, const, pointer...
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
