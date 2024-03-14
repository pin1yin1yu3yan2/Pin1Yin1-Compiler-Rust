use super::*;
use crate::{complex_pu, keywords::types::BasicExtenWord};

#[cfg(feature = "ser")]
use crate::keywords::types::defaults::BasicExtenWord::*;
/// Decorators
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(from = "bool"))]
#[cfg_attr(feature = "ser", serde(into = "bool"))]
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend<'s> {
    pub keyword: Token<'s, BasicExtenWord>,
}

#[cfg(feature = "ser")]
impl From<bool> for TypeConstExtend<'_> {
    fn from(_: bool) -> Self {
        Self { keyword: Const() }
    }
}

#[cfg(feature = "ser")]
impl From<TypeConstExtend<'_>> for bool {
    fn from(_: TypeConstExtend<'_>) -> Self {
        true
    }
}

impl ParseUnit for TypeConstExtend<'_> {
    type Target<'t> = TypeConstExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?.is(BasicExtenWord::Const)?;
        p.finish(TypeConstExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeArrayExtend<'s> {
    pub keyword: Token<'s, BasicExtenWord>,
    pub size: Option<Token<'s, usize>>,
}

impl ParseUnit for TypeArrayExtend<'_> {
    type Target<'t> = TypeArrayExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?.is(BasicExtenWord::Array)?;

        let size = p.parse::<usize>().ok();
        p.finish(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeReferenceExtend<'s> {
    pub keyword: Token<'s, BasicExtenWord>,
}

impl ParseUnit for TypeReferenceExtend<'_> {
    type Target<'t> = TypeReferenceExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?.is(BasicExtenWord::Reference)?;

        p.finish(TypeReferenceExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypePointerExtend<'s> {
    pub keyword: Token<'s, BasicExtenWord>,
}

impl ParseUnit for TypePointerExtend<'_> {
    type Target<'t> = TypePointerExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?.is(BasicExtenWord::Pointer)?;

        p.finish(TypePointerExtend { keyword })
    }
}

#[cfg(feature = "ser")]
mod serde_ {

    use pin1yin1_parser::Token;

    pub enum TypeDecoratorExtends {
        Array,
        Reference,
        Pointer,
        SizedArray(usize),
    }

    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = TypeDecoratorExtends;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Array, Reference, Pointer or a digit")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                "Array" => Ok(TypeDecoratorExtends::Array),
                "Reference" => Ok(TypeDecoratorExtends::Reference),
                "Pointer" => Ok(TypeDecoratorExtends::Pointer),
                _ => v
                    .parse::<usize>()
                    .map(TypeDecoratorExtends::SizedArray)
                    .map_err(E::custom),
            }
        }
    }

    impl serde::Serialize for TypeDecoratorExtends {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                TypeDecoratorExtends::Array => serializer.serialize_str("Array"),
                TypeDecoratorExtends::Reference => serializer.serialize_str("Reference"),
                TypeDecoratorExtends::Pointer => serializer.serialize_str("Pointer"),
                TypeDecoratorExtends::SizedArray(v) => serializer.serialize_str(&format!("{v}")),
            }
        }
    }

    impl<'de> serde::Deserialize<'de> for TypeDecoratorExtends {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(Visitor)
        }
    }

    impl From<super::TypeDecoratorExtends<'_>> for TypeDecoratorExtends {
        fn from(value: super::TypeDecoratorExtends) -> Self {
            match value {
                super::TypeDecoratorExtends::TypeArrayExtend(a) => match a.size {
                    Some(size) => Self::SizedArray(*size),
                    None => Self::Array,
                },
                super::TypeDecoratorExtends::TypeReferenceExtend(_) => Self::Reference,
                super::TypeDecoratorExtends::TypePointerExtend(_) => Self::Pointer,
            }
        }
    }

    impl<'s> From<TypeDecoratorExtends> for super::TypeDecoratorExtends<'s> {
        fn from(value: TypeDecoratorExtends) -> Self {
            use crate::keywords::types::defaults::BasicExtenWord::{Array, Pointer, Reference};
            match value {
                TypeDecoratorExtends::Array => {
                    super::TypeDecoratorExtends::TypeArrayExtend(super::TypeArrayExtend {
                        keyword: Array(),
                        size: None,
                    })
                }
                TypeDecoratorExtends::SizedArray(size) => {
                    super::TypeDecoratorExtends::TypeArrayExtend(super::TypeArrayExtend {
                        keyword: Array(),
                        size: Some(Token::new_without_selection(size)),
                    })
                }
                TypeDecoratorExtends::Reference => {
                    super::TypeDecoratorExtends::TypeReferenceExtend(super::TypeReferenceExtend {
                        keyword: Reference(),
                    })
                }
                TypeDecoratorExtends::Pointer => {
                    super::TypeDecoratorExtends::TypePointerExtend(super::TypePointerExtend {
                        keyword: Pointer(),
                    })
                }
            }
        }
    }
}

complex_pu! {
    #[cfg_attr(feature = "ser", serde(from = "serde_::TypeDecoratorExtends"))]
    #[cfg_attr(feature = "ser", serde(into = "serde_::TypeDecoratorExtends"))]
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
    pub keyword: Token<'s, BasicExtenWord>,
    pub width: Token<'s, usize>,
}

impl ParseUnit for TypeWidthExtend<'_> {
    type Target<'t> = TypeWidthExtend<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let keyword = p.parse::<BasicExtenWord>()?;
        if *keyword != BasicExtenWord::Width {
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
    pub keyword: Token<'s, BasicExtenWord>,
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
    #[cfg_attr(feature = "ser", serde(default))]
    #[cfg_attr(feature = "ser", serde(skip_serializing_if = "Option::is_none"))]
    pub const_: Option<Token<'s, TypeConstExtend<'s>>>,
    #[cfg_attr(feature = "ser", serde(default))]
    #[cfg_attr(feature = "ser", serde(skip_serializing_if = "Vec::is_empty"))]
    pub decorators: Vec<Token<'s, TypeDecoratorExtends<'s>>>,
    #[cfg_attr(feature = "ser", serde(default))]
    #[cfg_attr(feature = "ser", serde(skip_serializing_if = "Option::is_none"))]
    pub width: Option<Token<'s, TypeWidthExtend<'s>>>,
    #[cfg_attr(feature = "ser", serde(default))]
    #[cfg_attr(feature = "ser", serde(skip_serializing_if = "Option::is_none"))]
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
