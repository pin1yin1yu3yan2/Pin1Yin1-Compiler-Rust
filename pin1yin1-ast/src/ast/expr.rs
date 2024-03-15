use std::marker::PhantomData;

use super::*;
use crate::{
    complex_pu,
    keywords::{
        operators::{self, OperatorAssociativity},
        syntax::Symbol,
    },
};

#[cfg(feature = "ser")]
use crate::keywords::syntax::defaults::Symbol::*;

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(into = "char"))]
#[cfg_attr(feature = "ser", serde(from = "char"))]
#[derive(Debug, Clone)]
pub struct CharLiteral<'s> {
    pub zi4: PU<'s, Symbol>,
    pub unparsed: PU<'s, String>,
    pub parsed: char,
}

#[cfg(feature = "ser")]
impl From<char> for CharLiteral<'_> {
    fn from(value: char) -> Self {
        Self {
            zi4: Char(),
            unparsed: PU::new_without_selection(String::new()),
            parsed: value,
        }
    }
}
#[cfg(feature = "ser")]
impl From<CharLiteral<'_>> for char {
    fn from(value: CharLiteral<'_>) -> Self {
        value.parsed
    }
}

fn escape<'s>(src: &PU<'s, String>, c: char) -> Result<'s, char> {
    Ok(match c {
        '_' => '_',
        't' => '\t',
        'n' => '\n',
        's' => ' ',
        _ => {
            return Err(Some(src.new_error(format!(
                "Invalid or unsupported escape character: {}",
                c
            ))))
        }
    })
}

impl ParseUnit for CharLiteral<'_> {
    type Target<'t> = CharLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let zi4 = p.parse::<Symbol>()?.is(Symbol::Char)?;
        let unparsed = p.parse::<String>()?.which_or(
            |s| s.len() == 1 || s.len() == 2 && s.starts_with('_'),
            |token| token.throw(format!("Invalid CharLiteral {}", *token)),
        )?;
        let parsed = if unparsed.len() == 1 {
            unparsed.as_bytes()[0] as char
        } else {
            escape(&unparsed, unparsed.as_bytes()[1] as _)?
        };

        p.finish(CharLiteral {
            zi4,
            unparsed,
            parsed,
        })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(into = "String"))]
#[cfg_attr(feature = "ser", serde(from = "String"))]
#[derive(Debug, Clone)]
pub struct StringLiteral<'s> {
    pub chuan4: PU<'s, Symbol>,
    pub unparsed: PU<'s, String>,
    pub parsed: String,
}

#[cfg(feature = "ser")]
impl From<String> for StringLiteral<'_> {
    fn from(value: String) -> Self {
        Self {
            chuan4: String(),
            unparsed: PU::new_without_selection(String::default()),
            parsed: value,
        }
    }
}

#[cfg(feature = "ser")]
impl From<StringLiteral<'_>> for String {
    fn from(value: StringLiteral<'_>) -> Self {
        value.parsed
    }
}

impl ParseUnit for StringLiteral<'_> {
    type Target<'t> = StringLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let chuan4 = p.parse::<Symbol>()?.is(Symbol::String)?;
        let unparsed = p.parse::<String>()?;

        let mut next_escape = false;
        let mut parsed = String::new();
        for c in unparsed.chars() {
            if next_escape {
                next_escape = false;
                parsed.push(escape(&unparsed, c)?);
            } else if c == '_' {
                next_escape = true
            } else {
                parsed.push(c)
            }
        }
        if next_escape {
            return unparsed.throw("Invalid escape! maybe you losted a character");
        }

        p.finish(StringLiteral {
            chuan4,
            unparsed,
            parsed,
        })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral<'s> {
    Float {
        number: f64,
        #[cfg_attr(feature = "ser", serde(skip))]
        _p: PhantomData<&'s ()>,
    },
    Digit {
        number: usize,
        #[cfg_attr(feature = "ser", serde(skip))]
        _p: PhantomData<&'s ()>,
    },
}

impl ParseUnit for NumberLiteral<'_> {
    type Target<'t> = NumberLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let number = *p.parse::<usize>()?;

        if p.parse::<char>().is_ok_and(|c| *c == '.') {
            let decimal = p.parse::<usize>().map(|t| *t).unwrap_or(0) as f64;
            let decimal = decimal / 10f64.powi(decimal.log10().ceil() as _);
            p.finish(NumberLiteral::Float {
                number: number as f64 + decimal,
                _p: PhantomData,
            })
        } else {
            p.finish(NumberLiteral::Digit {
                number,
                _p: PhantomData,
            })
        }
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(from = "Vec<PU<'s, Expr<'s>>>"))]
#[cfg_attr(feature = "ser", serde(into = "Vec<PU<'s, Expr<'s>>>"))]
#[derive(Debug, Clone)]
pub struct Arguments<'s> {
    pub args: Vec<PU<'s, Expr<'s>>>,
    pub semicolons: Vec<PU<'s, Symbol>>,
}

impl<'s> From<Vec<PU<'s, Expr<'s>>>> for Arguments<'s> {
    fn from(value: Vec<PU<'s, Expr<'s>>>) -> Self {
        Arguments {
            args: value,
            semicolons: Vec::new(),
        }
    }
}

impl<'s> From<Arguments<'s>> for Vec<PU<'s, Expr<'s>>> {
    fn from(value: Arguments<'s>) -> Self {
        value.args
    }
}

impl ParseUnit for Arguments<'_> {
    type Target<'t> = Arguments<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let Ok(arg) = p.parse::<Expr>() else {
            return p.finish(Arguments {
                args: vec![],
                semicolons: vec![],
            });
        };

        let mut args = vec![arg];
        let mut semicolons = vec![];

        while let Ok(semicolon) = p
            .r#try(|p| p.parse::<Symbol>()?.is(Symbol::Semicolon))
            .finish()
        {
            semicolons.push(semicolon);
            if let Ok(arg) = p.parse::<Expr>() {
                args.push(arg)
            } else {
                break;
            }
        }

        p.finish(Arguments { args, semicolons })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(from = "Vec<PU<'s, AtomicExpr<'s>>>"))]
#[cfg_attr(feature = "ser", serde(into = "Vec<PU<'s, AtomicExpr<'s>>>"))]
#[derive(Debug, Clone)]
pub struct Initialization<'s> {
    pub han2: PU<'s, Symbol>,
    pub args: Vec<PU<'s, AtomicExpr<'s>>>,
    pub jie2: PU<'s, Symbol>,
}

impl ParseUnit for Initialization<'_> {
    type Target<'t> = Initialization<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let han2 = p.parse::<Symbol>()?.is(Symbol::Block)?;
        let mut args = vec![];
        while let Ok(expr) = p.parse::<AtomicExpr>() {
            args.push(expr);
        }
        let jie2 = p.match_one::<Symbol>(Symbol::EndOfBracket, "invalid Initialization Expr")?;

        p.finish(Initialization { han2, args, jie2 })
    }
}

#[cfg(feature = "ser")]
impl<'s> From<Vec<PU<'s, AtomicExpr<'s>>>> for Initialization<'s> {
    fn from(value: Vec<PU<'s, AtomicExpr<'s>>>) -> Self {
        Self {
            han2: Block(),
            args: value,
            jie2: EndOfBracket(),
        }
    }
}

#[cfg(feature = "ser")]
impl<'s> From<Initialization<'s>> for Vec<PU<'s, AtomicExpr<'s>>> {
    fn from(val: Initialization<'s>) -> Self {
        val.args
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct FunctionCall<'s> {
    pub fn_name: PU<'s, Ident<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "Parameter"))]
    pub han2: PU<'s, Symbol>,
    pub args: PU<'s, Arguments<'s>>,
    #[cfg_attr(feature = "ser", serde(skip))]
    #[cfg_attr(feature = "ser", serde(default = "EndOfBracket"))]
    pub jie2: PU<'s, Symbol>,
}

impl ParseUnit for FunctionCall<'_> {
    type Target<'t> = FunctionCall<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let fn_name = p.parse::<Ident>()?;
        let han2 = p.parse::<Symbol>()?.is(Symbol::Parameter)?;
        let args = p.parse::<Arguments>()?;
        let jie2 = p.match_one(Symbol::EndOfBracket, "should be `jie2`")?;

        p.finish(FunctionCall {
            fn_name,
            han2,
            args,
            jie2,
        })
    }
}

pub type Variable<'s> = Ident<'s>;

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[derive(Debug, Clone)]
pub struct UnaryExpr<'s> {
    pub operator: PU<'s, operators::Operators>,
    // using box, or cycle in AtomicExpr
    pub expr: Box<PU<'s, AtomicExpr<'s>>>,
}

impl ParseUnit for UnaryExpr<'_> {
    type Target<'t> = UnaryExpr<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let operator = p
            .parse::<operators::Operators>()?
            .which(|op| op.associativity() == OperatorAssociativity::Unary)?;
        let expr = Box::new(p.parse::<AtomicExpr>()?);
        p.finish(UnaryExpr { operator, expr })
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(from = "Expr"))]
#[cfg_attr(feature = "ser", serde(into = "Expr"))]
#[derive(Debug, Clone)]
pub struct BracketExpr<'s> {
    pub can1: PU<'s, Symbol>,
    pub expr: Box<PU<'s, Expr<'s>>>,
    pub jie2: PU<'s, Symbol>,
}

#[cfg(feature = "ser")]
impl<'s> From<Expr<'s>> for BracketExpr<'s> {
    fn from(value: Expr<'s>) -> Self {
        Self {
            can1: Parameter(),
            expr: Box::new(PU::new_without_selection(value)),
            jie2: EndOfBracket(),
        }
    }
}

#[cfg(feature = "ser")]
impl<'s> From<BracketExpr<'s>> for Expr<'s> {
    fn from(value: BracketExpr<'s>) -> Self {
        value.expr.take()
    }
}

impl ParseUnit for BracketExpr<'_> {
    type Target<'t> = BracketExpr<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let can1 = p.parse::<Symbol>()?.is(Symbol::Parameter)?;
        let expr = Box::new(p.parse::<Expr>()?);
        let jie2 = p.match_one::<Symbol>(Symbol::EndOfBracket, "expect `jie2` {BracketExpr}")?;

        p.finish(BracketExpr { can1, expr, jie2 })
    }
}

complex_pu! {
    cpu AtomicExpr {
        CharLiteral,
        StringLiteral,
        NumberLiteral,
        Initialization,
        FunctionCall,
        Variable,
        UnaryExpr,
        BracketExpr
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
#[cfg_attr(feature = "ser", serde(untagged))]
#[derive(Debug, Clone)]
pub enum Expr<'s> {
    Atomic(AtomicExpr<'s>),
    Binary(
        Box<PU<'s, Expr<'s>>>,
        PU<'s, operators::Operators>,
        Box<PU<'s, Expr<'s>>>,
    ),
}

impl ParseUnit for Expr<'_> {
    type Target<'t> = Expr<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let mut exprs = vec![p.parse::<AtomicExpr>()?.map::<Expr, _>(Expr::Atomic)];
        let mut ops = vec![];

        while let Ok(op) = p.try_once(|p| {
            p.parse::<operators::Operators>()?
                .which(|op| op.associativity() == OperatorAssociativity::Binary)
        }) {
            let expr = p
                .parse_or::<AtomicExpr>("exprct AtomicExpr")?
                .map(Expr::Atomic);

            if ops
                .last()
                .is_some_and(|p: &PU<'_, operators::Operators>| p.priority() <= op.priority())
            {
                let rhs = Box::new(exprs.pop().unwrap());
                let op = ops.pop().unwrap();
                let lhs = Box::new(exprs.pop().unwrap());

                let selection = lhs.selection().merge(rhs.selection());

                let binary = Expr::Binary(lhs, op, rhs);
                exprs.push(PU::new(selection, binary));
            }

            exprs.push(expr);
            ops.push(op);
        }

        while !ops.is_empty() {
            let rhs = Box::new(exprs.pop().unwrap());
            let op = ops.pop().unwrap();
            let lhs = Box::new(exprs.pop().unwrap());

            let selection = lhs.selection().merge(rhs.selection());

            let binary = Expr::Binary(lhs, op, rhs);
            exprs.push(PU::new(selection, binary));
        }

        // what jb
        p.finish(exprs.pop().unwrap().take())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_test;

    #[test]
    fn char() {
        parse_test("wen2 _t", |p| {
            assert!(p.parse::<CharLiteral>().is_ok());
        });
    }

    #[test]
    fn string() {
        parse_test("chuan4 _t11514___na", |p| {
            assert!(p.parse::<StringLiteral>().is_ok());
        })
    }

    #[test]
    fn number1() {
        parse_test("114514", |p| {
            assert!(p.parse::<NumberLiteral>().is_ok());
        })
    }

    #[test]
    fn number2() {
        parse_test("114514.", |p| {
            assert!(p.parse::<NumberLiteral>().is_ok());
        })
    }

    #[test]
    fn number3() {
        parse_test("1919.810", |p| {
            assert!(p.parse::<NumberLiteral>().is_ok());
        })
    }

    #[test]
    fn initialization() {
        parse_test("han2 1 1 4 5 1 4 jie2", |p| {
            assert!(p.parse::<Initialization>().is_ok());
        })
    }

    #[test]
    fn function_call() {
        parse_test("han2shu41 can1 1919810 fen1 chuan4 acminoac jie2", |p| {
            assert!(p.parse::<FunctionCall>().is_ok());
        })
    }

    #[test]
    fn unary() {
        parse_test("fei1 191810", |p| {
            assert!(p.parse::<UnaryExpr>().is_ok());
        })
    }

    #[test]
    fn nested_unary() {
        parse_test("fei1 fei1 fei1 fei1 191810", |p| {
            assert!(p.parse::<UnaryExpr>().is_ok());
        })
    }

    #[test]
    fn bracket() {
        // unary + bracket
        parse_test("fei1 can1 114514 jie2", |p| {
            assert!(p.parse::<UnaryExpr>().is_ok());
        })
    }

    #[test]
    fn complex_expr() {
        // 119 + 810 * 114514 - 12
        parse_test("1919 jia1 810 cheng2 114514 jian3 12", |p| {
            assert!(p.parse::<Expr>().is_ok());
        });
    }
}
