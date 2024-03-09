use pin1yin1_parser::*;

use crate::keywords::{syntax, types};

#[derive(Debug, Clone, Copy)]
pub struct TypeArrayDeclare<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub size: Option<Token<'s, usize>>,
}

impl ParseUnit for TypeArrayDeclare<'_> {
    type Target<'t> = TypeArrayDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;

        let size = p.try_parse::<usize>().ok();
        p.finish(TypeArrayDeclare { keyword, size })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeWidthDeclare<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub width: Token<'s, usize>,
}

impl ParseUnit for TypeWidthDeclare<'_> {
    type Target<'t> = TypeWidthDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Width {
            return Err(None);
        }
        let width = p
            .parse::<usize>()
            .map_err(|_| Some(p.gen_error("usage: kaun1 <width> ")))?;
        p.finish(TypeWidthDeclare { keyword, width })
    }
}

pub struct TypeSignDeclare<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub sign: bool,
}

impl ParseUnit for TypeSignDeclare<'_> {
    type Target<'t> = TypeSignDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        let sign = match *keyword {
            types::BasicExtenWord::Signed => true,
            types::BasicExtenWord::Unsigned => false,
            _ => {
                p.throw(std::any::type_name_of_val(&Self::parse))?;
                unreachable!()
            }
        };

        p.finish(TypeSignDeclare { keyword, sign })
    }
}

pub struct Ident<'s> {
    pub ident: Token<'s, String>,
}

impl ParseUnit for Ident<'_> {
    type Target<'t> = Ident<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ident = p.parse::<String>()?;
        let Some(start_char) = ident.chars().next() else {
            p.throw("empty ident!")?;
            unreachable!()
        };

        if !(start_char.is_alphabetic() || start_char == '_') {
            p.throw("bad ident")?;
        }
        p.finish(Ident { ident })
    }
}

pub struct TypeDeclareWithName<'s> {
    pub array: Option<Token<'s, TypeArrayDeclare<'s>>>,
    pub width: Option<Token<'s, TypeWidthDeclare<'s>>>,
    pub sign: Option<Token<'s, TypeSignDeclare<'s>>>,
    pub ty: Token<'s, Ident<'s>>,
    pub name: Token<'s, Ident<'s>>,
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
