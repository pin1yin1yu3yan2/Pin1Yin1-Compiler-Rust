use super::*;

use crate::keywords::syntax::Symbol;

#[derive(Debug, Clone)]
pub struct Comment<'s> {
    pub shi4: PU<'s, Symbol>,
    pub jie2: PU<'s, Symbol>,
}

impl ParseUnit for Comment<'_> {
    type Target<'t> = Comment<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let shi4 = Symbol::Comment.parse_or_unmatch(p)?;
        while let Some(str) = p.try_parse::<&[char]>() {
            let str = str?;
            if str.len() == 0 {
                break;
            }
            const JIE2: &[char] = &['j', 'i', 'e', '2'];
            if *str == JIE2 {
                let jie2 = str.map(|_| Symbol::EndOfBracket);
                return p.finish(Comment { shi4, jie2 });
            }
        }
        p.throw("comment without end")
    }
}

#[derive(Debug, Clone)]
pub struct VarAssign<'s> {
    pub deng3: PU<'s, Symbol>,
    pub value: PU<'s, Expr<'s>>,
}

impl ParseUnit for VarAssign<'_> {
    type Target<'t> = VarAssign<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let deng3 = Symbol::Assign.parse_or_unmatch(p)?;
        let value = p.parse::<Expr>()?;
        p.finish(VarAssign { deng3, value })
    }
}

#[derive(Debug, Clone)]
pub struct VarDefine<'s> {
    pub ty: PU<'s, types::TypeDefine<'s>>,
    pub name: PU<'s, Ident<'s>>,
    pub init: Option<Box<PU<'s, VarAssign<'s>>>>,
}

impl ParseUnit for VarDefine<'_> {
    type Target<'t> = VarDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ty = p.parse::<types::TypeDefine>()?;
        let name = p.parse::<Ident>()?;
        let init = p.parse::<VarAssign>().success().map(Box::new);
        p.finish(VarDefine { ty, name, init })
    }
}

#[derive(Debug, Clone)]
pub struct VarStore<'s> {
    pub name: PU<'s, Ident<'s>>,
    pub assign: PU<'s, VarAssign<'s>>,
}

impl ParseUnit for VarStore<'_> {
    type Target<'t> = VarStore<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let name = p.parse::<Ident>()?;
        let assign = p.parse::<VarAssign>()?;
        p.finish(VarStore { name, assign })
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock<'s> {
    pub han2: PU<'s, Symbol>,
    pub stmts: Vec<PU<'s, Statement<'s>>>,
    pub jie2: PU<'s, Symbol>,
}

impl ParseUnit for CodeBlock<'_> {
    type Target<'t> = CodeBlock<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let han2 = Symbol::Block.parse_or_unmatch(p)?;
        let mut stmts = vec![];
        while let Some(stmt) = p.try_parse::<Statement>() {
            let stmt = stmt?;
            if !matches!(*stmt, Statement::Comment(..)) {
                stmts.push(stmt);
            }
        }

        let jie2 = Symbol::EndOfBracket.parse_or_failed(p)?;
        p.finish(CodeBlock { han2, stmts, jie2 })
    }
}

#[derive(Debug, Clone)]
pub struct Parameter<'s> {
    /// so to make semantic cheaking easier
    pub inner: VarDefine<'s>,
}

impl ParseUnit for Parameter<'_> {
    type Target<'t> = Parameter<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ty = p.parse::<types::TypeDefine>()?;
        let name = p.parse::<Ident>()?;
        let inner = VarDefine {
            ty,
            name,
            init: None,
        };
        p.finish(Parameter { inner })
    }
}

#[derive(Debug, Clone)]
pub struct Parameters<'s> {
    pub params: Vec<PU<'s, Parameter<'s>>>,
    pub semicolons: Vec<PU<'s, Symbol>>,
}

impl<'s> From<Vec<PU<'s, Parameter<'s>>>> for Parameters<'s> {
    fn from(value: Vec<PU<'s, Parameter<'s>>>) -> Self {
        Self {
            params: value,
            semicolons: Vec::new(),
        }
    }
}

impl<'s> From<Parameters<'s>> for Vec<PU<'s, Parameter<'s>>> {
    fn from(value: Parameters<'s>) -> Self {
        value.params
    }
}

impl ParseUnit for Parameters<'_> {
    type Target<'t> = Parameters<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        // may be empty
        let Some(param) = p.try_parse::<Parameter>() else {
            return p.finish(Parameters {
                params: vec![],
                semicolons: vec![],
            });
        };

        let mut params = vec![param?];
        let mut semicolons = vec![];

        while let Some(semicolon) = p.try_once(|p| Symbol::Semicolon.parse_or_unmatch(p)) {
            semicolons.push(semicolon?);
            params.push(p.parse::<Parameter>()?);
        }

        p.finish(Parameters { params, semicolons })
    }
}

#[derive(Debug, Clone)]
pub struct FnDefine<'s> {
    pub function: PU<'s, VarDefine<'s>>,
    pub can1: PU<'s, Symbol>,
    pub params: PU<'s, Parameters<'s>>,
    pub jie2: PU<'s, Symbol>,
    pub codes: PU<'s, CodeBlock<'s>>,
}

impl ParseUnit for FnDefine<'_> {
    type Target<'t> = FnDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let function = p.parse::<VarDefine>()?;
        let can1 = Symbol::Parameter.parse_or_unmatch(p)?;
        let params = p.parse::<Parameters>()?;
        let jie2 = Symbol::EndOfBracket.parse_or_failed(p)?;
        let codes = p.parse::<CodeBlock>().must_match()?;
        p.finish(FnDefine {
            function,
            can1,
            params,
            jie2,
            codes,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_test;

    use super::*;

    #[test]
    fn variable_define() {
        parse_test("kuan1 32 zheng3 a", |p| {
            assert!(p.parse::<VarDefine>().is_success())
        })
    }

    #[test]
    fn variable_define_init() {
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(dbg!(p.parse::<Statement>()).is_success())
        });
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(dbg!(p.parse::<VarDefine>()).is_success())
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_success())
        });
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<VarStore>().is_success())
        });
    }

    #[test]
    fn code_block() {
        parse_test("han2 jie2", |p| {
            assert!(p.parse::<CodeBlock>().is_success())
        });
    }

    #[test]
    fn function_define() {
        parse_test(
            "zheng3 zhu3 can1 zheng3 argc fen1 zhi3 zhi3 zi4 argv jie2
                    han2
                    jie2",
            |p| assert!((p.parse::<FnDefine>()).is_success()),
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
            |p| assert!(p.parse::<FnDefine>().is_success()),
        )
    }

    #[test]
    fn comment() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa jie2", |p| {
            assert!(p.parse::<Comment>().is_success())
        });
    }

    #[test]
    fn comment_without_ending() {
        parse_test("shi4 ehhhaaaaaaaaaaaaaaaaaaaaaaaa ajie2", |p| {
            assert!(p.parse::<Comment>().is_error())
        });
    }
}
