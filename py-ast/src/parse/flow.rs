use super::*;
use py_lex::syntax::{ControlFlow, Symbol};

#[derive(Debug, Clone)]
pub struct Conditions {
    pub conds: Vec<Expr>,
    pub semicolons: Vec<Span>,
}

impl std::ops::Deref for Conditions {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.conds
    }
}

impl ParseUnit<Token> for Conditions {
    type Target = Conditions;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(Symbol::Parameter)?;
        let Some(cond) = p.parse::<Expr>().apply(mapper::Try)? else {
            p.r#match(Symbol::EndOfBlock).apply(mapper::MustMatch)?;
            return Ok(Conditions {
                conds: vec![],
                semicolons: vec![],
            });
        };

        let mut conds = vec![cond];
        let mut semicolons = vec![];

        while let Some(semicolon) = p.r#match(RPU(Symbol::Semicolon)).apply(mapper::Try)? {
            semicolons.push(semicolon.get_span());
            conds.push(p.parse::<Expr>()?);
        }

        p.r#match(Symbol::EndOfBlock).apply(mapper::MustMatch)?;
        Ok(Conditions { conds, semicolons })
    }
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub conds: Conditions,
    pub body: CodeBlock,
}

impl ParseUnit<Token> for IfBranch {
    type Target = IfBranch;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(ControlFlow::If)?;
        let conds = p.parse::<Conditions>()?;
        let body = p.parse::<CodeBlock>().apply(mapper::MustMatch)?;
        Ok(IfBranch { conds, body })
    }
}

#[derive(Debug, Clone)]
pub struct ElseBranch {
    pub block: CodeBlock,
}

impl ParseUnit<Token> for ElseBranch {
    type Target = ElseBranch;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(ControlFlow::Else)?;
        let block = p.parse::<CodeBlock>().apply(mapper::MustMatch)?;
        Ok(ElseBranch { block })
    }
}

#[derive(Debug, Clone)]
pub struct ElseIfBranch {
    pub block: PU<CodeBlock>,
}

impl ParseUnit<Token> for ElseIfBranch {
    type Target = IfBranch;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(ControlFlow::Else)?;
        p.r#match(ControlFlow::If)?;

        let conds = p.parse::<Conditions>()?;
        let body = p.parse::<CodeBlock>().apply(mapper::MustMatch)?;
        Ok(IfBranch { conds, body })
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub branches: Vec<IfBranch>,
    pub else_: Option<ElseBranch>,
}

impl ParseUnit<Token> for If {
    type Target = If;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let mut branches = vec![p.parse::<IfBranch>()?];
        while let Some(chain) = p.parse::<ElseIfBranch>().apply(mapper::Try)? {
            branches.push(chain);
        }
        let else_ = p.parse::<ElseBranch>().apply(mapper::Try)?;
        Ok(If { branches, else_ })
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub conds: Conditions,
    pub block: CodeBlock,
}

impl ParseUnit<Token> for While {
    type Target = While;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(ControlFlow::Repeat)?;
        let conds = p.parse::<Conditions>().apply(mapper::MustMatch)?;
        let block = p.parse::<CodeBlock>().apply(mapper::MustMatch)?;
        Ok(While { conds, block })
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub val: Option<Expr>,
}

impl ParseUnit<Token> for Return {
    type Target = Return;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(ControlFlow::Return)?;
        let val = p.parse::<Expr>().apply(mapper::Try)?;
        p.r#match(Symbol::Semicolon)?;

        Ok(Return { val })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_test;

    #[test]
    fn r#if() {
        let src = "
        ruo4 can1 jie2 1 da4 0 he2 huo4 jie2 2 xiao3 3 he2 yu3 fei1 fou3 jie2 han2
        
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
