use crate::{complex_pu, keywords::syntax};

use super::*;

#[derive(Debug, Clone)]
pub struct CharLiteral<'s> {
    pub zi4: Token<'s, syntax::Symbol>,
    pub unparsed: Token<'s, String>,
    pub parsed: char,
}

fn escape<'s>(src: &Token<'s, String>, c: char) -> Result<'s, char> {
    Ok(match c {
        '_' => '_',
        't' => '\t',
        'n' => '\n',
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
        let zi4 = p.parse::<syntax::Symbol>()?.is(syntax::Symbol::Char)?;
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

#[derive(Debug, Clone)]
pub struct StringLiteral<'s> {
    pub chuan4: Token<'s, syntax::Symbol>,
    pub unparsed: Token<'s, String>,
    pub parsed: String,
}

impl ParseUnit for StringLiteral<'_> {
    type Target<'t> = StringLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let chuan4 = p.parse::<syntax::Symbol>()?.is(syntax::Symbol::String)?;
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

#[derive(Debug, Clone, Copy)]
pub struct NumberLiteral<'s> {
    pub number: f64,
    _p: &'s (),
}

impl ParseUnit for NumberLiteral<'_> {
    type Target<'t> = NumberLiteral<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let mut number = *p.parse::<usize>()? as f64;

        if p.parse::<char>().is_ok_and(|c| *c == '.') {
            let decimal = p.parse::<usize>().map(|t| *t).unwrap_or(0) as f64;
            number += decimal / 10f64.powi(decimal.log10().ceil() as _);
        }

        p.finish(NumberLiteral { number, _p: &() })
    }
}

#[derive(Debug, Clone)]
pub struct Arguments<'s> {
    pub parms: Vec<Token<'s, Expr<'s>>>,
    pub semicolons: Vec<Token<'s, syntax::Symbol>>,
}

impl ParseUnit for Arguments<'_> {
    type Target<'t> = Arguments<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        // may be empty
        let Ok(parm) = p.parse::<Expr>() else {
            return p.finish(Arguments {
                parms: vec![],
                semicolons: vec![],
            });
        };

        let mut parms = vec![parm];
        let mut semicolons = vec![];

        while let Ok(semicolon) = p
            .r#try(|p| p.parse::<syntax::Symbol>()?.is(syntax::Symbol::Semicolon))
            .finish()
        {
            semicolons.push(semicolon);
            if let Ok(parm) = p.parse::<Expr>() {
                parms.push(parm)
            } else {
                break;
            }
        }

        p.finish(Arguments { parms, semicolons })
    }
}

#[derive(Debug, Clone)]
pub struct Initialization<'s> {
    pub han2: Token<'s, syntax::Symbol>,
    pub args: Vec<Token<'s, Expr<'s>>>,
    pub jie2: Token<'s, syntax::Symbol>,
}

impl ParseUnit for Initialization<'_> {
    type Target<'t> = Initialization<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let han2 = p.parse::<syntax::Symbol>()?;
        let mut args = vec![];
        while let Ok(expr) = p.parse::<Expr>() {
            args.push(dbg!(expr));
        }
        let jie2 = p
            .parse::<syntax::Symbol>()
            .map_err(|e| e.map(|e| e.emit("invalid Initialization Expr")))?;

        p.finish(Initialization { han2, args, jie2 })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'s> {
    pub fn_name: Token<'s, Ident<'s>>,
    pub han2: Token<'s, syntax::Symbol>,
    pub args: Token<'s, Arguments<'s>>,
    pub jie2: Token<'s, syntax::Symbol>,
}

impl ParseUnit for FunctionCall<'_> {
    type Target<'t> = FunctionCall<'t>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let fn_name = p.parse::<Ident>()?;
        let han2 = p.parse::<syntax::Symbol>()?.is(syntax::Symbol::Parameter)?;
        let args = p.parse::<Arguments>()?;
        let jie2 = p
            .r#try(|p| {
                p.parse::<syntax::Symbol>()
                    .or_else(|_| p.throw("should insert jie2"))?
                    .is_or(syntax::Symbol::EndOfBracket, |t| t.throw("should be jie2"))
            })
            .finish()?;

        p.finish(FunctionCall {
            fn_name,
            han2,
            args,
            jie2,
        })
    }
}

complex_pu! {
    cpu Expr {
        CharLiteral,
        StringLiteral,
        NumberLiteral,
        Initialization,
        FunctionCall
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
            assert!(dbg!(p.parse::<NumberLiteral>()).is_ok());
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
            assert!(dbg!(p.parse::<FunctionCall>()).is_ok());
        })
    }
}
