use super::*;
use crate::{
    complex_pu,
    keywords::syntax::{ControlFlow, Symbol},
};

#[derive(Debug, Clone)]
pub struct AtomicIf {
    pub ruo4: PU<ControlFlow>,
    pub conds: PU<Arguments>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for AtomicIf {
    type Target = AtomicIf;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ruo4 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::If, |t| t.unmatch("expect `ruo4`"))?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse::<CodeBlock>().must_match()?;
        p.finish(AtomicIf { ruo4, conds, block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElse {
    pub ze2: PU<ControlFlow>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for AtomicElse {
    type Target = AtomicElse;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ze2 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::Else, |t| t.unmatch("expect `else`"))?;
        let block = p.parse::<CodeBlock>().must_match()?;
        p.finish(AtomicElse { ze2, block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElseIf {
    pub ze2: PU<ControlFlow>,
    pub ruo4: PU<AtomicIf>,
}

impl ParseUnit for AtomicElseIf {
    type Target = AtomicElseIf;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
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
pub struct If {
    pub ruo4: PU<AtomicIf>,
    pub chains: Vec<PU<ChainIf>>,
}

impl ParseUnit for If {
    type Target = If;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
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
pub struct While {
    pub chong2: PU<ControlFlow>,
    pub conds: PU<Arguments>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for While {
    type Target = While;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let chong2 = p
            .parse::<ControlFlow>()
            .eq_or(ControlFlow::Repeat, |t| t.unmatch("expect `chong2`"))?;
        let conds = p.parse::<Arguments>().must_match()?;
        let block = p.parse::<CodeBlock>().must_match()?;
        p.finish(While {
            chong2,
            conds,
            block,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub fan3: PU<ControlFlow>,
    pub val: Option<PU<Expr>>,
    pub fen1: PU<Symbol>,
}

impl ParseUnit for Return {
    type Target = Return;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
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
            .eq_or(Symbol::Semicolon, |t| t.unmatch(or))
            .must_match()?;

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
        parse_test(src, |p| assert!(dbg!(p.parse::<While>()).is_success()));
    }
}
