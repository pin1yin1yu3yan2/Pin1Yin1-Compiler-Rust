use crate::*;

// TODO: multiple selections, multiple reasons for more friendlier error messages
// TODO: lazy eval error messages for better performance

#[derive(Debug, Clone)]
pub struct Error {
    pub(crate) span: Span,
    pub(crate) reason: String,
    pub(crate) kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unmatch,
    Semantic,
    OtherError,
    /// Debug only
    NotError,
}

impl Error {
    pub fn new(span: Span, reason: impl Into<String>, kind: ErrorKind) -> Self {
        Self {
            span,
            reason: reason.into(),
            kind,
        }
    }

    pub fn map(mut self, new_reason: impl Into<String>) -> Self {
        self.reason = new_reason.into();
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn to_error(mut self) -> Self {
        self.kind = ErrorKind::OtherError;
        self
    }

    pub fn to_unmatch(mut self) -> Self {
        self.kind = ErrorKind::Unmatch;
        self
    }
}

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Result::Err(value)
    }
}

impl<T> TryFrom<Result<T>> for Error {
    type Error = T;

    fn try_from(
        value: Result<T>,
    ) -> std::prelude::v1::Result<Self, <Error as TryFrom<Result<T>>>::Error> {
        match value {
            Result::Ok(success) => Err(success),
            Result::Err(e) => Ok(e),
        }
    }
}

pub trait WithSpan {
    fn get_span(&self) -> Span;

    fn make_error(&self, reason: impl Into<String>, kind: ErrorKind) -> Error {
        Error::new(self.get_span(), reason, kind)
    }

    fn unmatch<T>(&self, reason: impl Into<String>) -> Result<T> {
        Err(self.make_error(reason, ErrorKind::Unmatch))
    }

    fn throw<T>(&self, reason: impl Into<String>) -> Result<T> {
        Err(self.make_error(reason, ErrorKind::OtherError))
    }

    fn make_pu<P: ParseUnit<S>, S>(&self, target: P::Target) -> PU<P, S> {
        PU::new(self.get_span(), target)
    }
}

impl WithSpan for Error {
    fn get_span(&self) -> Span {
        self.span
    }
}
