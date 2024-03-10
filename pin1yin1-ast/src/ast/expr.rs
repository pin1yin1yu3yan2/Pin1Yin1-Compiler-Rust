use crate::keywords::syntax;

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

// pub enum Literal<'s> {}

// pub struct Expr<'s> {}

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
}
