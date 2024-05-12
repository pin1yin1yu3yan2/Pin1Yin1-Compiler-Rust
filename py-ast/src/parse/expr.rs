use py_lex::{
    ops::{OperatorAssociativity, OperatorTypes, Operators},
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
    pub args: Vec<Expr>,
}

impl ParseUnit<Token> for Initialization {
    type Target = Initialization;

    fn parse(_p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        todo!()
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

complex_pu! {
    cpu AtomicExpr {
        CharLiteral,
        StringLiteral,
        NumberLiteral,
        FnCall,
        Variable
        // Initialization
    }
}

complex_pu! {
    cpu ExprItem {
        AtomicExpr,
        Operators
    }
}

#[derive(Debug, Clone)]
struct ExprItems;

impl ParseUnit<Token> for ExprItems {
    type Target = Vec<PU<ExprItem>>;

    fn parse(p: &mut Parser<Token>) -> terl::Result<Self::Target, ParseError> {
        let get_unary_op = |p: &mut Parser<Token>| {
            p.parse::<PU<Operators>>().apply(mapper::Satisfy::new(
                |op: &PU<Operators>| op.associativity() == OperatorAssociativity::Unary,
                |e| e.unmatch(""),
            ))
        };
        let get_binary_op = |p: &mut Parser<Token>| {
            p.parse::<PU<Operators>>().apply(mapper::Satisfy::new(
                |op: &PU<Operators>| op.associativity() == OperatorAssociativity::Binary,
                |e| e.unmatch(""),
            ))
        };

        let last_left_bracket = |items: &[PU<ExprItem>]| {
            items.iter().rev().find_map(|item| match &**item {
                ExprItem::Operators(Operators::BracketL) => {
                    Some(item.get_span().make_message("last left bracket here"))
                }
                _ => None,
            })
        };

        enum Expect {
            Val,
            OP,
        }
        let mut items = vec![];
        let mut bracket_depth = 0;
        let mut state = Expect::Val;
        loop {
            state = match state {
                Expect::Val => {
                    if let Some(bl) = p.match_(RPU(Operators::BracketL)).apply(mapper::Try)? {
                        items.push(bl.map(Into::into));
                        bracket_depth += 1;
                        Expect::Val
                    } else if let Some(unary) = p.once(get_unary_op).apply(mapper::Try)? {
                        items.push(unary.map(Into::into));
                        Expect::Val
                    } else {
                        items.push(p.parse::<PU<AtomicExpr>>()?.map(Into::into));
                        Expect::OP
                    }
                }
                Expect::OP => {
                    if let Some(bl) = p.match_(RPU(Operators::BracketR)).apply(mapper::Try)? {
                        items.push(bl.map(Into::into));
                        bracket_depth -= 1;
                        Expect::OP
                    } else if let Some(unary) = p.once(get_binary_op).apply(mapper::Try)? {
                        items.push(unary.map(Into::into));
                        Expect::Val
                    } else {
                        if bracket_depth != 0 {
                            break p.throw("unclosed bracket").map_err(|mut e| {
                                e.extend(last_left_bracket(&items));
                                e
                            });
                        }
                        break Ok(items);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    items: Vec<PU<ExprItem>>,
    span: Span,
}

impl WithSpan for Expr {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl std::ops::Deref for Expr {
    type Target = Vec<PU<ExprItem>>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl ParseUnit<Token> for Expr {
    type Target = Expr;

    fn parse(p: &mut Parser<Token>) -> terl::Result<Self::Target, ParseError> {
        // is heap allocation fewer than previous algo?
        let mut exprs = vec![];
        let mut ops: Vec<PU<Operators>> = vec![];

        fn could_fold(last: Operators, current: Operators) -> bool {
            last.op_ty() != OperatorTypes::Bracket && last.priority() <= current.priority()
        }

        for item in p.parse::<ExprItems>()? {
            match &*item {
                ExprItem::AtomicExpr(_) => {
                    exprs.push(item);
                }
                ExprItem::Operators(op) => match *op {
                    Operators::BracketL => ops.push(PU::new(item.get_span(), *op)),
                    Operators::BracketR => {
                        while let Some(op) = ops.pop() {
                            if *op == Operators::BracketL {
                                break;
                            }
                            exprs.push(op.map(Into::into))
                        }
                    }
                    current => {
                        while ops.last().is_some_and(|last| {
                            could_fold(**last, current) && exprs.len() >= last.cost()
                        }) {
                            let last = ops.pop().unwrap();
                            exprs.push(last.map(Into::into));
                        }
                        ops.push(PU::new(item.get_span(), *op));
                    }
                },
            }
        }

        for op in ops.into_iter().rev() {
            exprs.push(op.map(Into::into));
        }

        Ok(Self {
            items: exprs,
            span: p.get_span(),
        })
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
    fn function_call() {
        parse_test("ya1 1919810 fen1 chuan4 acminoac ru4 han2shu4", |p| {
            assert!(p.parse::<FnCall>().is_ok());
        })
    }

    #[test]
    fn unary() {
        parse_test("fei1 191810", |p| {
            assert!(p.parse::<Expr>().is_ok());
        })
    }

    #[test]
    fn nested_unary() {
        parse_test("fei1 fei1 fei1 fei1 191810", |p| {
            assert!(p.parse::<Expr>().is_ok());
        })
    }

    #[test]
    fn bracket() {
        // unary + bracket
        parse_test("fei1 jie2 114514 he2", |p| {
            assert!(p.parse::<Expr>().is_ok());
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
