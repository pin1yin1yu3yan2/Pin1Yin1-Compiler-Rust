use super::*;
use crate::lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct Comment {
    pub shi4: PU<Symbol>,
    pub jie2: PU<Symbol>,
}

impl ParseUnit for Comment {
    type Target = Comment;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let shi4 =
            Equal::new(Symbol::Comment, |e| e.unmatch("not `shi4`")).mapper(Symbol::parse(p))?;
        loop {
            let str = p.get_chars()?;

            if str.is_empty() {
                return p.throw("comment without end");
            }

            const JIE2: &[char] = &['j', 'i', 'e', '2'];
            if *str == JIE2 {
                let jie2 = str.map(|_| Symbol::EndOfBracket);
                return p.finish(Comment { shi4, jie2 });
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnDefine {
    pub ty: PU<types::TypeDefine>,
    pub name: PU<Ident>,
    pub can1: PU<Symbol>,
    pub params: PU<Parameters>,
    pub jie2: PU<Symbol>,
    pub codes: PU<CodeBlock>,
}

impl ParseUnit for FnDefine {
    type Target = FnDefine;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ty = p.parse()?;
        let name = p.parse()?;
        let can1 = p.match_(Symbol::Parameter)?;
        let params = p.parse()?;
        let jie2 = p.match_(Symbol::EndOfBracket).apply(MustMatch)?;
        let codes = p.parse().apply(MustMatch)?;

        p.finish(Self {
            ty,
            name,
            can1,
            params,
            jie2,
            codes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub han2: PU<Symbol>,
    pub stmts: Vec<PU<Statement>>,
    pub jie2: PU<Symbol>,
}

impl ParseUnit for CodeBlock {
    type Target = CodeBlock;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let han2 = p.match_(Symbol::Block)?;
        let mut stmts = vec![];
        while let Some(stmt) = p.parse().r#try()? {
            stmts.push(stmt)
        }
        let jie2 = p.match_(Symbol::EndOfBracket)?;
        p.finish(Self { han2, stmts, jie2 })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn code_block() {
        parse_test("han2 jie2", |p| assert!(p.parse::<CodeBlock>().is_ok()));
    }

    #[test]
    fn function_define() {
        parse_test(
            "zheng3 zhu3 can1 zheng3 argc fen1 zhi3 zhi3 zi4 argv jie2
                    han2
                    jie2",
            |p| assert!((p.parse::<FnDefine>()).is_ok()),
        );
    }

    #[test]
    fn complex_funcion_define() {
        parse_test(
            "zheng3 zhu3 can1 zheng3 argc fen1 zhi3 zu3 zi4 argv jie2 
                    han2
                        ruo4 can1 can1 1 da4 0 jie2 huo4 can1 2 xiao3 3 jie2 yu3 fei1 fou3 jie2
                        han2 
                            shi4 if ((1>0)||(2<3)&&!false){} else{} jie2
                        jie2 ze2 han2 

                        jie2
                    jie2",
            |p| assert!(p.parse::<FnDefine>().is_ok()),
        )
    }

    #[test]
    fn comment() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa jie2", |p| {
            assert!(p.parse::<Comment>().is_ok())
        });
    }

    #[test]
    fn comment_without_ending() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa ajie2", |p| {
            assert!(p.parse::<Comment>().is_err())
        });
    }
}
