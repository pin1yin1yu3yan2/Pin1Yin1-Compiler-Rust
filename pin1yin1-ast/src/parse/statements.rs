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
        pub struct $into<'s> {
            pub inner: pin1yin1_parser::PU<'s,$from<'s>>,
            pub fen1: pin1yin1_parser::PU<'s, $crate::keywords::syntax::Symbol>
        }

        impl pin1yin1_parser::ParseUnit for $into<'_> {
            type Target<'t> = $into<'t>;

            fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
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
    VariableDefine => VariableDefineStatement,
    FunctionCall => FunctionCallStatement,
    VariableStore => VariableStoreStatement,
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
        pub enum $enum_name<'s> {
            $(
                $(#[$v_metas])*
                $variant(Box<$variant<'s>>),
            )*
        }

        $(
        impl<'s> From<$variant<'s>> for $enum_name<'s> {
             fn from(v: $variant<'s>) -> $enum_name<'s> {
                <$enum_name>::$variant(Box::new(v))
            }
        }
        )*

        impl pin1yin1_parser::ParseUnit for $enum_name<'_> {
            type Target<'t> = $enum_name<'t>;

            fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self>
            {
                pin1yin1_parser::Try::new(p)
                $(
                     .or_try::<Self, _>(|p| {
                        p.parse::<$variant>()
                            .map(|s| <$enum_name>::$variant(Box::new(s)))
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
        FunctionCallStatement,
        // $name = $expr
        VariableStoreStatement,
        // $ty $name (...)
        FunctionDefine,
        // $ty $name
        VariableDefineStatement,
        If,
        While,
        Return,
        Comment,
        CodeBlock
    }
}
