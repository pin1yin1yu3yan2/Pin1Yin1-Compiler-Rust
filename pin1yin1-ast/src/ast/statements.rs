use crate::complex_pu;

use crate::keywords::syntax::Symbol;

#[cfg(feature = "ser")]
use crate::keywords::syntax::defaults::Symbol::*;

use super::controlflow::*;
use super::expr::FunctionCall;
use super::syntax::*;
use pin1yin1_parser::*;

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(transparent))]
#[derive(Debug, Clone)]
pub struct FunctionCallStatement<'s> {
    pub inner: Token<'s, FunctionCall<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Semicolon"))]
    pub fen1: Token<'s, Symbol>,
}

impl ParseUnit for FunctionCallStatement<'_> {
    type Target<'t> = FunctionCallStatement<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let inner = p.parse::<FunctionCall>()?;
        let fen1 = p.match_one(Symbol::Semicolon, "missing `fen1`")?;
        p.finish(FunctionCallStatement { inner, fen1 })
    }
}

#[cfg(feature = "ser")]
impl<'s> From<FunctionCall<'s>> for FunctionCallStatement<'s> {
    fn from(value: FunctionCall<'s>) -> Self {
        Self {
            inner: Token::new_without_selection(value),
            fen1: Semicolon(),
        }
    }
}

#[cfg(feature = "ser")]
impl<'s> From<FunctionCallStatement<'s>> for FunctionCall<'s> {
    fn from(value: FunctionCallStatement<'s>) -> Self {
        value.inner.take()
    }
}

complex_pu! {
    cpu Statement {
        #[cfg_attr(feature = "ser", serde(skip))]
        Comment,
        FunctionCallStatement,
        VariableInit,
        VariableReAssign,
        CodeBlock,
        FunctionDefine,
        If,
        While
    }
}
