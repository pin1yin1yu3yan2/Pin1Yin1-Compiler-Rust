use super::*;
use crate::{complex_pu, keywords::types::BasicExtenWord};

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend<'s> {
    pub keyword: BasicExtenWord,
    _p: PhantomData<&'s ()>,
}

impl ParseUnit for TypeConstExtend<'_> {
    type Target<'t> = TypeConstExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = BasicExtenWord::Const.parse_or_unmatch(p)?.take();
        p.finish(TypeConstExtend {
            keyword,
            _p: PhantomData,
        })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeArrayExtend<'s> {
    pub keyword: PU<'s, BasicExtenWord>,
    pub size: Option<PU<'s, usize>>,
}

impl ParseUnit for TypeArrayExtend<'_> {
    type Target<'t> = TypeArrayExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = BasicExtenWord::Array.parse_or_unmatch(p)?;

        let size = match p.try_parse::<usize>() {
            Some(s) => Some(s?),
            None => None,
        };
        p.finish(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeReferenceExtend<'s> {
    pub keyword: BasicExtenWord,
    _p: PhantomData<&'s ()>,
}

impl ParseUnit for TypeReferenceExtend<'_> {
    type Target<'t> = TypeReferenceExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = BasicExtenWord::Reference.parse_or_unmatch(p)?.take();

        p.finish(TypeReferenceExtend {
            keyword,
            _p: PhantomData,
        })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypePointerExtend<'s> {
    pub keyword: BasicExtenWord,
    _p: PhantomData<&'s ()>,
}

impl ParseUnit for TypePointerExtend<'_> {
    type Target<'t> = TypePointerExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = BasicExtenWord::Pointer.parse_or_unmatch(p)?.take();
        p.finish(TypePointerExtend {
            keyword,
            _p: PhantomData,
        })
    }
}

complex_pu! {
    cpu TypeDecorators {
        TypeArrayExtend,
        TypeReferenceExtend,
        TypePointerExtend
    }
}

/// Decorators for primitive types

#[derive(Debug, Clone, Copy)]
pub struct TypeWidthExtend<'s> {
    pub keyword: PU<'s, BasicExtenWord>,
    pub width: PU<'s, usize>,
}

impl ParseUnit for TypeWidthExtend<'_> {
    type Target<'t> = TypeWidthExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = BasicExtenWord::Width.parse_or_unmatch(p)?;
        let width = p
            .parse::<usize>()
            .map_err(|e| e.unmatch("usage: kaun1 <width> "))?;
        p.finish(TypeWidthExtend { keyword, width })
    }
}

/// Decorators for `zheng3`

#[derive(Debug, Clone, Copy)]
pub struct TypeSignExtend<'s> {
    pub keyword: PU<'s, BasicExtenWord>,
    pub sign: bool,
}

impl ParseUnit for TypeSignExtend<'_> {
    type Target<'t> = TypeSignExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?;
        let sign = match *keyword {
            BasicExtenWord::Signed => true,
            BasicExtenWord::Unsigned => false,
            _ => {
                return keyword.throw("should be `you3fu2` or `wu2fu2`");
            }
        };

        p.finish(TypeSignExtend { keyword, sign })
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefine<'s> {
    pub const_: Option<PU<'s, TypeConstExtend<'s>>>,
    pub decorators: Vec<PU<'s, TypeDecorators<'s>>>,
    pub width: Option<PU<'s, TypeWidthExtend<'s>>>,
    pub sign: Option<PU<'s, TypeSignExtend<'s>>>,
    pub ty: PU<'s, Ident<'s>>,
}

impl ParseUnit for TypeDefine<'_> {
    type Target<'t> = TypeDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let const_ = p.parse::<TypeConstExtend>().success();
        let mut decorators = vec![];
        while let Some(decorator) = p.try_parse::<TypeDecorators>() {
            decorators.push(decorator?);
        }
        let width = p.parse::<TypeWidthExtend>().success();
        let sign = p.parse::<TypeSignExtend>().success();
        let ty = p.parse::<Ident>()?;
        p.finish(TypeDefine {
            const_,
            decorators,
            width,
            sign,
            ty,
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
            assert!(p.parse::<TypeDefine>().is_success())
        })
    }
}
