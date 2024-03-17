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
pub struct VariableAssign<'s> {
    pub deng3: PU<'s, Symbol>,
    pub value: PU<'s, Expr<'s>>,
}

impl ParseUnit for VariableAssign<'_> {
    type Target<'t> = VariableAssign<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let deng3 = Symbol::Assign.parse_or_unmatch(p)?;
        let value = p.parse::<Expr>()?;
        p.finish(VariableAssign { deng3, value })
    }
}

#[derive(Debug, Clone)]
pub struct VariableDefine<'s> {
    pub ty: PU<'s, types::TypeDeclare<'s>>,
    pub name: PU<'s, Ident<'s>>,
    pub init: Option<PU<'s, VariableAssign<'s>>>,
}

impl ParseUnit for VariableDefine<'_> {
    type Target<'t> = VariableDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ty = p.parse::<types::TypeDeclare>()?;
        let name = p.parse::<Ident>()?;
        let init = p.parse::<VariableAssign>().success();
        p.finish(VariableDefine { ty, name, init })
    }
}

#[derive(Debug, Clone)]
pub struct VariableStore<'s> {
    pub name: PU<'s, Ident<'s>>,
    pub assign: PU<'s, VariableAssign<'s>>,
}

impl ParseUnit for VariableStore<'_> {
    type Target<'t> = VariableStore<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let name = p.parse::<Ident>()?;
        let assign = p.parse::<VariableAssign>()?;
        p.finish(VariableStore { name, assign })
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
    pub ty: PU<'s, types::TypeDeclare<'s>>,
    pub name: PU<'s, Ident<'s>>,
}

impl ParseUnit for Parameter<'_> {
    type Target<'t> = Parameter<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let ty = p.parse::<types::TypeDeclare>()?;
        let name = p.parse::<Ident>()?;
        p.finish(Parameter { ty, name })
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
pub struct FunctionDefine<'s> {
    pub function: PU<'s, VariableDefine<'s>>,
    pub can1: PU<'s, Symbol>,
    pub params: PU<'s, Parameters<'s>>,
    pub jie2: PU<'s, Symbol>,
    pub codes: PU<'s, CodeBlock<'s>>,
}

impl ParseUnit for FunctionDefine<'_> {
    type Target<'t> = FunctionDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let function = p.parse::<VariableDefine>()?;
        let can1 = Symbol::Parameter.parse_or_unmatch(p)?;
        let params = p.parse::<Parameters>()?;
        let jie2 = Symbol::EndOfBracket.parse_or_failed(p)?;
        let codes = p.parse::<CodeBlock>()?;
        p.finish(FunctionDefine {
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
            assert!(p.parse::<VariableDefine>().is_success())
        })
    }

    #[test]
    fn variable_define_init() {
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(dbg!(p.parse::<Statement>()).is_success())
        });
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(dbg!(p.parse::<VariableDefine>()).is_success())
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_success())
        });
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<VariableStore>().is_success())
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
            |p| assert!((p.parse::<FunctionDefine>()).is_success()),
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
            |p| assert!(p.parse::<FunctionDefine>().is_success()),
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
