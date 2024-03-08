use std::fmt::Debug;

use crate::tokens::{Location, Token};

pub type Result<'s, T> = std::result::Result<T, Option<Error<'s>>>;

pub type ParseResult<'s, T> = std::result::Result<Token<'s, T>, Option<Error<'s>>>;

#[derive(Clone)]
pub struct Error<'s> {
    location: Location<'s>,
    // reason :
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line_num, line) = self.location.backtrace_line();
        writeln!(f, "Error happend: ")?;
        writeln!(f, "at line {line_num}\t| {line}")
    }
}
