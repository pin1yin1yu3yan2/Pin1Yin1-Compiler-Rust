use super::controlflow::*;
use super::expr::*;
use super::syntax::*;

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
            pub inner: pin1yin1_parser::PU<$from>,
            pub fen1: pin1yin1_parser::PU<$crate::keywords::syntax::Symbol>
        }

        impl pin1yin1_parser::ParseUnit for $into {
            type Target = $into;

            fn parse(p: &mut pin1yin1_parser::Parser) -> pin1yin1_parser::ParseResult<Self> {
                use pin1yin1_parser::WithSelection;
                let inner = p.parse::<$from>()?;

                #[cfg(debug_assertions)]
                let or = format!(
                    "expect `fen1` {{{}}}",
                    std::any::type_name_of_val(&Self::parse)
                );
                #[cfg(not(debug_assertions))]
                let or = "expect `fen1`";
                let fen1 = p.parse::<$crate::keywords::syntax::Symbol>()
                    .eq_or($crate::keywords::syntax::Symbol::Semicolon, |t| t.unmatch(or))?;
                p.finish($into { inner, fen1 })
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
                        $variant::parse(p)
                            .map_pu(|pu| <$enum_name>::$variant(Box::new(pu)))
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
