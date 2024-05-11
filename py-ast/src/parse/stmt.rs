use crate::complex_pu;

use super::*;

/// however, this is the "best" way
macro_rules! statement_wrapper {
    (
        $(
            $(#[$metas:meta])*
            $from:ident => $into:ident,
        )*
    ) => {
        $(
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub struct $into(py_lex::PU<$from>);

        impl terl::ParseUnit<py_lex::Token> for $into {
            type Target = $into;

            fn parse(p: &mut terl::Parser<py_lex::Token>) -> terl::ParseResult<Self, py_lex::Token> {

                let inner = p.parse::<py_lex::PU<$from>>()?;
                p.match_(py_lex::syntax::Symbol::Semicolon).apply(terl::mapper::MustMatch)?;
                Ok($into(inner))
            }
        }

        impl std::ops::Deref for $into {
            type Target =  py_lex::PU<$from>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $into {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        )*
    };
}

statement_wrapper! {
    VarDefine => VarDefineStmt,
    FnCall => FnCallStmt,
    VarStore => VarStoreStmt,
}

/// be different from [`crate::complex_pu`], this version using box to make [`Statement`] enum smaller
macro_rules! statements {
    (
        $(#[$metas:meta])*
        stmt $enum_name:ident {
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
                $variant(Box<$variant>),
            )*
        }

        $(
        impl From<$variant> for $enum_name {
             fn from(v: $variant) -> $enum_name {
                <$enum_name>::$variant(Box::new(v))
            }
        }
        )*

        impl terl::ParseUnit<py_lex::Token> for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut terl::Parser<py_lex::Token>) -> terl::ParseResult<Self, py_lex::Token>
            {
                terl::Try::<$enum_name ,_>::new(p)
                $(
                .or_try::<Self, _>(|p| {
                    p.once_no_try::<$variant ,_>($variant::parse)
                        .map(Box::new).map(<$enum_name>::$variant)
                })
                )*
                .finish()
            }
        }
    };
}

statements! {
    stmt Statement {

        // $name (...)
        FnCallStmt,
        // $name = $expr
        VarStoreStmt,

        // $ty $name
        VarDefineStmt,
        If,
        While,
        Return,
        Comment,
        CodeBlock
    }
}

complex_pu! {
    cpu Item {
        // $ty $name (...)
        FnDefine,
        Comment
    }
}
