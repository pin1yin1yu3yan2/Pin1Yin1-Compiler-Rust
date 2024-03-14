use super::*;
use super::{expr::Expr, statements::Statement};

use crate::keywords::syntax::Symbol;

#[cfg(feature = "ser")]
use crate::keywords::syntax::defaults::Symbol::*;

#[derive(Debug, Clone)]
pub struct Comment<'s> {
    pub shi4: Token<'s, Symbol>,
    pub jie2: Token<'s, Symbol>,
}

impl ParseUnit for Comment<'_> {
    type Target<'t> = Comment<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let shi4 = p.parse::<Symbol>()?.is(Symbol::Comment)?;
        while let Ok(str) = p.parse::<&[char]>() {
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

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct VariableDefine<'s> {
    #[cfg_attr(feature = "ser", serde(rename = "type"))]
    pub type_: Token<'s, types::TypeDeclare<'s>>,
    pub name: Token<'s, Ident<'s>>,
}

impl ParseUnit for VariableDefine<'_> {
    type Target<'t> = VariableDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let type_ = p.parse::<types::TypeDeclare>()?;
        let name = p.parse::<Ident>()?;
        p.finish(VariableDefine { type_, name })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct VariableAssign<'s> {
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Assign"))]
    pub deng3: Token<'s, Symbol>,
    pub value: Token<'s, Expr<'s>>,
}

impl ParseUnit for VariableAssign<'_> {
    type Target<'t> = VariableAssign<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let deng3 = p.parse::<Symbol>()?.is(Symbol::Assign)?;
        let value = p.parse::<Expr>()?;
        p.finish(VariableAssign { deng3, value })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct VariableInit<'s> {
    pub define: Token<'s, VariableDefine<'s>>,
    pub init: Token<'s, VariableAssign<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Semicolon"))]
    pub fen1: Token<'s, Symbol>,
}

impl ParseUnit for VariableInit<'_> {
    type Target<'t> = VariableInit<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let define = p.parse::<VariableDefine>()?;
        let init = p.parse::<VariableAssign>()?;
        let fen1 = p.match_one(Symbol::Semicolon, "missing `fen1`")?;
        p.finish(VariableInit { define, init, fen1 })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct VariableReAssign<'s> {
    pub name: Token<'s, Ident<'s>>,
    pub assign: Token<'s, VariableAssign<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Semicolon"))]
    pub fen1: Token<'s, Symbol>,
}

impl ParseUnit for VariableReAssign<'_> {
    type Target<'t> = VariableReAssign<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let name = p.parse::<Ident>()?;
        let assign = p.parse::<VariableAssign>()?;
        let fen1 = p.match_one(Symbol::Semicolon, "missing `fen1`")?;
        p.finish(VariableReAssign { name, assign, fen1 })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(from = "Vec<Token<'s, Statement<'s>>>"))]
#[cfg_attr(feature = "ser", serde(into = "Vec<Token<'s, Statement<'s>>>"))]
#[derive(Debug, Clone)]
pub struct CodeBlock<'s> {
    pub han2: Token<'s, Symbol>,
    pub stmts: Vec<Token<'s, Statement<'s>>>,
    pub jie2: Token<'s, Symbol>,
}

#[cfg(feature = "ser")]
impl<'s> From<Vec<Token<'s, Statement<'s>>>> for CodeBlock<'s> {
    fn from(value: Vec<Token<'s, Statement<'s>>>) -> Self {
        CodeBlock {
            han2: Block(),
            stmts: value,
            jie2: EndOfBracket(),
        }
    }
}

#[cfg(feature = "ser")]
impl<'s> From<CodeBlock<'s>> for Vec<Token<'s, Statement<'s>>> {
    fn from(value: CodeBlock<'s>) -> Self {
        value.stmts
    }
}

impl ParseUnit for CodeBlock<'_> {
    type Target<'t> = CodeBlock<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let han2 = p.parse::<Symbol>()?.is(Symbol::Block)?;
        let mut stmts = vec![];
        while let Ok(stmt) = p.parse::<Statement>() {
            if !matches!(*stmt, Statement::Comment(..)) {
                stmts.push(stmt);
            }
        }
        let jie2 = p.match_one::<Symbol>(Symbol::EndOfBracket, "expect `jie2` {CodeBlock}")?;
        p.finish(CodeBlock { han2, stmts, jie2 })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct Parameter<'s> {
    #[cfg_attr(feature = "ser", serde(rename = "type"))]
    pub type_: Token<'s, types::TypeDeclare<'s>>,
    pub name: Token<'s, Ident<'s>>,
}

impl ParseUnit for Parameter<'_> {
    type Target<'t> = Parameter<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let type_ = p.parse::<types::TypeDeclare>()?;
        let name = p.parse::<Ident>()?;
        p.finish(Parameter { type_, name })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(from = "Vec<Token<'s, Parameter<'s>>>"))]
#[cfg_attr(feature = "ser", serde(into = "Vec<Token<'s, Parameter<'s>>>"))]
#[derive(Debug, Clone)]
pub struct Parameters<'s> {
    pub params: Vec<Token<'s, Parameter<'s>>>,
    pub semicolons: Vec<Token<'s, Symbol>>,
}

impl<'s> From<Vec<Token<'s, Parameter<'s>>>> for Parameters<'s> {
    fn from(value: Vec<Token<'s, Parameter<'s>>>) -> Self {
        Self {
            params: value,
            semicolons: Vec::new(),
        }
    }
}

impl<'s> From<Parameters<'s>> for Vec<Token<'s, Parameter<'s>>> {
    fn from(value: Parameters<'s>) -> Self {
        value.params
    }
}

impl ParseUnit for Parameters<'_> {
    type Target<'t> = Parameters<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        // may be empty
        let Ok(param) = p.parse::<Parameter>() else {
            return p.finish(Parameters {
                params: vec![],
                semicolons: vec![],
            });
        };

        let mut params = vec![param];
        let mut semicolons = vec![];

        while let Ok(semicolon) = p
            .r#try(|p| p.parse::<Symbol>()?.is(Symbol::Semicolon))
            .finish()
        {
            semicolons.push(semicolon);
            if let Ok(param) = p.parse::<Parameter>() {
                params.push(param)
            } else {
                break;
            }
        }

        p.finish(Parameters { params, semicolons })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct FunctionDefine<'s> {
    pub function: Token<'s, VariableDefine<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Parameter"))]
    pub can1: Token<'s, Symbol>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "EndOfBracket"))]
    pub jie2: Token<'s, Symbol>,
    pub params: Token<'s, Parameters<'s>>,
    pub codes: Token<'s, CodeBlock<'s>>,
}

impl ParseUnit for FunctionDefine<'_> {
    type Target<'t> = FunctionDefine<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let function = p.parse::<VariableDefine>()?;
        let can1 = p.parse::<Symbol>()?.is(Symbol::Parameter)?;
        let params = p.parse::<Parameters>()?;
        let jie2 = p.match_one(Symbol::EndOfBracket, "should be `jie2`")?;
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
            assert!(p.parse::<VariableDefine>().is_ok())
        })
    }

    #[test]
    fn variable_define_init() {
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_ok())
        });
        parse_test("kuan1 32 zheng3 a deng3 114514 fen1", |p| {
            assert!(p.parse::<VariableInit>().is_ok())
        });
    }

    #[test]
    fn variable_reassign() {
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<Statement>().is_ok())
        });
        parse_test("a deng3 114514 fen1", |p| {
            assert!(p.parse::<VariableReAssign>().is_ok())
        });
    }

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
            |p| assert!((p.parse::<FunctionDefine>()).is_ok()),
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
            |p| assert!(p.parse::<FunctionDefine>().is_ok()),
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
