use std::fmt::Debug;

use crate::{tokens::Token, Selection};

pub type Result<'s, T> = std::result::Result<T, Option<Error<'s>>>;

pub type ParseResult<'s, T> = std::result::Result<Token<'s, T>, Option<Error<'s>>>;

#[derive(Clone)]
pub struct Error<'s> {
    selection: Selection<'s>,
    reason: String,
}

impl<'s> Error<'s> {
    pub fn new(selection: Selection<'s>, reason: String) -> Self {
        Self { selection, reason }
    }
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line_num, line) = self.selection.location().backtrace_line();
        writeln!(f, "fa1sheng1le1yi1ge4cuo4wu4: {}", self.reason)?;
        let head = format!("zai4di4{line_num}hang2\t| ");
        writeln!(f, "{head}{line}")?;
        let ahead = (0..head.len()).map(|_| ' ').collect::<String>();
        let point = (0..self.selection.len()).map(|_| '^').collect::<String>();
        writeln!(f, "{ahead}{point}")
    }
}
