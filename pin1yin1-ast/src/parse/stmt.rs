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
        pub struct $into {
            inner: pin1yin1_parser::PU<$from>,
            pub fen1: pin1yin1_parser::PU<$crate::lex::syntax::Symbol>
        }

        impl pin1yin1_parser::ParseUnit for $into {
            type Target = $into;

            fn parse(p: &mut pin1yin1_parser::Parser) -> pin1yin1_parser::ParseResult<Self> {

                let inner = p.parse::<$from>()?;
                let fen1 = p.match_($crate::lex::syntax::Symbol::Semicolon)?;
                p.finish($into { inner, fen1 })
            }
        }

        impl std::ops::Deref for $into {
            type Target =  pin1yin1_parser::PU<$from>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::ops::DerefMut for $into {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
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

        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut pin1yin1_parser::Parser) -> pin1yin1_parser::ParseResult<Self>
            {
                pin1yin1_parser::Try::new(p)
                $(
                .or_try::<Self, _>(|p| {
                    p.once_no_try($variant::parse)
                        .map(|pu| pu.map(|t |<$enum_name>::$variant(Box::new(t))))
                })
                )*
                .finish()
            }
        }
    };
}

statements! {
    cpu Statement {
        // $name (...)
        FnCallStmt,
        // $name = $expr
        VarStoreStmt,
        // $ty $name (...)
        FnDefine,
        // $ty $name
        VarDefineStmt,
        If,
        While,
        Return,
        Comment,
        CodeBlock
    }
}
