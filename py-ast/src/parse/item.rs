use super::*;
use py_lex::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct Comment;

impl ParseUnit<Token> for Comment {
    type Target = Comment;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(Symbol::Comment)?;

        loop {
            let str = p
                .parse::<Token>()
                .apply(mapper::MapMsg("comment without end"))?;

            if &*str == "jie2" {
                return Ok(Comment {});
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnDefine {
    pub ty: types::TypeDefine,
    pub name: Ident,
    pub params: Parameters,
    pub codes: CodeBlock,
    pub retty_span: Span,
    pub sign_span: Span,
}

impl ParseUnit<Token> for FnDefine {
    type Target = FnDefine;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let ty = p.parse::<PU<types::TypeDefine>>()?;
        let name = p.parse::<Ident>()?;
        let params = p.parse::<PU<Parameters>>()?;
        let codes = p.parse::<CodeBlock>().apply(mapper::MustMatch)?;

        Ok(Self {
            retty_span: ty.get_span(),
            sign_span: ty.get_span().merge(params.get_span()),
            ty: ty.take(),
            name,
            params: params.take(),
            codes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub stmts: Vec<Statement>,
}

impl ParseUnit<Token> for CodeBlock {
    type Target = CodeBlock;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.r#match(Symbol::Block)?;
        let mut stmts = vec![];
        while let Some(stmt) = p.parse::<Statement>().apply(mapper::Try)? {
            stmts.push(stmt)
        }
        p.r#match(Symbol::EndOfBlock)?;
        Ok(Self { stmts })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn code_block() {
        parse_test("han2 jie2", |p| {
            p.parse::<CodeBlock>()?;
            Ok(())
        });
    }

    #[test]
    fn function_define() {
        parse_test(
            "zheng3 zhu3 can1 zheng3 argc fen1 zhi3 zhi3 zi4 argv jie2
                    han2
                    jie2",
            |p| {
                p.parse::<FnDefine>()?;
                Ok(())
            },
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
                p.parse::<FnDefine>()?;
                Ok(())
            },
        )
    }

    #[test]
    fn comment() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa jie2", |p| {
            p.parse::<Comment>()?;
            Ok(())
        });
    }

    #[test]
    #[should_panic]
    fn comment_without_ending() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa ajie2", |p| {
            p.parse::<CodeBlock>()?;
            Ok(())
        });
    }
}
