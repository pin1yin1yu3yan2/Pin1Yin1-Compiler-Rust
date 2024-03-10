use std::fmt::Debug;

use crate::*;

pub trait ParseUnit: Sized {
    type Target<'t>: Debug;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self>;
}

pub(crate) const fn chars_taking_rule(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl ParseUnit for String {
    type Target<'t> = String;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let s = p.parse::<&[char]>()?.iter().collect::<String>();
        p.finish(s)
    }
}

///  very hot funtion!!!
impl ParseUnit for &[char] {
    type Target<'t> = &'t [char];

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        if p.chars_cache_idx != p.idx {
            p.chars_cache_idx = p.idx;
            p.chars_cache = p.skip_whitespace().take_while(chars_taking_rule);
            p.chars_cache_final = p.idx;
        } else {
            // just skip
            p.idx = p.chars_cache_final;
        }

        p.finish(p.chars_cache)
    }
}

impl ParseUnit for usize {
    type Target<'t> = usize;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let num = p
            .skip_whitespace()
            .take_while(|c| c.is_ascii_digit())
            .iter()
            .rev()
            .enumerate()
            .map(|(fac, c)| (c.to_digit(10).unwrap() as usize) * 10usize.pow(fac as _))
            .sum();
        p.finish(num)
    }
}
