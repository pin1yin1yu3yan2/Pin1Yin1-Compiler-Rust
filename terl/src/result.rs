use crate::*;

pub type ParseResult<P, S> = Result<<P as ParseUnit<S>>::Target, ParseError>;
pub type Result<T, E = Error> = std::result::Result<T, E>;
