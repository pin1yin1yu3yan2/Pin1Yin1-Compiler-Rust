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
        p.r#match(Symbol::Char)?;
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
        p.r#match(Symbol::String)?;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberLiteral {
    Float(f64),
    Digit(usize),
}

impl ParseUnit<Token> for NumberLiteral {
    type Target = NumberLiteral;

    fn parse(p: &mut Parser<Token>) -> ParseResult<Self, Token> {
        let number = p.parse::<Token>()?; // digit
        let mut int_dec = number.split('f');

        let Some(int) = int_dec.next() else {
            unreachable!()
        };
        let int = match int.parse::<usize>() {
            Ok(int) => int,
            Err(e) => return p.unmatch(e),
        };

        let dec = match int_dec.next() {
            Some("") => 0.0,
            Some(dec) => match dec.parse::<usize>() {
                Ok(0) => 0.0,
                Ok(dec) => dec as f64 / 10f64.powi(dec.ilog10() as _),
                Err(e) => return p.unmatch(e),
            },
            None => return Ok(Self::Digit(int)),
        };
        // check that if anything left
        if let Some(next) = int_dec.next() {
            return p.unmatch(format!("unexpect {}", next));
        }

        Ok(Self::Float(int as f64 + dec))
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
        p.r#match(Symbol::FnCallL)?;
        let Some(arg) = p.parse::<Expr>().apply(mapper::Try)? else {
            p.r#match(Symbol::FnCallR).apply(mapper::MustMatch)?;
            return Ok(FnCallArgs { args: vec![] });
        };

        let mut args = vec![arg];

        while p.r#match(Symbol::Semicolon).is_ok() {
            args.push(p.parse::<Expr>()?);
        }

        p.r#match(Symbol::FnCallR).apply(mapper::MustMatch)?;

        Ok(FnCallArgs { args })
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
pub struct Array {
    elements: Vec<Expr>,
}

impl std::ops::Deref for Array {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl ParseUnit<Token> for Array {
    type Target = Array;

    fn parse(p: &mut Parser<Token>) -> terl::Result<Self::Target, ParseError> {
        p.r#match(Symbol::ArrayL)?;
        let mut elements = vec![];
        while let Some(expr) = p.parse::<Expr>().apply(mapper::Try)? {
            elements.push(expr);
        }
        p.r#match(Symbol::ArrayR).apply(mapper::MustMatch)?;
        Ok(Self { elements })
    }
}

complex_pu! {
    cpu AtomicExpr {
        CharLiteral,
        StringLiteral,
        NumberLiteral,
        FnCall,
        Array,
        Variable
    }
}

#[derive(Debug, Clone)]
pub enum ExprItem {
    AtomicExpr(PU<AtomicExpr>),
    Operators(PU<Operators>),
}

impl WithSpan for ExprItem {
    fn get_span(&self) -> Span {
        match self {
            ExprItem::AtomicExpr(ws) => ws.get_span(),
            ExprItem::Operators(ws) => ws.get_span(),
        }
    }
}

impl From<PU<Operators>> for ExprItem {
    fn from(v: PU<Operators>) -> Self {
        Self::Operators(v)
    }
}

impl From<PU<AtomicExpr>> for ExprItem {
    fn from(v: PU<AtomicExpr>) -> Self {
        Self::AtomicExpr(v)
    }
}

#[derive(Debug, Clone)]
struct ExprItems;

impl ParseUnit<Token> for ExprItems {
    type Target = Vec<ExprItem>;

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

        let left_bracket = |items: &[ExprItem], nth: usize| {
            items
                .iter()
                .rev()
                .filter_map(|item| match item {
                    ExprItem::Operators(pu) if **pu == Operators::BracketL => Some(item.get_span()),
                    _ => None,
                })
                .nth(nth)
                .map(|span| span.make_message("left bracket here"))
        };

        enum Expect {
            Val,
            OP,
        }
        let mut items: Vec<ExprItem> = vec![];
        let mut bracket_depth = 0;
        let mut state = Expect::Val;
        loop {
            state = match state {
                Expect::Val => {
                    if let Some(lb) = p.r#match(RPU(Operators::BracketL)).apply(mapper::Try)? {
                        items.push(lb.into());
                        bracket_depth += 1;
                        Expect::Val
                    } else if let Some(unary) = p.once(get_unary_op).apply(mapper::Try)? {
                        items.push(unary.into());
                        Expect::Val
                    } else {
                        items.push(p.parse::<PU<AtomicExpr>>()?.into());
                        Expect::OP
                    }
                }
                Expect::OP => {
                    if bracket_depth != 0
                        && let Some(rb) = p.r#match(RPU(Operators::BracketR)).apply(mapper::Try)?
                    {
                        items.push(rb.into());

                        bracket_depth -= 1;
                        Expect::OP
                    } else if let Some(unary) = p.once(get_binary_op).apply(mapper::Try)? {
                        items.push(unary.into());
                        Expect::Val
                    } else if bracket_depth != 0 {
                        let left_bracket = left_bracket(&items, bracket_depth);
                        let current_span = p.get_span();
                        let expect_next = format!("expect this to be `{}`", Operators::BracketR);
                        let expect_next = p
                            .parse::<PU<Token>>()
                            .map(|tk| tk.make_message(expect_next));
                        break current_span.throw("unclosed bracket").map_err(|mut e| {
                            e.extend(left_bracket);
                            e.extend(expect_next.ok());
                            e
                        });
                    } else {
                        break Ok(items);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    items: Vec<ExprItem>,
    span: Span,
}

impl WithSpan for Expr {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl std::ops::Deref for Expr {
    type Target = Vec<ExprItem>;

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
            last.op_ty() != OperatorTypes::StructOperator && last.priority() <= current.priority()
        }

        for item in p.parse::<ExprItems>()? {
            match item {
                ExprItem::AtomicExpr(..) => {
                    exprs.push(item);
                }
                ExprItem::Operators(op) => match *op {
                    Operators::BracketL => ops.push(PU::new(item.get_span(), *op)),
                    Operators::BracketR => {
                        while let Some(op) = ops.pop() {
                            if *op == Operators::BracketL {
                                break;
                            }
                            exprs.push(op.into())
                        }
                    }
                    current => {
                        while ops.last().is_some_and(|last| {
                            could_fold(**last, current) && exprs.len() >= last.cost()
                        }) {
                            let last = ops.pop().unwrap();
                            exprs.push(last.into());
                        }
                        ops.push(PU::new(item.get_span(), *op));
                    }
                },
            }
        }

        for op in ops.into_iter().rev() {
            exprs.push(op.into());
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
            p.parse::<CharLiteral>()?;
            Ok(())
        });
    }

    #[test]
    fn string() {
        parse_test("chuan4 _t11514___na", |p| {
            p.parse::<StringLiteral>()?;
            Ok(())
        })
    }

    #[test]
    fn number1() {
        parse_test("114514", |p| {
            p.parse::<NumberLiteral>()?;
            Ok(())
        })
    }

    #[test]
    fn number2() {
        parse_test("114514f", |p| {
            p.parse::<NumberLiteral>()?;
            Ok(())
        })
    }

    #[test]
    fn number3() {
        parse_test("1919f810", |p| {
            p.parse::<NumberLiteral>()?;
            Ok(())
        })
    }

    #[test]
    fn function_call() {
        parse_test("ya1 1919810 fen1 chuan4 acminoac ru4 han2shu4", |p| {
            p.parse::<FnCall>()?;
            Ok(())
        })
    }

    #[test]
    fn unary() {
        parse_test("fei1 191810", |p| {
            p.parse::<Expr>()?;
            Ok(())
        })
    }

    #[test]
    fn nested_unary() {
        parse_test("fei1 fei1 fei1 fei1 191810", |p| {
            p.parse::<Expr>()?;
            Ok(())
        })
    }

    #[test]
    fn bracket() {
        // unary + bracket
        parse_test("fei1 jie2 114514 he2", |p| {
            p.parse::<Expr>()?;
            Ok(())
        })
    }

    #[test]
    fn complex_expr() {
        // 119 + 810 * 114514 - 12
        parse_test("1919 jia1 810 cheng2 114514 jian3 12", |p| {
            p.parse::<Expr>()?;
            Ok(())
        });
    }

    #[test]
    fn empty_array() {
        parse_test("zu3 he2", |p| {
            assert!(p.parse::<Array>()?.elements.is_empty());
            Ok(())
        });
    }

    #[test]
    fn array_with_an_element() {
        parse_test("zu3 jie2 1 he2 he2", |p| {
            p.parse::<Array>()?;
            Ok(())
        });
    }

    #[test]
    fn array_with_elements() {
        parse_test(
            concat!(
                "zu3 ",             // start array
                "jie2 1 he2 ",      // bracket
                "ya1 ru4 foo ",     // fn_call
                "a b c d e ",       // variables
                "114514 1919f810 ", // number
                "chuan4 awa ",      // string
                "he2"               // end array
            ),
            |p| {
                p.parse::<Array>()?;
                Ok(())
            },
        );
    }

    #[test]
    fn array_with_elements_with_white_space() {
        parse_test(
            concat!(
                "zu3%",                                       // start array
                "jie2#$#$1*&)*(he2^&*(^&*(",                  // bracket
                "ya1{#$*()!@*}ru4<>#$%*$%&*(%^*(*^&foo{[&*}", // fn_call
                "a啊b波呲d地e鹅",                             // variables
                "114514-=-=-=-1919f810<?><{}(*)",             // number
                "chuan4<>(^&%^%^&*awa$%&^",                   // string
                "he2"                                         // end array
            ),
            |p| {
                p.parse::<Array>()?;
                Ok(())
            },
        );
    }
}
