use std::marker::PhantomData;

use pin1yin1_parser::*;

pub mod controlflow;
pub mod expr;
pub mod statements;
pub mod syntax;
pub mod types;

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct Ident<'s> {
    pub ident: String,
    #[serde(skip)]
    _p: PhantomData<&'s ()>,
}

impl ParseUnit for Ident<'_> {
    type Target<'t> = Ident<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ident = p
            .parse::<&[char]>()?
            .which_or(|s| !s.is_empty(), |s| s.throw("empty ident!"))?
            .which_or(|s| !s[0].is_ascii_digit(), |s| s.throw("bad ident"))?;

        use crate::keywords::*;

        // keeping keywords cant be used as identifiers
        if operators::KEPPING_KEYWORDS.contains(*ident)
            || operators::sub_classes::KEPPING_KEYWORDS.contains(*ident)
            || preprocess::KEPPING_KEYWORDS.contains(*ident)
            || syntax::KEPPING_KEYWORDS.contains(*ident)
            || types::KEPPING_KEYWORDS.contains(*ident)
        {
            return Err(None);
        }

        p.finish(Ident {
            ident: ident.take().iter().collect(),
            _p: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn good_ident() {
        parse_test("*)(&%^&*a(*&^%", |p| {
            assert!((p.parse::<Ident>()).is_ok());
        })
    }

    #[test]
    fn bad_ident() {
        parse_test("1*)(&%^&*a(*&^%", |p| {
            assert!(p.parse::<Ident>().is_err());
        })
    }

    #[test]
    fn e4chou4de1_ident() {
        fn is_e4chou4de1<P: ParseUnit>(r: ParseResult<'_, P>) -> bool {
            r.is_err()
        }

        parse_test("114514", |p| {
            assert!(is_e4chou4de1(p.parse::<Ident>()));
        })
    }
}
