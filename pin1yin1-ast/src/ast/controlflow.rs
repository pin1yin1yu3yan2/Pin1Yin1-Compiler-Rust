use self::{
    expr::{Arguments, Expr},
    syntax::CodeBlock,
};
use super::*;
use crate::{
    complex_pu,
    keywords::syntax::{ControlFlow, Symbol},
};

#[cfg(feature = "ser")]
use crate::keywords::syntax::defaults::ControlFlow::*;
#[cfg(feature = "ser")]
use crate::keywords::syntax::defaults::Symbol::*;

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

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct While<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Repeat"))]
    pub chong2: Token<'s, ControlFlow>,
    pub conds: Token<'s, Arguments<'s>>,
    pub block: Token<'s, CodeBlock<'s>>,
}

impl ParseUnit for While<'_> {
    type Target<'t> = While<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let chong2 = p.parse::<ControlFlow>()?.is(ControlFlow::Repeat)?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse_or::<CodeBlock>("`chong2` without a code block")?;
        p.finish(While {
            chong2,
            conds,
            block,
        })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct Return<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Return"))]
    pub fan3: Token<'s, ControlFlow>,
    pub val: Token<'s, Expr<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Semicolon"))]
    pub fen1: Token<'s, Symbol>,
}

impl ParseUnit for Return<'_> {
    type Target<'t> = Return<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let fan3 = p.parse::<ControlFlow>()?.is(ControlFlow::Return)?;
        let val = p.parse::<Expr>()?;
        let fen1 = p.match_one(Symbol::Semicolon, "expect `fen1`")?;
        p.finish(Return { fan3, val, fen1 })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn r#if() {
        let src = "
        ruo4 can1 can1 1 da4 0 jie2 huo4 can1 2 xiao3 3 jie2 yu3 fei1 fo3 jie2 han2
        
        jie2 ze2 han2
        
        jie2";

        parse_test(src, |p| assert!(p.parse::<If>().is_ok()));
    }

    #[test]
    fn r#while() {
        let src = "
        chong2 can1 i xiao3 5 jie2 han2 
            i deng3 i jia1 1 fen1 
        jie2";
        parse_test(src, |p| assert!(p.parse::<While>().is_ok()));
    }
}
