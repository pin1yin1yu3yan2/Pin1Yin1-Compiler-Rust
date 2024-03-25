use crate::*;

// TODO: multiple selections, multiple reasons for more friendlier error messages
// TODO: lazy eval error messages for better performance

/// error type with a [`Selection`] and a [`String`] as reason
///
///
///
#[derive(Clone, Debug)]
pub struct Error {
    pub(crate) selection: Selection,
    pub(crate) reason: String,
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
