use crate::*;

// TODO: lazy eval error messages for better performance

#[derive(Debug, Clone)]
pub struct Error {
    pub(crate) messages: Vec<Message>,
    pub(crate) kind: ErrorKind,
}

impl WithSpan for Error {
    fn get_span(&self) -> Span {
        self.messages.last().unwrap().get_span()
    }
}

impl FromIterator<Error> for Error {
    fn from_iter<T: IntoIterator<Item = Error>>(iter: T) -> Self {
        let mut kind = ErrorKind::OtherError;
        let mut messages = vec![];
        for err in iter.into_iter() {
            kind = err.kind();
            messages.extend(err.messages);
        }
        Self { messages, kind }
    }
}

impl Extend<Message> for Error {
    fn extend<T: IntoIterator<Item = Message>>(&mut self, iter: T) {
        self.messages.extend(iter)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub(crate) span: Span,
    pub(crate) reason: String,
}

impl WithSpan for Message {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl Message {
    pub fn new(span: Span, reason: impl ToString) -> Self {
        Self {
            span,
            reason: reason.to_string(),
        }
    }
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
    pub fn new(span: Span, reason: impl ToString, kind: ErrorKind) -> Self {
        Self {
            messages: vec![Message::new(span, reason)],
            kind,
        }
    }

    pub fn from_message(message: Message, kind: ErrorKind) -> Self {
        Self {
            messages: vec![message],
            kind,
        }
    }

    pub fn map(mut self, new_reason: impl ToString) -> Self {
        self.messages.last_mut().unwrap().reason = new_reason.to_string();
        self
    }

    pub fn append(mut self, new_span: Span, new_reason: impl ToString) -> Self {
        self.messages.push(Message::new(new_span, new_reason));
        self
    }

    pub fn append_message(mut self, message: Message) -> Self {
        self.messages.push(message);
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

    fn make_message(&self, reason: impl ToString) -> Message {
        Message::new(self.get_span(), reason)
    }

    fn make_error(&self, reason: impl ToString, kind: ErrorKind) -> Error {
        Error::new(self.get_span(), reason, kind)
    }

    fn unmatch<T>(&self, reason: impl ToString) -> Result<T> {
        Err(self.make_error(reason, ErrorKind::Unmatch))
    }

    fn throw<T>(&self, reason: impl ToString) -> Result<T> {
        Err(self.make_error(reason, ErrorKind::OtherError))
    }

    fn make_pu<P: ParseUnit<S>, S>(&self, target: P::Target) -> PU<P, S> {
        PU::new(self.get_span(), target)
    }
}
