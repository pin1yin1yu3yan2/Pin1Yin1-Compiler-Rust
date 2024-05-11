use std::ops::Deref;

use py_lex::types::BasicExtenWord;

use super::*;
use crate::complex_pu;

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeConstExtend;

impl ParseUnit<Token> for TypeConstExtend {
    type Target = TypeConstExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(BasicExtenWord::Const)?;

        Ok(TypeConstExtend)
    }
}

#[derive(Debug,Clone, Copy)]
pub struct Size{
    size: PU<usize>
}

impl Deref for Size{
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.size
    }
}

impl WithSpan for Size{
    fn get_span(&self) -> Span {
        self.size.get_span()
    }
}


impl ParseUnit<Token> for Size{
    type Target = Size;

    fn parse(p: &mut Parser<Token>) -> terl::Result<Self::Target, ParseError> {
        let token =p.parse::<Token>()?;
        let size = match token.parse::<usize>(){
                Ok(num) => PU::new(token.get_span(),num),
                Err(pe) => return token.unmatch(format!("expect a number, but got `{}` while parsing",pe)),
            };
        Ok(Self { size })
    }
}
/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeArrayExtend {
    pub keyword: PU<BasicExtenWord>,
    pub size: Option<Size>,
}

impl ParseUnit<Token> for TypeArrayExtend {
    type Target = TypeArrayExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let keyword = p.match_(RPU(BasicExtenWord::Array))?;
        let size = p.parse::<Size>().apply(mapper::Try)?;
        Ok(TypeArrayExtend { keyword, size })
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypeReferenceExtend;

impl ParseUnit<Token> for TypeReferenceExtend {
    type Target = TypeReferenceExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(BasicExtenWord::Reference)?;
        Ok(TypeReferenceExtend)
    }
}

/// Decorators
#[derive(Debug, Clone, Copy)]
pub struct TypePointerExtend;

impl ParseUnit<Token> for TypePointerExtend {
    type Target = TypePointerExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(BasicExtenWord::Pointer)?;
        Ok(TypePointerExtend)
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
    
    pub width: Size,
}

impl ParseUnit<Token> for TypeWidthExtend {
    type Target = TypeWidthExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(BasicExtenWord::Width)?;
        let width = p.parse::<Size>().apply(mapper::MustMatch)?;

        Ok(TypeWidthExtend {  width })
    }
}

/// Decorators for `zheng3`

#[derive(Debug, Clone, Copy)]
pub struct TypeSignExtend {
    pub sign: bool,
}

impl ParseUnit<Token> for TypeSignExtend {
    type Target = TypeSignExtend;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let keyword = p.parse::<PU<BasicExtenWord>>()?;
        let sign = match *keyword {
            BasicExtenWord::Signed => true,
            BasicExtenWord::Unsigned => false,
            _ => {
                return keyword.throw("should be `you3fu2` or `wu2fu2`");
            }
        };

        Ok(TypeSignExtend { sign })
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefine {
    pub const_: Option<PU<TypeConstExtend>>,
    pub decorators: Vec<PU<TypeDecorators>>,
    pub width: Option<PU<TypeWidthExtend>>,
    pub sign: Option<PU<TypeSignExtend>>,
    pub ty: Ident,
}

impl TypeDefine {
    pub(crate) fn to_mir_ty(&self) -> terl::Result<crate::ir::TypeDefine> {
        self.clone().try_into()
    }
}

impl TryFrom<TypeDefine> for crate::ir::TypeDefine {
    type Error = terl::Error;

    fn try_from(def: crate::parse::TypeDefine) -> std::result::Result<Self, Self::Error> {
        use crate::ir::{ComplexType, PrimitiveType};
        /*
           int: sign, width
           float: width
        */

        if &*def.ty == "zheng3" {
            // default to be i64
            let sign = def.sign.map(|pu| pu.sign).unwrap_or(true);

            let width = if let Some(width_extend) = def.width {
                if !width_extend.width.is_power_of_two()
                    || *width_extend.width > 128
                    || *width_extend.width < 64
                {
                    return Err(width_extend.make_error(format!(
                        "`zheng3` with width {} is not suppert now",
                        *width_extend.width
                    )));
                }
                *width_extend.width
            } else {
                64
            };

            use PrimitiveType::*;
            #[rustfmt::skip]
            let ty = match width {
                8   => if sign { I8   } else { U8   },                
                16  => if sign { I16  } else { U16  },                
                32  => if sign { I32  } else { U32  },                
                64  => if sign { I64  } else { U64  },            
                128 => if sign { I128 } else { U128 },
                _ => unreachable!(),
            };
            return Ok(ty.into());
        } else if &*def.ty == "fu2" {
            // default to be f32
            if let Some(sign) = def.sign {
                return Err(
                    sign.make_error("`fu2` type cant be decorated with `you3fu2` or `wu2fu2`")
                );
            }
            let width = if let Some(width) = def.width {
                if *width.width == 32 || *width.width == 64 {
                    *width.width
                } else {
                    return Err(width.make_error(format!(
                        "`fu2` with width {} is not supperted now",
                        *width.width
                    )));
                }
            } else {
                32
            };

            let ty = match width {
                32 => PrimitiveType::F32,
                64 => PrimitiveType::F64,
                _ => unreachable!(),
            };
            return Ok(ty.into());
        }

        if let Some(sign) = def.sign {
            return Err(sign.make_error(format!(
                "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                def.ty
            )));
        }
        if let Some(width) = def.width {
            return Err(width.make_error(format!(
                "type `{}` with `you3fu2` or `wu2fu2` is not supperted now",
                def.ty
            )));
        }
        let ty = (*def.ty.0.string).clone();

        if def.const_.is_none() && def.decorators.is_empty() {
            return Ok(ComplexType::no_decorators(ty).into());
        }

        let mut decorators = vec![];
        if def.const_.is_some() {
            decorators.push(crate::ir::TypeDecorators::Const);
        }

        for decorator in def.decorators {
            let decorator = match decorator.take() {
                crate::parse::TypeDecorators::TypeArrayExtend(array) => match array.size {
                    Some(size) => crate::ir::TypeDecorators::SizedArray(*size),
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

        Ok(ComplexType { decorators, ty }.into())
    }
}

impl ParseUnit<Token> for TypeDefine {
    type Target = TypeDefine;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let const_ = p.parse::<PU<TypeConstExtend>>().apply(mapper::Try)?;
        let mut decorators = vec![];
        while let Some(decorator) = p.parse::<PU<TypeDecorators>>().apply(mapper::Try)? {
            decorators.push(decorator);
        }
        let width = p.parse::<PU<TypeWidthExtend>>().apply(mapper::Try)?;
        let sign = p.parse::<PU<TypeSignExtend>>().apply(mapper::Try)?;
        let ty = p.parse::<Ident>()?;
        Ok(TypeDefine {
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
