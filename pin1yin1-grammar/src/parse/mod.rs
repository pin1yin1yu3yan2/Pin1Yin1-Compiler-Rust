use pin1yin1_parser::*;

mod controlflow;
mod expr;
mod into_ast;
mod statements;
/// we still decide to keep [`Rule`] in [`FnDefine::parse`], [`VarDefine::parse`],[`FnDefine::parse`] and [`VarAssign::parse`],
/// because its a kind of trying: rebuild
///
/// but for better code quality, [`Rule`] may be removed in future
mod syntax;
mod types;

pub use controlflow::*;
pub use expr::*;
pub use into_ast::*;
pub use statements::*;
pub use syntax::*;
pub use types::*;

#[derive(Debug, Clone)]
pub struct Ident(String);

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

        use crate::keywords::*;

        // keeping keywords cant be used as identifiers
        if operators::KEPPING_KEYWORDS.contains(*ident)
            || operators::sub_classes::KEPPING_KEYWORDS.contains(*ident)
            || preprocess::KEPPING_KEYWORDS.contains(*ident)
            || syntax::KEPPING_KEYWORDS.contains(*ident)
            || types::KEPPING_KEYWORDS.contains(*ident)
        {
            return p.unmatch("keeping keywords could not be ident");
        }

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
