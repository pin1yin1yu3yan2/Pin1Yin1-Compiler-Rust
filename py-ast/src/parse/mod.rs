use terl::*;
type Result<T> = terl::Result<T, terl::ParseError>;

mod expr;
mod flow;
mod item;
mod stmt;
mod syntax;
mod types;

pub use expr::*;
pub use flow::*;
pub use item::*;
pub use stmt::*;
pub use syntax::*;
pub use types::*;

#[derive(Debug, Clone)]
pub struct Ident(pub String);

impl std::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl ParseUnit for Ident {
    type Target = Ident;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ident = p.get_chars()?;

        if ident.is_empty() {
            return p.unmatch("empty ident!");
        }
        if ident[0].is_ascii_digit() {
            return p.unmatch("bad ident!");
        }

        use crate::lex::*;
        use py_ir::ops;

        let keeps = &[
            ops::KEPPING_KEYWORDS,
            ops::sub_classes::KEPPING_KEYWORDS,
            preprocess::KEPPING_KEYWORDS,
            syntax::KEPPING_KEYWORDS,
            types::KEPPING_KEYWORDS,
        ];

        for keeps in keeps {
            if keeps.with(|keeps| keeps.contains(*ident)) {
                return p.unmatch("keeping keywords could not be ident");
            }
        }

        // keeping keywords cant be used as identifiers

        let t = Ident(ident.iter().collect());
        p.finish(t)
    }
}

pub fn do_parse(parser: &mut Parser) -> Result<Vec<PU<Statement>>> {
    let mut stmts = vec![];
    while let Some(stmt) = parser.parse::<Statement>().r#try()? {
        stmts.push(stmt);
    }

    Result::Ok(stmts)
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
        fn is_e4chou4de1<P: ParseUnit>(r: ParseResult<P>) -> bool {
            r.is_err()
        }

        parse_test("114514", |p| {
            assert!(is_e4chou4de1(p.parse::<Ident>()));
        })
    }

    #[test]
    fn double_idnet() {
        parse_test("a b", |p| {
            let a = p.parse::<Ident>();
            assert!(a.is_ok());
            assert_eq!(a.unwrap().0, "a");

            let b = p.parse::<Ident>();
            assert!(b.is_ok());
            assert_eq!(b.unwrap().0, "b");
        })
    }
}
