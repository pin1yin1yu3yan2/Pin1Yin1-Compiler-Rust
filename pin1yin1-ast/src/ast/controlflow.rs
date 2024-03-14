use self::{expr::Arguments, syntax::CodeBlock};
use crate::{
    complex_pu,
    keywords::syntax::{defaults::ControlFlow::*, ControlFlow},
};

use super::*;

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct AtomicIf<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "If"))]
    pub ruo4: Token<'s, ControlFlow>,
    pub conds: Token<'s, Arguments<'s>>,
    pub block: Token<'s, CodeBlock<'s>>,
}

impl ParseUnit for AtomicIf<'_> {
    type Target<'t> = AtomicIf<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ruo4 = p.parse::<ControlFlow>()?.is(ControlFlow::If)?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse_or::<CodeBlock>("`ruo4` without a code block")?;
        p.finish(AtomicIf { ruo4, conds, block })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct AtomicElse<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Else"))]
    pub ze2: Token<'s, ControlFlow>,
    pub block: Token<'s, CodeBlock<'s>>,
}

impl ParseUnit for AtomicElse<'_> {
    type Target<'t> = AtomicElse<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ze2 = p.parse::<ControlFlow>()?.is(ControlFlow::Else)?;
        let block = p.parse_or::<CodeBlock>("`ze2` without a code block")?;
        p.finish(AtomicElse { ze2, block })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct AtomicElseIf<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Else"))]
    pub ze2: Token<'s, ControlFlow>,
    pub ruo4: Token<'s, AtomicIf<'s>>,
}

impl ParseUnit for AtomicElseIf<'_> {
    type Target<'t> = AtomicElseIf<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ze2 = p.parse::<ControlFlow>()?.is(ControlFlow::Else)?;
        let ruo4 = p.parse::<AtomicIf>()?;
        p.finish(AtomicElseIf { ze2, ruo4 })
    }
}

complex_pu! {
    #[cfg_attr(feature = "ser", serde(untagged))]
    cpu ChainIf {
        AtomicElseIf,
        AtomicElse
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct If<'s> {
    pub ruo4: Token<'s, AtomicIf<'s>>,
    pub chains: Vec<Token<'s, ChainIf<'s>>>,
}

impl ParseUnit for If<'_> {
    type Target<'t> = If<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ruo4 = p.parse::<AtomicIf>()?;
        let mut chains = vec![];
        while let Ok(chain) = p.parse::<ChainIf>() {
            let is_atomic_else = matches!(*chain, ChainIf::AtomicElse(..));
            chains.push(chain);
            if is_atomic_else {
                break;
            }
        }
        p.finish(If { ruo4, chains })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn if_() {
        parse_test(
            "ruo4 can1 can1 1 da4 0 jie2 huo4 can1 2 xiao3 3 jie2 yu3 fei1 fo3 jie2
                    han2 jie2
                    ze2 han2 jie2",
            |p| assert!(p.parse::<If>().is_ok()),
        );
    }
}
