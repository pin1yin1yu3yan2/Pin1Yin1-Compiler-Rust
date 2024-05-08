use super::*;
use crate::lex::syntax::{ControlFlow, Symbol};

#[derive(Debug, Clone)]
pub struct Conditions {
    pub conds: Vec<PU<Expr>>,
    pub semicolons: Vec<Span>,
}

impl std::ops::Deref for Conditions {
    type Target = Vec<PU<Expr>>;

    fn deref(&self) -> &Self::Target {
        &self.conds
    }
}

impl ParseUnit for Conditions {
    type Target = Conditions;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Parameter)?;
        let Some(arg) = p.parse::<Expr>().r#try()? else {
            p.match_(Symbol::Jie2).apply(MustMatch)?;
            return p.finish(Conditions {
                conds: vec![],
                semicolons: vec![],
            });
        };

        let mut conds = vec![arg];
        let mut semicolons = vec![];

        while let Some(semicolon) = p.match_(Symbol::Semicolon).r#try()? {
            semicolons.push(semicolon.get_span());
            conds.push(p.parse::<Expr>()?);
        }

        p.match_(Symbol::Jie2).apply(MustMatch)?;
        p.finish(Conditions { conds, semicolons })
    }
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub conds: PU<Conditions>,
    pub body: PU<CodeBlock>,
}

impl ParseUnit for IfBranch {
    type Target = IfBranch;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::If)?;
        let conds = p.parse::<Conditions>()?;
        let body = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(IfBranch { conds, body })
    }
}

#[derive(Debug, Clone)]
pub struct ElseBranch {
    pub block: PU<CodeBlock>,
}

impl ParseUnit for ElseBranch {
    type Target = ElseBranch;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Else)?;
        let block = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(ElseBranch { block })
    }
}

#[derive(Debug, Clone)]
pub struct ElseIfBranch {
    pub block: PU<CodeBlock>,
}

impl ParseUnit for ElseIfBranch {
    type Target = IfBranch;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Else)?;
        p.match_(ControlFlow::If)?;
        let conds = p.parse::<Conditions>()?;
        let body = p.parse::<CodeBlock>().apply(MustMatch)?;
        p.finish(IfBranch { conds, body })
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<PU<IfBranch>>,
    pub else_: Option<PU<ElseBranch>>,
}

impl ParseUnit for If {
    type Target = If;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let mut branches = vec![p.parse::<IfBranch>()?];
        while let Some(chain) = p.parse::<ElseIfBranch>().r#try()? {
            branches.push(chain.map(|a| a));
        }
        let else_ = p.parse::<ElseBranch>().r#try()?;
        p.finish(If { branches, else_ })
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub conds: PU<Conditions>,
    pub block: PU<CodeBlock>,
}

impl ParseUnit for While {
    type Target = While;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(ControlFlow::Repeat)?;
        let conds = p.parse::<Conditions>().apply(MustMatch)?;
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
