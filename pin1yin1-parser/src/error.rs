use std::fmt::Debug;

use crate::*;

// TODO: multiple selections, multiple reasons for more friendlier error messages
// TODO: lazy eval error messages for better performance

/// error type with a [`Selection`] and a [`String`] as reason
///
///
///
#[derive(Clone)]
pub struct Error {
    selection: Selection,
    reason: String,
}

impl Error {
    pub fn new(selection: Selection, reason: String) -> Self {
        Self { selection, reason }
    }

    pub fn emit(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
}

impl Debug for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // let selection = self.selection;

        // let left = (0..selection.start)
        //     .rev()
        //     .find(|idx| selection.src[*idx] == '\n')
        //     .map(|idx| idx + 1)
        //     .unwrap_or(0);

        // let right = (selection.start..selection.src.len())
        //     .find(|idx| selection.src[*idx] == '\n')
        //     .unwrap_or(selection.src.len());

        // let line_num = (0..selection.start)
        //     .filter(|idx| selection.src[*idx] == '\n')
        //     .count()
        //     + 1;

        // let row_num = selection.start - left;

        // let line = selection.src[left..right].iter().collect::<String>();

        // let location = format!("[{}:{}:{}]", selection.src.file_name(), line_num, row_num,);

        // writeln!(f)?;
        // // there is an error happend
        // writeln!(f, "{location}Error: {}", self.reason)?;
        // // at line {line_num}
        // let head = format!("at line {line_num} | ");
        // writeln!(f, "{head}{line}")?;
        // let ahead = (0..head.len() + selection.start - left)
        //     .map(|_| ' ')
        //     .collect::<String>();
        // let point = (0..selection.len()).map(|_| '^').collect::<String>();
        // writeln!(f, "{ahead}{point}")
        todo!()
    }
}

pub trait WithSelection {
    fn get_selection(&self) -> Selection;

    /// make a new [`Error`] with the given value and parser's selection
    fn new_error(&self, reason: impl Into<String>) -> Error {
        Error::new(self.get_selection(), reason.into())
    }

    fn make_error(&self, reason: impl Into<String>, kind: ErrorKind) -> ParseError {
        ParseError::new(self.new_error(reason), kind)
    }

    fn unmatch<T>(&self, reason: impl Into<String>) -> Result<T> {
        Result::Failed(self.make_error(reason, ErrorKind::Unmatch))
    }

    fn throw<T>(&self, reason: impl Into<String>) -> Result<T> {
        Result::Failed(self.make_error(reason, ErrorKind::OtherError))
    }
}

impl WithSelection for Error {
    fn get_selection(&self) -> Selection {
        self.selection
    }
}
