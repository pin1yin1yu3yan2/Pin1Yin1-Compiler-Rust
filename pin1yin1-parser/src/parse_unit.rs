use std::fmt::Debug;

use crate::*;

pub trait ParseUnit: Sized {
    type Target<'t>: Debug;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self>;
}

impl ParseUnit for String {
    type Target<'t> = String;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
        let s = p
            .skip_whitespace()
            .take_while(|s| s.is_ascii_alphanumeric())
            .iter()
            .collect::<String>();
        p.finish(s)
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
