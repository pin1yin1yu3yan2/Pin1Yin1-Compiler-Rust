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
        #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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
                let fen1 = p.match_one($crate::keywords::syntax::Symbol::Semicolon, "expect `fen1`")?;
                p.finish($into { inner, fen1 })
            }
        }

        impl<'s> From<$from<'s>> for $into<'s> {
            fn from(v: $from<'s>) -> Self {
                Self {
                    inner: pin1yin1_parser::PU::new_without_selection(v),
                    fen1: $crate::keywords::syntax::defaults::Symbol::Semicolon()
                }
            }
        }

        impl<'s> From<$into<'s>> for $from<'s> {
            fn from(v: $into<'s>) -> Self {
                v.inner.take()
            }
        }

        )*
    };
}

statement_wrapper! {
    #[cfg_attr(feature = "ser", serde(from = "FunctionDefine"))]
    #[cfg_attr(feature = "ser", serde(into = "FunctionDefine"))]
    FunctionDefine => VariableDefineStatement,
    #[cfg_attr(feature = "ser", serde(from = "FunctionCall"))]
    #[cfg_attr(feature = "ser", serde(into = "FunctionCall"))]
    FunctionCall => FunctionCallStatement,
    #[cfg_attr(feature = "ser", serde(from = "VariableInit"))]
    #[cfg_attr(feature = "ser", serde(into = "VariableInit"))]
    VariableInit => VariableInitStatement,
    #[cfg_attr(feature = "ser", serde(from = "VariableReAssign"))]
    #[cfg_attr(feature = "ser", serde(into = "VariableReAssign"))]
    VariableReAssign => VariableReAssignStatement,
}

complex_pu! {
    cpu Statement {
        FunctionCallStatement,
        VariableDefineStatement,
        VariableInitStatement,
        VariableReAssignStatement,
        FunctionDefine,
        CodeBlock,
        If,
        While,
        Return,
        #[cfg_attr(feature = "ser", serde(skip))]
        Comment
    }
}
