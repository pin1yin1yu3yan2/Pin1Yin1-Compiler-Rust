use py_ir::ops::Operators;

use super::*;
use crate::{complex_pu, lex::syntax::Symbol, ops::OperatorAssociativity};

#[derive(Debug, Clone)]
pub struct CharLiteral {
    pub unparsed: PU<String>,
    pub parsed: char,
}

fn escape(src: &PU<String>, c: char) -> Result<char> {
    Result::Ok(match c {
        '_' => '_',
        't' => '\t',
        'n' => '\n',
        's' => ' ',
        _ => return src.throw(format!("Invalid or unsupported escape character: {}", c)),
    })
}

impl ParseUnit for CharLiteral {
    type Target = CharLiteral;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Char)?;
        let unparsed = p.parse::<String>()?;
        if !(unparsed.len() == 1 || unparsed.len() == 2 && unparsed.starts_with('_')) {
            return unparsed.throw(format!("Invalid CharLiteral {}", *unparsed));
        }
        let parsed = if unparsed.len() == 1 {
            unparsed.as_bytes()[0] as char
        } else {
            escape(&unparsed, unparsed.as_bytes()[1] as _)?
        };

        p.finish(CharLiteral { unparsed, parsed })
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub unparsed: PU<String>,
    pub parsed: String,
}

impl ParseUnit for StringLiteral {
    type Target = StringLiteral;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::String)?;
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

        p.finish(StringLiteral { unparsed, parsed })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral {
    Float { number: f64 },
    Digit { number: usize },
}

impl ParseUnit for NumberLiteral {
    type Target = NumberLiteral;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let number = *p.parse::<usize>()?; // digit
        if p.once(|p| p.match_('.')).is_ok() {
            let decimal = p.parse::<usize>().r#try()?.map(|t| *t).unwrap_or(0);
            let decimal = if decimal == 0 {
                0.0
            } else {
                let decimal = decimal as f64;
                decimal / 10f64.powi(decimal.log10().ceil() as _)
            };
            p.finish(NumberLiteral::Float {
                number: number as f64 + decimal,
            })
        } else {
            p.finish(NumberLiteral::Digit { number })
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnCallArgs {
    pub args: Vec<PU<Expr>>,
    pub semicolons: Vec<Span>,
}

impl std::ops::Deref for FnCallArgs {
    type Target = Vec<PU<Expr>>;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl ParseUnit for FnCallArgs {
    type Target = FnCallArgs;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::FnCallL)?;
        let Some(arg) = p.parse::<Expr>().r#try()? else {
            p.match_(Symbol::FnCallR).apply(MustMatch)?;
            return p.finish(FnCallArgs {
                args: vec![],
                semicolons: vec![],
            });
        };

        let mut args = vec![arg];
        let mut semicolons = vec![];

        while let Some(semicolon) = p.match_(Symbol::Semicolon).r#try()? {
            semicolons.push(semicolon.get_span());
            args.push(p.parse::<Expr>()?);
        }

        p.match_(Symbol::FnCallR).apply(MustMatch)?;
        p.finish(FnCallArgs { args, semicolons })
    }
}

#[derive(Debug, Clone)]
pub struct Initialization {
    pub args: Vec<PU<AtomicExpr>>,
}

impl ParseUnit for Initialization {
    type Target = Initialization;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Block)?;
        let mut args = vec![];
        while let Some(expr) = p.parse::<AtomicExpr>().r#try()? {
            args.push(expr);
        }
        p.match_(Symbol::Jie2).apply(MustMatch)?;
        p.finish(Initialization { args })
    }
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub fn_name: PU<Ident>,
    pub args: PU<FnCallArgs>,
}

impl ParseUnit for FnCall {
    type Target = FnCall;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let args = p.parse::<FnCallArgs>()?;
        let fn_name = p.parse::<Ident>()?;

        p.finish(FnCall { fn_name, args })
    }
}

pub type Variable = Ident;

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: PU<Operators>,
    // using box, or cycle in AtomicExpr
    pub expr: Box<PU<AtomicExpr>>,
}

impl ParseUnit for UnaryExpr {
    type Target = UnaryExpr;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let operator = p.parse::<Operators>()?;
        if operator.associativity() != OperatorAssociativity::Unary {
            return operator.throw("unary expr must start with an unary operator!");
        }
        let expr = Box::new(p.parse::<AtomicExpr>().apply(MustMatch)?);
        p.finish(UnaryExpr { operator, expr })
    }
}

#[derive(Debug, Clone)]
pub struct BracketExpr {
    pub expr: Box<PU<Expr>>,
}

impl ParseUnit for BracketExpr {
    type Target = BracketExpr;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.match_(Symbol::Jie2)?;
        let expr = Box::new(p.parse::<Expr>()?);
        p.match_(Symbol::BracketR).apply(MustMatch)?;

        p.finish(BracketExpr { expr })
    }
}

complex_pu! {
    cpu AtomicExpr {
        CharLiteral,
        StringLiteral,
        NumberLiteral,
        FnCall,
        Variable,
        UnaryExpr,
        Initialization,
        BracketExpr
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Atomic(AtomicExpr),
    Binary(Box<PU<Expr>>, PU<Operators>, Box<PU<Expr>>),
}

pub trait AsExpr {
    fn as_expr(&self) -> Expr;

    // fn ty(&self, state: &mut GLobalScope) -> usize;
}

impl ParseUnit for Expr {
    type Target = Expr;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let mut exprs = vec![p.parse::<AtomicExpr>()?.map::<Expr, _>(Expr::Atomic)];
        let mut ops = vec![];

        let get_binary = |p: &mut Parser| {
            let operator = p.parse::<Operators>()?;
            if operator.associativity() != OperatorAssociativity::Binary {
                operator.throw("atomic exprs must be connected with binary operators!")
            } else {
                Ok(operator)
            }
        };

        while let Some(op) = p.once(get_binary).r#try()? {
            let expr = p.parse::<AtomicExpr>()?.map(Expr::Atomic);

            if ops
                .last()
                .is_some_and(|p: &PU<Operators>| p.priority() <= op.priority())
            {
                let rhs = Box::new(exprs.pop().unwrap());
                let op = ops.pop().unwrap();
                let lhs = Box::new(exprs.pop().unwrap());

                let span = lhs.get_span().merge(rhs.get_span());

                let binary = Expr::Binary(lhs, op, rhs);
                exprs.push(PU::new(span, binary));
            }

            exprs.push(expr);
            ops.push(op);
        }

        while !ops.is_empty() {
            let rhs = Box::new(exprs.pop().unwrap());
            let op = ops.pop().unwrap();
            let lhs = Box::new(exprs.pop().unwrap());

            let span = lhs.get_span().merge(rhs.get_span());

            let binary = Expr::Binary(lhs, op, rhs);
            exprs.push(PU::new(span, binary));
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
        parse_test("ya1 1919810 fen1 chuan4 acminoac ru4 han2shu4", |p| {
            assert!(p.parse::<FnCall>().is_ok());
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
        parse_test("fei1 jie2 114514 he2", |p| {
            p.parse::<UnaryExpr>().unwrap();
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
