use super::*;
use crate::{complex_pu, lex::types::BasicExtenWord};

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend {
    pub keyword: BasicExtenWord,
}

impl ParseUnit for TypeConstExtend {
    type Target = TypeConstExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let keyword = p.match_(BasicExtenWord::Const)?.take();

        p.finish(TypeConstExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeArrayExtend {
    pub keyword: PU<BasicExtenWord>,
    pub size: Option<PU<usize>>,
}

impl ParseUnit for TypeArrayExtend {
    type Target = TypeArrayExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let keyword = p.match_(BasicExtenWord::Array)?;
        let size = p.parse::<usize>().r#try()?;
        p.finish(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeReferenceExtend {
    pub keyword: BasicExtenWord,
}

impl ParseUnit for TypeReferenceExtend {
    type Target = TypeReferenceExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let keyword = p.match_(BasicExtenWord::Reference)?.take();
        p.finish(TypeReferenceExtend { keyword })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypePointerExtend {
    pub keyword: BasicExtenWord,
}

impl ParseUnit for TypePointerExtend {
    type Target = TypePointerExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let keyword = p.match_(BasicExtenWord::Pointer)?.take();
        p.finish(TypePointerExtend { keyword })
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
pub struct TypeWidthExtend {
    pub keyword: PU<BasicExtenWord>,
    pub width: PU<usize>,
}

impl ParseUnit for TypeWidthExtend {
    type Target = TypeWidthExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let keyword = p.match_(BasicExtenWord::Width)?;
        let width = p.parse::<usize>()?;

        p.finish(TypeWidthExtend { keyword, width })
    }
}

/// Decorators for `zheng3`

#[derive(Debug, Clone, Copy)]
pub struct TypeSignExtend {
    pub keyword: PU<BasicExtenWord>,
    pub sign: bool,
}

impl ParseUnit for TypeSignExtend {
    type Target = TypeSignExtend;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
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
pub struct TypeDefine {
    pub const_: Option<PU<TypeConstExtend>>,
    pub decorators: Vec<PU<TypeDecorators>>,
    pub width: Option<PU<TypeWidthExtend>>,
    pub sign: Option<PU<TypeSignExtend>>,
    pub ty: PU<Ident>,
}

impl TypeDefine {
    pub(crate) fn to_ast_ty(&self) -> Result<crate::ir::TypeDefine> {
        self.clone().try_into()
    }
}

impl TryFrom<TypeDefine> for crate::ir::TypeDefine {
    type Error = pin1yin1_parser::Error;

    fn try_from(value: crate::parse::TypeDefine) -> std::result::Result<Self, Self::Error> {
        /*
           int: sign, width
           float: width
        */

        let ty = if &**value.ty == "zheng3" {
            // default to be i64
            let sign = value.sign.map(|pu| pu.sign).unwrap_or(true);
            let sign_char = if sign { 'i' } else { 'u' };
            let width = if let Some(width) = value.width {
                if !width.width.is_power_of_two() || *width.width > 64 {
                    return Err(width.make_error(
                        format!("`zheng3` with width {} is not suppert now", *width.width),
                        ErrorKind::Semantic,
                    ));
                }
                *width.width
            } else {
                64
            };
            value.width.map(|pu| *pu.width).unwrap_or(64);

            format!("{sign_char}{width}")
        } else if &**value.ty == "fu2" {
            // default to be f32
            if let Some(sign) = value.sign {
                return Err(sign.make_error(
                    "`fu2` type cant be decorated with `you3fu2` or `wu2fu2`",
                    ErrorKind::Semantic,
                ));
            }
            let width = if let Some(width) = value.width {
                if *width.width == 32 || *width.width == 64 {
                    *width.width
                } else {
                    return Err(width.make_error(
                        format!("`fu2` with width {} is not supperted now", *width.width),
                        ErrorKind::Semantic,
                    ));
                }
            } else {
                32
            };
            format!("f{width}")
        } else {
            if let Some(sign) = value.sign {
                return Err(sign.make_error(
                    format!(
                        "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                        *value.ty
                    ),
                    ErrorKind::Semantic,
                ));
            }
            if let Some(width) = value.width {
                return Err(width.make_error(
                    format!(
                        "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                        *value.ty
                    ),
                    ErrorKind::Semantic,
                ));
            }
            value.ty.take().0
        };

        if value.const_.is_none() && value.decorators.is_empty() {
            return Ok(Self::no_decorators(ty));
        }

        let mut decorators = vec![];
        if value.const_.is_some() {
            decorators.push(crate::ir::TypeDecorators::Const);
        }

        for decorator in value.decorators {
            let decorator = match decorator.take() {
                crate::parse::TypeDecorators::TypeArrayExtend(array) => match array.size {
                    Some(size) => crate::ir::TypeDecorators::SizedArray(size.take()),
                    None => crate::ir::TypeDecorators::Array,
                },
                crate::parse::TypeDecorators::TypeReferenceExtend(_) => {
                    crate::ir::TypeDecorators::Reference
                }
                crate::parse::TypeDecorators::TypePointerExtend(_) => {
                    crate::ir::TypeDecorators::Pointer
                }
            };
            decorators.push(decorator);
        }

        Ok(Self {
            decorators: decorators.into(),
            ty,
        })
    }
}

impl ParseUnit for TypeDefine {
    type Target = TypeDefine;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let const_ = p.parse::<TypeConstExtend>().r#try()?;
        let mut decorators = vec![];
        while let Some(decorator) = p.parse::<TypeDecorators>().r#try()? {
            decorators.push(decorator);
        }
        let width = p.parse::<TypeWidthExtend>().r#try()?;
        let sign = p.parse::<TypeSignExtend>().r#try()?;
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
            assert!(p.parse::<TypeDefine>().is_ok())
        })
    }
}
