use super::controlflow::*;
use super::expr::*;
use super::syntax::*;
use crate::complex_pu;

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
                let inner = p.parse::<$from>()?;

                #[cfg(debug_assertions)]
                let or = format!(
                    "expect `fen1` {{{}}}",
                    std::any::type_name_of_val(&Self::parse)
                );
                #[cfg(not(debug_assertions))]
                let or = "expect `fen1`";
                let fen1 = p.match_one($crate::keywords::syntax::Symbol::Semicolon, or)?;
                p.finish($into { inner, fen1 })
            }
        }
        )*
    };
}

statement_wrapper! {
    VariableDefine => VariableDefineStatement,
    FunctionCall => FunctionCallStatement,
    VariableInit => VariableInitStatement,
    VariableReAssign => VariableReAssignStatement,
}

complex_pu! {
    cpu Statement {
        // $name (...)
        FunctionCallStatement,
        // $ty $name = $expr
        VariableInitStatement,
        // $name = $expr
        VariableReAssignStatement,
        // $ty $name (...)
        FunctionDefine,
        // $ty $name
        VariableDefineStatement,
        CodeBlock,
        If,
        While,
        Return,

        Comment
    }
}
