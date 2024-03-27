use std::fmt::Debug;

use crate::*;

/// implement for a type and make it parseable
///
/// [`ParseUnit::Target`] measn the actual type of the parse result
///
/// S is the type of source
pub trait ParseUnit<S = char>: Sized {
    type Target: Debug;

    fn parse(p: &mut Parser<S>) -> ParseResult<Self, S>;

    fn is_or<R, C, Or>(cond: C, or: Or) -> impl FnOnce(PU<Self, S>) -> Result<PU<Self, S>>
    where
        C: FnOnce(&Self::Target) -> bool,
        R: Into<ParseResult<Self, S>>,
        Or: FnOnce(PU<Self, S>) -> R,
    {
        move |pu| {
            if cond(&pu.target) {
                Ok(pu)
            } else {
                or(pu).into()
            }
        }
    }

    fn eq_or<R, Or>(rhs: Self::Target, or: Or) -> impl FnOnce(PU<Self, S>) -> Result<PU<Self, S>>
    where
        Self::Target: PartialEq,
        R: Into<ParseResult<Self, S>>,
        Or: FnOnce(PU<Self, S>) -> R,
    {
        Self::is_or(move |t| t == &rhs, or)
    }
}

/// extract this function to make the addition of the UNICODE support much easier
///
/// but, whether how easy support UNICODE is, i wont support it, QWQ
pub(crate) const fn chars_taking_rule(c: &char) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
}

impl ParseUnit<char> for String {
    type Target = String;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let s: String = p.get_chars()?.iter().collect();
        p.finish(s)
    }
}

impl<'a> ParseUnit<char> for &'a [char] {
    type Target = &'a [char];

    fn parse(_p: &mut Parser) -> ParseResult<Self> {
        unimplemented!(
            "use `p.once(Parser::get_chars)` instead of `parse::<&[char]>(), to work around lifetime"
        )
    }
}

impl ParseUnit<char> for usize {
    type Target = usize;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let span = p.skip_whitespace().take_while(|c| c.is_ascii_digit());
        let chars = p.select(span);
        if chars.is_empty() {
            return p.unmatch("no chars found");
        }

        let num = chars
            .iter()
            .rev()
            .enumerate()
            .map(|(fac, c)| (c.to_digit(10).unwrap() as usize) * 10usize.pow(fac as _))
            .sum::<usize>();
        p.finish(num)
    }
}

impl ParseUnit<char> for char {
    type Target = char;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.start_taking();
        let next = p.next();
        let Some(char) = next.copied() else {
            return p.unmatch("no character rest");
        };
        p.finish(char)
    }
}
