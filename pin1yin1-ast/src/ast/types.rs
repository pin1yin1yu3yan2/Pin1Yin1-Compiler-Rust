use super::*;
use crate::{complex_pu, keywords::types};

/// Decorators
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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

        let size = p.parse::<usize>().ok();
        p.finish(TypeConstExtend { keyword, size })
    }
}

/// Decorators
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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

        let size = p.parse::<usize>().ok();
        p.finish(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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
        TypePointerExtend
    }
}

/// Decorators for primitive types
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct TypeDeclare<'s> {
    #[cfg_attr(feature = "ser", serde(rename = "const"))]
    pub const_: Option<Token<'s, TypeConstExtend<'s>>>,
    pub decorators: Vec<Token<'s, TypeDecoratorExtends<'s>>>,
    pub width: Option<Token<'s, TypeWidthExtend<'s>>>,
    pub sign: Option<Token<'s, TypeSignExtend<'s>>>,
    pub real_type: Token<'s, Ident<'s>>,
}

impl ParseUnit for TypeDeclare<'_> {
    type Target<'t> = TypeDeclare<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let const_ = p.parse::<TypeConstExtend>().ok();
        let mut decorators = vec![];
        while let Ok(decorator) = p.parse::<TypeDecoratorExtends>() {
            decorators.push(decorator);
        }
        let width = p.parse::<TypeWidthExtend>().ok();
        let sign = p.parse::<TypeSignExtend>().ok();
        let real_type = p.parse::<Ident>()?;
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
            assert!(p.parse::<TypeDeclare>().is_ok())
        })
    }
}
