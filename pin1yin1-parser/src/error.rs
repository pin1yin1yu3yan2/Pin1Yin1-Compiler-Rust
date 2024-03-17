use std::fmt::Debug;

use crate::*;

/// normally result, with an optional [`Error`]
pub type Result<'s, T, S = char> = std::result::Result<T, ParseError<'s, S>>;

// /// normally parse result, storage [`T::Target`] in token
// pub type ParseResult<'s, P, S = char> = std::result::Result<PU<'s, P, S>, Option<Error<'s, S>>>;

/// error type with a [`Selection`] and a [`String`] as reason
#[derive(Clone)]
pub struct Error<'s, S: Copy = char> {
    // TODO: multiple selections, multiple reasons for more friendlier error messages
    selection: Selection<'s, S>,
    reason: String,
}

impl<'s, S: Copy> Error<'s, S> {
    pub fn new(selection: Selection<'s, S>, reason: String) -> Self {
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

        let row_num = selection.start - left;

        let line = selection.src[left..right].iter().collect::<String>();

        let location = format!("[{}:{}:{}]", selection.src.file_name(), line_num, row_num,);

        writeln!(f)?;
        // there is an error happend
        writeln!(f, "{location}Error: {}", self.reason)?;
        // at line {line_num}
        let head = format!("at line {line_num} | ");
        writeln!(f, "{head}{line}")?;
        let ahead = (0..head.len() + selection.start - left)
            .map(|_| ' ')
            .collect::<String>();
        let point = (0..selection.len()).map(|_| '^').collect::<String>();
        writeln!(f, "{ahead}{point}")
    }
}

pub trait WithSelection<'s, S: Copy = char> {
    fn get_selection(&self) -> Selection<'s, S>;

    /// make a new [`Error`] with the given value and parser's selection
    fn make_error(&self, reason: impl Into<String>) -> Error<'s, S> {
        Error::new(self.get_selection(), reason.into())
    }

    fn unmatch<P: ParseUnit<S>>(&self, reason: impl Into<String>) -> ParseResult<'s, P, S> {
        ParseResult::Unmatch(self.make_error(reason))
    }

    fn unmatch_error(&self, reason: impl Into<String>) -> ParseError<'s, S> {
        ParseError::Unmatch(self.make_error(reason))
    }

    fn throw<P: ParseUnit<S>>(&self, reason: impl Into<String>) -> ParseResult<'s, P, S> {
        ParseResult::Failed(self.make_error(reason))
    }

    fn failed_error(&self, reason: impl Into<String>) -> ParseError<'s, S> {
        ParseError::Unmatch(self.make_error(reason))
    }
}
