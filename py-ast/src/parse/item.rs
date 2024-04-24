use super::*;
use crate::lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct Comment;

impl ParseUnit for Comment {
    type Target = Comment;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Comment)?;

        loop {
            let str = p.get_chars()?;

            if str.is_empty() {
                return p.throw("comment without end");
            }

            const JIE2: &[char] = &['j', 'i', 'e', '2'];
            if *str == JIE2 {
                return p.finish(Comment {});
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnDefine {
    pub ty: PU<types::TypeDefine>,
    pub name: PU<Ident>,
    pub params: PU<Parameters>,
    pub codes: PU<CodeBlock>,
}

impl ParseUnit for FnDefine {
    type Target = FnDefine;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let ty = p.parse()?;
        let name = p.parse()?;
        let params = p.parse()?;
        let codes = p.parse().apply(MustMatch)?;

        p.finish(Self {
            ty,
            name,
            params,
            codes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub stmts: Vec<PU<Statement>>,
}

impl ParseUnit for CodeBlock {
    type Target = CodeBlock;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Block)?;
        let mut stmts = vec![];
        while let Some(stmt) = p.parse().r#try()? {
            stmts.push(stmt)
        }
        p.match_(Symbol::Jie2)?;
        p.finish(Self { stmts })
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
                        ruo4 can1 jie2 1 da4 0 he2 huo4 jie2 2 xiao3 3 he2 yu3 fei1 fou3 jie2
                        han2 
                            shi4 if ((1>0)||(2<3)&&!false){} else{} jie2
                        jie2 ze2 han2 

                        jie2
                    jie2",
            |p| {
                p.parse::<FnDefine>().unwrap();
            },
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
