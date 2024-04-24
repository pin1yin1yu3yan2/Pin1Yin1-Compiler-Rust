use super::*;
use crate::{
    complex_pu,
    lex::syntax::{ControlFlow, Symbol},
};

#[derive(Debug, Clone)]
pub struct AtomicIf {
    pub conds: PU<Arguments>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for AtomicIf {
    type Target = AtomicIf;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::If)?;
        let conds = p.parse::<Arguments>()?;
        let block = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(AtomicIf { conds, block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElse {
    pub block: PU<CodeBlock>,
}

impl ParseUnit for AtomicElse {
    type Target = AtomicElse;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Else)?;
        let block = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(AtomicElse { block })
    }
}

#[derive(Debug, Clone)]
pub struct AtomicElseIf {
    pub ruo4: PU<AtomicIf>,
}

impl ParseUnit for AtomicElseIf {
    type Target = AtomicElseIf;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::If)?;
        let ruo4 = p.parse::<AtomicIf>()?;
        p.finish(AtomicElseIf { ruo4 })
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
        while let Some(chain) = p.parse::<ChainIf>().r#try()? {
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
    pub conds: PU<Arguments>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for While {
    type Target = While;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Repeat)?;
        let conds = p.parse::<Arguments>().apply(MustMatch)?;
        let block = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(While { conds, block })
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub val: Option<PU<Expr>>,
}

impl ParseUnit for Return {
    type Target = Return;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Return)?;
        let val = p.parse::<Expr>().r#try()?;
        p.match_(Symbol::Semicolon)?;

        p.finish(Return { val })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_test;

    #[test]
    fn r#if() {
        let src = "
        ruo4 can1 jie2 1 da4 0 he2 huo4 jie2 2 xiao3 3 he2 yu3 fei1 fo3 jie2 han2
        
        jie2 ze2 han2
        
        jie2";

        parse_test(src, |p| assert!(p.parse::<If>().is_ok()));
    }

    #[test]
    fn r#while() {
        let src = "
        chong2 can1 i xiao3 5 jie2 han2 
            i wei2 i jia1 1 fen1 
        jie2";
        parse_test(src, |p| assert!(p.parse::<While>().is_ok()));
    }
}
