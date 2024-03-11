use super::*;
use crate::{complex_pu, keywords::types};

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub size: Option<Token<'s, usize>>,
}

impl ParseUnit for TypeConstExtend<'_> {
    type Target<'t> = TypeConstExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p
            .parse::<types::BasicExtenWord>()?
            .is(types::BasicExtenWord::Const)?;

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
        let keyword = p
            .parse::<types::BasicExtenWord>()?
            .is(types::BasicExtenWord::Array)?;

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
        let keyword = p
            .parse::<types::BasicExtenWord>()?
            .is(types::BasicExtenWord::Reference)?;

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
        let keyword = p
            .parse::<types::BasicExtenWord>()?
            .is(types::BasicExtenWord::RightReference)?;

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
        let keyword = p
            .parse::<types::BasicExtenWord>()?
            .is(types::BasicExtenWord::Pointer)?;

        p.finish(TypePointerExtend { keyword })
    }
}

complex_pu! {
    cpu TypeDecoratorExtends {
        TypeArrayExtend,
        TypeReferenceExtend,
        TypeRightReferenceExtend,
        TypePointerExtend
    }
}

/// Decorators for primitive types
#[derive(Debug, Clone, Copy)]
pub struct TypeWidthExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub width: Token<'s, usize>,
}

impl ParseUnit for TypeWidthExtend<'_> {
    type Target<'t> = TypeWidthExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        if *keyword != types::BasicExtenWord::Width {
            return Err(None);
        }
        let width = p
            .parse::<usize>()
            .map_err(|_| Some(p.new_error("usage: kaun1 <width> ")))?;
        p.finish(TypeWidthExtend { keyword, width })
    }
}

/// Decorators for `zheng3`
#[derive(Debug, Clone, Copy)]
pub struct TypeSignExtend<'s> {
    pub keyword: Token<'s, types::BasicExtenWord>,
    pub sign: bool,
}

impl ParseUnit for TypeSignExtend<'_> {
    type Target<'t> = TypeSignExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<types::BasicExtenWord>()?;
        let sign = match *keyword {
            types::BasicExtenWord::Signed => true,
            types::BasicExtenWord::Unsigned => false,
            _ => {
                return keyword.throw::<Self>("should be `you3fu2` or `wu2fu2`");
            }
        };

        p.finish(TypeSignExtend { keyword, sign })
    }
}

#[derive(Debug)]
pub struct TypeDeclare<'s> {
    pub const_: Option<Token<'s, TypeConstExtend<'s>>>,
    pub decorators: Vec<Token<'s, TypeDecoratorExtends<'s>>>,
    pub width: Option<Token<'s, TypeWidthExtend<'s>>>,
    pub sign: Option<Token<'s, TypeSignExtend<'s>>>,
    pub real_type: Token<'s, Ident>,
}

impl ParseUnit for TypeDeclare<'_> {
    type Target<'t> = TypeDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let const_ = p.try_parse::<TypeConstExtend>().ok();
        let mut decorators = vec![];
        while let Ok(decorator) = p.try_parse::<TypeDecoratorExtends>() {
            decorators.push(decorator);
        }
        let width = p.try_parse::<TypeWidthExtend>().ok();
        let sign = p.try_parse::<TypeSignExtend>().ok();
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

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn fucking_type() {
        parse_test("yin3 zu3 114514 kuan1 32 wu2fu2 zheng3", |p| {
            assert!(dbg!(p.parse::<TypeDeclare>()).is_ok())
        })
    }
}
