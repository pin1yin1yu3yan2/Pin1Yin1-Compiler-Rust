use super::*;
use crate::{
    complex_pu,
    keywords::syntax::{ControlFlow, Symbol},
};

#[derive(Debug, Clone)]
pub struct AtomicIf<'s> {
    pub ruo4: PU<'s, ControlFlow>,
    pub conds: PU<'s, Arguments<'s>>,
    pub block: PU<'s, CodeBlock<'s>>,
}

impl ParseUnit for AtomicIf<'_> {
    type Target<'t> = AtomicIf<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ruo4 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::If, |t| t.unmatch("expect `ruo4`"))?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse::<CodeBlock>()?;
        p.finish(AtomicIf { ruo4, conds, block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElse<'s> {
    pub ze2: PU<'s, ControlFlow>,
    pub block: PU<'s, CodeBlock<'s>>,
}

impl ParseUnit for AtomicElse<'_> {
    type Target<'t> = AtomicElse<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ze2 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::Else, |t| t.unmatch("expect `else`"))?;
        let block = p.parse::<CodeBlock>()?;
        p.finish(AtomicElse { ze2, block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElseIf<'s> {
    pub ze2: PU<'s, ControlFlow>,
    pub ruo4: PU<'s, AtomicIf<'s>>,
}

impl ParseUnit for AtomicElseIf<'_> {
    type Target<'t> = AtomicElseIf<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ze2 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::If, |t| t.unmatch("expect `ruo4`"))?;
        let ruo4 = p.parse::<AtomicIf>()?;
        p.finish(AtomicElseIf { ze2, ruo4 })
    }
}

complex_pu! {

    cpu ChainIf {
        AtomicElseIf,
        AtomicElse
    }
}

#[derive(Debug, Clone)]
pub struct If<'s> {
    pub ruo4: PU<'s, AtomicIf<'s>>,
    pub chains: Vec<PU<'s, ChainIf<'s>>>,
}

impl ParseUnit for If<'_> {
    type Target<'t> = If<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ruo4 = p.parse::<AtomicIf>()?;
        let mut chains = vec![];
        while let Some(chain) = p.try_parse::<ChainIf>() {
            let chain = chain?;
            let is_atomic_else = matches!(*chain, ChainIf::AtomicElse(..));
            chains.push(chain);
            if is_atomic_else {
                break;
            }
        }
        p.finish(If { ruo4, chains })
    }
}

#[derive(Debug, Clone)]
pub struct While<'s> {
    pub chong2: PU<'s, ControlFlow>,
    pub conds: PU<'s, Arguments<'s>>,
    pub block: PU<'s, CodeBlock<'s>>,
}

impl ParseUnit for While<'_> {
    type Target<'t> = While<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let chong2 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::Repeat, |t| t.unmatch("expect `chong2`"))?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse::<CodeBlock>()?;
        p.finish(While {
            chong2,
            conds,
            block,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Return<'s> {
    pub fan3: PU<'s, ControlFlow>,
    pub val: Option<PU<'s, Expr<'s>>>,
    pub fen1: PU<'s, Symbol>,
}

impl ParseUnit for Return<'_> {
    type Target<'t> = Return<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let fan3 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::Return, |t| t.unmatch("expect `fan3`"))?;
        let val = match p.try_parse::<Expr>() {
            Some(val) => Some(val?),
            None => None,
        };

        #[cfg(debug_assertions)]
        let or = format!(
            "expect `fen1` {{{}}}",
            std::any::type_name_of_val(&Self::parse)
        );
        #[cfg(not(debug_assertions))]
        let or = "expect `fen1`";
        let fen1 = p
            .parse::<Symbol>()
            .eq_or(Symbol::Semicolon, |t| t.unmatch(or))?;

        p.finish(Return { fan3, val, fen1 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_test;

    #[test]
    fn r#if() {
        let src = "
        ruo4 can1 can1 1 da4 0 jie2 huo4 can1 2 xiao3 3 jie2 yu3 fei1 fo3 jie2 han2
        
        jie2 ze2 han2
        
        jie2";

        parse_test(src, |p| assert!(dbg!(p.parse::<If>()).is_success()));
    }

    #[test]
    fn r#while() {
        let src = "
        chong2 can1 i xiao3 5 jie2 han2 
            i deng3 i jia1 1 fen1 
        jie2";
        parse_test(src, |p| assert!(p.parse::<While>().is_success()));
    }
}
