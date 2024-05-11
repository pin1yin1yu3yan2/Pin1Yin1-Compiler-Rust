use py_lex::{
    ops::{OperatorAssociativity, Operators},
    syntax::*,
};

use super::*;
use crate::complex_pu;

#[derive(Debug, Clone)]
pub struct CharLiteral {
    pub parsed: char,
}

fn escape(src: &Token, c: char) -> Result<char> {
    Result::Ok(match c {
        '_' => '_',
        't' => '\t',
        'n' => '\n',
        's' => ' ',
        _ => return src.throw(format!("Invalid or unsupported escape character: {}", c)),
    })
}

impl ParseUnit<Token> for CharLiteral {
    type Target = CharLiteral;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::Char)?;
        let unparsed = p.parse::<Token>()?;
        if !(unparsed.len() == 1 || unparsed.len() == 2 && unparsed.starts_with('_')) {
            return unparsed.throw(format!("Invalid CharLiteral {}", unparsed));
        }
        let parsed = if unparsed.len() == 1 {
            unparsed.as_bytes()[0] as char
        } else {
            escape(&unparsed, unparsed.as_bytes()[1] as _)?
        };

        Ok(CharLiteral { parsed })
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub parsed: String,
}

impl ParseUnit<Token> for StringLiteral {
    type Target = StringLiteral;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::String)?;
        let unparsed = p.parse::<Token>()?;

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

        Ok(StringLiteral { parsed })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral {
    Float { number: f64 },
    Digit { number: usize },
}

impl ParseUnit<Token> for NumberLiteral {
    type Target = NumberLiteral;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let number = p.parse::<Token>()?; // digit
        if let Ok(number) = number.parse::<usize>() {
            Ok(Self::Digit { number })
        } else {
            match number.parse::<f64>() {
                Ok(number) => Ok(Self::Float { number }),
                Err(fe) => p.unmatch(fe),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnCallArgs {
    pub args: Vec<Expr>,
}

impl std::ops::Deref for FnCallArgs {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl ParseUnit<Token> for FnCallArgs {
    type Target = FnCallArgs;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::FnCallL)?;
        let Some(arg) = p.parse::<Expr>().apply(mapper::Try)? else {
            p.match_(Symbol::FnCallR).apply(mapper::MustMatch)?;
            return Ok(FnCallArgs { args: vec![] });
        };

        let mut args = vec![arg];

        while p.match_(Symbol::Semicolon).is_ok() {
            args.push(p.parse::<Expr>()?);
        }

        dbg!(&args);
        p.match_(Symbol::FnCallR).apply(mapper::MustMatch)?;

        Ok(FnCallArgs { args })
    }
}

#[derive(Debug, Clone)]
pub struct Initialization {
    pub args: Vec<PU<AtomicExpr>>,
}

impl ParseUnit<Token> for Initialization {
    type Target = Initialization;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::Block)?;
        let mut args = vec![];
        while let Some(expr) = p.parse::<PU<AtomicExpr>>().apply(mapper::Try)? {
            args.push(expr);
        }
        p.match_(Symbol::EndOfBlock).apply(mapper::MustMatch)?;
        Ok(Initialization { args })
    }
}

#[derive(Debug, Clone)]
pub struct FnCall {
    span: Span,
    pub fn_name: Ident,
    pub args: FnCallArgs,
}

impl WithSpan for FnCall {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl ParseUnit<Token> for FnCall {
    type Target = FnCall;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let args = p.parse::<FnCallArgs>()?;
        let fn_name = p.parse::<Ident>()?;

        Ok(FnCall {
            fn_name,
            args,
            span: p.get_span(),
        })
    }
}

pub type Variable = Ident;

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: PU<Operators>,
    // using box, or cycle in AtomicExpr
    pub expr: Box<PU<AtomicExpr>>,
}

impl ParseUnit<Token> for UnaryExpr {
    type Target = UnaryExpr;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let operator = p.parse::<PU<Operators>>()?;
        if operator.associativity() != OperatorAssociativity::Unary {
            return operator.throw("unary expr must start with an unary operator!");
        }
        let expr = Box::new(p.parse::<PU<AtomicExpr>>().apply(mapper::MustMatch)?);
        Ok(UnaryExpr { operator, expr })
    }
}

#[derive(Debug, Clone)]
pub struct BracketExpr {
    pub expr: Box<Expr>,
}

impl ParseUnit<Token> for BracketExpr {
    type Target = BracketExpr;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        p.match_(Symbol::BracketL)?;
        let expr = Box::new(p.parse::<Expr>()?);
        p.match_(Symbol::BracketR).apply(mapper::MustMatch)?;

        Ok(BracketExpr { expr })
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
    Atomic(PU<AtomicExpr>),
    Binary(Box<Expr>, PU<Operators>, Box<Expr>),
}

impl Expr {
    #[inline]
    fn get_l_span(&self) -> Span {
        match self {
            Expr::Atomic(l) => l.get_span(),
            Expr::Binary(l, _, _) => l.get_span(),
        }
    }

    #[inline]
    fn get_r_span(&self) -> Span {
        match self {
            Expr::Atomic(r) => r.get_span(),
            Expr::Binary(_, _, r) => r.get_span(),
        }
    }
}

impl WithSpan for Expr {
    #[inline]
    fn get_span(&self) -> Span {
        match self {
            Expr::Atomic(atomic) => atomic.get_span(),
            Expr::Binary(l, _, r) => l.get_l_span().merge(r.get_r_span()),
        }
    }
}

impl ParseUnit<Token> for Expr {
    type Target = Expr;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let mut exprs = vec![p.parse::<PU<AtomicExpr>>().map(Expr::Atomic)?];
        let mut ops = vec![];

        let get_binary = |p: &mut Parser<Token>| {
            let operator = p.parse::<PU<Operators>>()?;
            if operator.associativity() != OperatorAssociativity::Binary {
                operator.throw("atomic exprs must be connected with binary operators!")
            } else {
                Ok(operator)
            }
        };

        while let Some(op) = p.once::<PU<Operators>, _>(get_binary).apply(mapper::Try)? {
            let expr = p.parse::<PU<AtomicExpr>>().map(Expr::Atomic)?;

            while ops
                .last()
                .is_some_and(|p: &PU<Operators>| p.priority() <= op.priority())
            {
                let rhs = Box::new(exprs.pop().unwrap());
                let op = ops.pop().unwrap();
                let lhs = Box::new(exprs.pop().unwrap());

                exprs.push(Expr::Binary(lhs, op, rhs));
            }

            exprs.push(expr);
            ops.push(op);
        }

        while !ops.is_empty() {
            let rhs = Box::new(exprs.pop().unwrap());
            let op = ops.pop().unwrap();
            let lhs = Box::new(exprs.pop().unwrap());

            exprs.push(Expr::Binary(lhs, op, rhs));
        }

        Ok(exprs.pop().unwrap())
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
