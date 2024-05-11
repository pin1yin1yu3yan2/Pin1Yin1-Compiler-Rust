use std::fmt::Debug;

use crate::*;

/// implement for a type and make it parseable
///
/// [`ParseUnit::Target`] measn the actual type of the parse result
///
/// S is the type of source
pub trait ParseUnit<S: Source>: Sized + Debug {
    type Target: Debug;

    /// you should not call [`ParseUnit::parse`] directly, using methods like [`Parser::once`]
    /// instead
    fn parse(p: &mut Parser<S>) -> Result<Self::Target, ParseError>;

    fn is_or<R, C, Or>(cond: C, or: Or) -> impl FnOnce(Self::Target) -> ParseResult<Self, S>
    where
        C: FnOnce(&Self::Target) -> bool,
        R: Into<ParseResult<Self, S>>,
        Or: FnOnce(Self::Target) -> R,
    {
        move |target| {
            if cond(&target) {
                Ok(target)
            } else {
                or(target).into()
            }
        }
    }

    fn eq_or<R, Or>(rhs: Self::Target, or: Or) -> impl FnOnce(Self::Target) -> ParseResult<Self, S>
    where
        Self::Target: PartialEq,
        R: Into<ParseResult<Self, S>>,
        Or: FnOnce(Self::Target) -> R,
    {
        Self::is_or(move |t| t == &rhs, or)
    }
}

// impl ParseUnit<char> for usize {
//     type Target = usize;

//     fn parse(p: &mut Parser) -> ParseResult<Self> {
//         let span = p.skip_whitespace().take_while(|c| c.is_ascii_digit());
//         let chars = p.select(span);
//         if chars.is_empty() {
//             return p.unmatch("no chars found");
//         }

//         let num = chars
//             .iter()
//             .rev()
//             .enumerate()
//             .map(|(fac, c)| (c.to_digit(10).unwrap() as usize) * 10usize.pow(fac as _))
//             .sum::<usize>();
//         Ok(num)
//     }
// }

pub trait ReverseParser<S: Source> {
    type Left;
    fn reverse_parse(&self, p: &mut Parser<S>) -> Result<Self::Left, ParseError>;
}

impl ReverseParser<char> for &str {
    type Left = ();
    fn reverse_parse(&self, p: &mut Parser) -> Result<(), error::ParseError> {
        if !self
            .chars()
            .all(|char| p.next().is_some_and(|next| *next == char))
        {
            return p.unmatch(format!("expect {}", self));
        }

        Ok(())
    }
}
