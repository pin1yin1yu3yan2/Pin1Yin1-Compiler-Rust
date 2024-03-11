use pin1yin1_parser::*;

pub mod expr;
pub mod syntax;
pub mod types;

#[derive(Debug, Clone)]
pub struct Ident {
    pub ident: String,
}

impl ParseUnit for Ident {
    type Target<'t> = Ident;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ident = p
            .parse::<String>()?
            .which_or(|s| !s.is_empty(), |s| s.throw("empty ident!"))?
            .which_or(
                |s| !s.chars().next().unwrap().is_ascii_digit(),
                |s| s.throw("bad ident"),
            )?;

        p.finish(Ident {
            ident: ident.take(),
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
