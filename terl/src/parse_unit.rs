use std::fmt::Debug;

use crate::*;

/// implement for a type and make it be able to be parsed from [`Parser`]
pub trait ParseUnit<S: Source>: Sized + Debug {
    /// the type of the parse result
    type Target: Debug;

    /// you should not call [`ParseUnit::parse`] directly, using methods like [`Parser::once`] instead
    fn parse(p: &mut Parser<S>) -> Result<Self::Target, ParseError>;
}

/// testfor is the type can be parsed from [`Parser`]
pub trait ReverseParseUnit<S: Source> {
    /// the type of the reverse parse result
    type Left;
    /// you should not call [`ReverseParser::reverse_parse`] directly, using [`Parser::r#match`] instead
    fn reverse_parse(&self, p: &mut Parser<S>) -> Result<Self::Left, ParseError>;
}
