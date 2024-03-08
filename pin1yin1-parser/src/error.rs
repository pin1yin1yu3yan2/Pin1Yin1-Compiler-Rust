use crate::{parse_unit::ParseUnit, parser::Location, tokens::Token};

pub type Result<'s, T> = std::result::Result<T, Option<Error<'s>>>;

pub type ParseResult<'s, T: ParseUnit> = std::result::Result<Token<'s, T>, Option<Error<'s>>>;

#[derive(Debug, Clone)]
pub struct Error<'s> {
    location: Location<'s>,
    // reason :
}
