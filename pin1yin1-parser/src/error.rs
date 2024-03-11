use std::fmt::Debug;

use crate::*;

/// normally result, with an optional [`Error`]
pub type Result<'s, T> = std::result::Result<T, Option<Error<'s>>>;

/// normally parse result, storage [`T::Target`] in token
pub type ParseResult<'s, T> = std::result::Result<Token<'s, T>, Option<Error<'s>>>;

/// error type with a [`Selection`] and a [`String`] as reason
#[derive(Clone)]
pub struct Error<'s> {
    selection: Selection<'s>,
    reason: String,
}

impl<'s> Error<'s> {
    pub fn new(selection: Selection<'s>, reason: String) -> Self {
        Self { selection, reason }
    }

    pub fn emit(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let selection = self.selection;

        let left = (0..selection.start)
            .rev()
            .find(|idx| selection.src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);

        let right = (selection.start..selection.src.len())
            .find(|idx| selection.src[*idx] == '\n')
            .unwrap_or(selection.src.len());

        let line_num = (0..selection.start)
            .filter(|idx| selection.src[*idx] == '\n')
            .count()
            + 1;

        let line = selection.src[left..right].iter().collect::<String>();

        writeln!(f)?;
        // there is an error happend
        writeln!(f, "fa1sheng1le1yi1ge4cuo4wu4: {}", self.reason)?;
        // at line {line_num}
        let head = format!("zai4di4 {line_num} hang2\t| ");
        writeln!(f, "{head}{line}")?;
        let ahead = (0..head.len() + self.selection.start - left)
            .map(|_| ' ')
            .collect::<String>();
        let point = (0..self.selection.len()).map(|_| '^').collect::<String>();
        writeln!(f, "{ahead}{point}")
    }
}
