use py_lex::*;
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
pub struct Ident(Token);

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

impl ParseUnit<Token> for Ident {
    type Target = Ident;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let Some(token) = p.next() else {
            return p.unmatch("expect a `Ident`, but no token left");
        };

        if token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return p.unmatch("bad ident! ident should not start with a digit");
        }

        use py_lex::*;

        let keeps = &[
            ops::KEPPING_KEYWORDS,
            ops::sub_classes::KEPPING_KEYWORDS,
            preprocess::KEPPING_KEYWORDS,
            syntax::KEPPING_KEYWORDS,
            types::KEPPING_KEYWORDS,
        ];

        for keeps in keeps {
            if keeps.with(|keeps| keeps.contains(&**token)) {
                return p.unmatch("keeping keywords could not be ident");
            }
        }

        // keeping keywords cant be used as identifiers
        Ok(Ident(token.clone()))
    }
}

/// use to define a complex parse unit which could be one of its variants
#[macro_export]
macro_rules! complex_pu {
    (
        $(#[$metas:meta])*
        cpu $enum_name:ident {
        $(
            $(#[$v_metas:meta])*
            $variant:ident
        ),*
    }) => {
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub enum $enum_name {
            $(
                $(#[$v_metas])*
                $variant($variant),
            )*
        }

        $(
        impl From<$variant> for $enum_name {
             fn from(v: $variant) -> $enum_name {
                <$enum_name>::$variant(v)
            }
        }
        )*


        impl terl::ParseUnit<py_lex::Token> for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut terl::Parser<py_lex::Token>) -> terl::ParseResult<Self, py_lex::Token>
            {
                terl::Try::<$enum_name, _>::new(p)
                $(
                .or_try::<Self, _>(|p| {
                    p.once_no_try::<$variant ,_>($variant::parse)
                        .map(<$enum_name>::$variant)
                })
                )*
                .finish()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn good_ident() {
        parse_test("*)(&%^&*a(*&^%", |p| {
            assert!(p.parse::<Ident>().is_ok());
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
        parse_test("114514", |p| {
            assert!(p.parse::<Ident>().is_err());
        })
    }

    #[test]
    fn double_idnet() {
        parse_test("a b", |p| {
            let a = p.parse::<Ident>();
            assert!(a.is_ok());
            assert_eq!(&a.unwrap(), "a");

            let b = p.parse::<Ident>();
            assert!(b.is_ok());
            assert_eq!(&b.unwrap(), "b");
        })
    }
}
