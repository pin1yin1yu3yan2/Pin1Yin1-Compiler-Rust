use super::*;
use crate::keywords::{syntax, types};

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub size: Option<Token<'s, usize>>,
}

impl ParseUnit for TypeConstExtend<'_> {
    type Target<'t> = TypeConstExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Const {
            return Err(None);
        }

        let size = p.try_parse::<usize>().ok();
        p.finish(TypeConstExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeArrayExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub size: Option<Token<'s, usize>>,
}

impl ParseUnit for TypeArrayExtend<'_> {
    type Target<'t> = TypeArrayExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Array {
            return Err(None);
        }

        let size = p.try_parse::<usize>().ok();
        p.finish(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeReferenceExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
}

impl ParseUnit for TypeReferenceExtend<'_> {
    type Target<'t> = TypeReferenceExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Reference {
            return Err(None);
        }
        p.finish(TypeReferenceExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeRightReferenceExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
}

impl ParseUnit for TypeRightReferenceExtend<'_> {
    type Target<'t> = TypeRightReferenceExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::RightReference {
            return Err(None);
        }
        p.finish(TypeRightReferenceExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypePointerExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
}

impl ParseUnit for TypePointerExtend<'_> {
    type Target<'t> = TypePointerExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Pointer {
            return Err(None);
        }
        p.finish(TypePointerExtend { keyword })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeDecoratorExtends<'s> {
    Array(TypeArrayExtend<'s>),
    Reference(TypeReferenceExtend<'s>),
    RightReference(TypeRightReferenceExtend<'s>),
    Pointer(TypePointerExtend<'s>),
}

impl ParseUnit for TypeDecoratorExtends<'_> {
    type Target<'t> = TypeDecoratorExtends<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        p.r#try(|p| {
            p.parse::<TypeArrayExtend>()
                .map(|tae| tae.map(TypeDecoratorExtends::Array))
        })
        .or_try(|p| {
            p.parse::<TypeReferenceExtend>()
                .map(|tae| tae.map::<Self, _>(TypeDecoratorExtends::Reference))
        })
        .or_try(|p| {
            p.parse::<TypeRightReferenceExtend>()
                .map(|tae| tae.map::<Self, _>(TypeDecoratorExtends::RightReference))
        })
        .or_try(|p| {
            p.parse::<TypePointerExtend>()
                .map(|tae| tae.map::<Self, _>(TypeDecoratorExtends::Pointer))
        })
        .no_error()
        .finish()
    }
}

/// Decorators for primitive types
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

/// Decorators for `zheng3`
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct TypeDeclare<'s> {
    pub const_: Option<Token<'s, TypeConstExtend<'s>>>,
    pub decorators: Vec<Token<'s, TypeDecoratorExtends<'s>>>,
    pub width: Option<Token<'s, TypeWidthDeclare<'s>>>,
    pub sign: Option<Token<'s, TypeSignDeclare<'s>>>,
    pub real_type: Token<'s, Ident<'s>>,
}

impl ParseUnit for TypeDeclare<'_> {
    type Target<'t> = TypeDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let const_ = p.try_parse::<TypeConstExtend>().ok();
        let mut decorators = vec![];
        while let Ok(decorator) = p.try_parse::<TypeDecoratorExtends>() {
            decorators.push(decorator);
        }
        let width = p.try_parse::<TypeWidthDeclare>().ok();
        let sign = p.try_parse::<TypeSignDeclare>().ok();
        let real_type = p.try_parse::<Ident>()?;
        p.finish(TypeDeclare {
            const_,
            decorators,
            width,
            sign,
            real_type,
        })
    }
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

#[test]
fn fucking_test() {
    let fucking_type = "yin3 zu3 114514 kuan1 32 wu2fu2 zheng3"
        .chars()
        .collect::<Vec<_>>();
    let mut parser = Parser::new(&fucking_type);

    let fucking_decl = parser.parse::<TypeDeclare>().unwrap();
    dbg!(&fucking_decl);
}
