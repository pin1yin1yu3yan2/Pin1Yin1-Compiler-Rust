use std::fmt::Debug;

use crate::*;

/// implement for a type and make it parseable
///
/// [`ParseUnit::Target`] measn the actual type of the parse result
///
/// [`S`] is the type of source
pub trait ParseUnit<S: Copy = char>: Sized {
    type Target<'t>: Debug;

    fn parse<'s>(p: &mut Parser<'s, S>) -> ParseResult<'s, Self, S>;
}

/// extract this function to make the addition of the UNICODE support much easier
///
/// but, whether how easy to support UNICODE, i wont support it QWQ
pub(crate) const fn chars_taking_rule(c: char) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
}

impl ParseUnit for String {
    type Target<'t> = String;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let s = <&[char]>::parse(p)?.iter().collect::<String>();

        p.finish(s)
    }
}

///  very hot funtion!!!
impl ParseUnit for &[char] {
    type Target<'t> = &'t [char];

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        // reparse and cache the result
        if p.cache.first_index != p.idx {
            p.cache.first_index = p.idx;
            p.cache.chars = p.skip_whitespace().take_while(chars_taking_rule);
            p.cache.final_index = p.idx;
        } else {
            // load from cache, call p.start_taking() to perform the right behavior
            p.start_taking();
            p.idx = p.cache.final_index;
        }

        p.finish(p.cache.chars)
    }
}

impl ParseUnit for usize {
    type Target<'t> = usize;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let chars = p.skip_whitespace().take_while(|c| c.is_ascii_digit());
        if chars.is_empty() {
            return Err(None);
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
    type Target<'t> = char;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        p.start_taking();
        let next = p.next();
        let char = *next.ok_or(None)?;
        p.finish(char)
    }
}
