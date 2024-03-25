use std::fmt::Debug;

use crate::*;

/// implement for a type and make it parseable
///
/// [`ParseUnit::Target`] measn the actual type of the parse result
///
/// [`S`] is the type of source
pub trait ParseUnit<S: Copy = char>: Sized {
    type Target: Debug;

    fn parse(p: &mut Parser<S>) -> ParseResult<Self, S>;
}

/// extract this function to make the addition of the UNICODE support much easier
///
/// but, whether how easy support UNICODE is, i wont support it, QWQ
pub(crate) const fn chars_taking_rule(c: char) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
}

impl ParseUnit for String {
    type Target = String;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let s = p.get_chars()?.iter().collect::<String>();
        p.finish(s)
    }
}

///  very hot funtion!!!

impl<'a> ParseUnit for &'a [char] {
    type Target = &'a [char];

    fn parse(_p: &mut Parser) -> ParseResult<Self> {
        todo!()
    }
}

impl ParseUnit for usize {
    type Target = usize;

    fn parse(p: &mut Parser) -> ParseResult<Self> {
        let chars = p.skip_whitespace().take_while(|c| c.is_ascii_digit());
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

impl ParseUnit for char {
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
