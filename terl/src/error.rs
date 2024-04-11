use crate::*;

// TODO: lazy eval error messages for better performance

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    Unmatch,
    Semantic,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub(crate) error: Error,
    pub(crate) kind: ParseErrorKind,
}

impl std::ops::Deref for ParseError {
    type Target = Error;

    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl std::ops::DerefMut for ParseError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.error
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Location(Span),
    Text(String),
    Rich(String, Span),
}

impl Message {
    pub fn rich(message: String, at: Span) -> Self {
        Self::Rich(message, at)
    }
}

impl From<Span> for Message {
    fn from(v: Span) -> Self {
        Self::Location(v)
    }
}

impl<S: ToString> From<S> for Message {
    fn from(v: S) -> Self {
        Self::Text(v.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub(crate) main_span: Span,
    pub(crate) main_message: String,
    pub(crate) messages: Vec<Message>,
}

impl Error {
    pub fn new(main_span: Span, reason: impl ToString) -> Self {
        Self {
            main_span,
            main_message: reason.to_string(),
            messages: vec![],
        }
    }

    pub fn append(mut self, message: impl Into<Message>) -> Self {
        self.messages.push(message.into());
        self
    }
}

impl WithSpan for Error {
    fn get_span(&self) -> Span {
        self.main_span
    }
}

impl Extend<Message> for Error {
    fn extend<T: IntoIterator<Item = Message>>(&mut self, iter: T) {
        self.messages.extend(iter)
    }
}

impl std::ops::Add for Error {
    type Output = Error;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.messages.extend(
            std::iter::once(Message::rich(rhs.main_message, rhs.main_span)).chain(rhs.messages),
        );
        self
    }
}

impl std::ops::AddAssign for Error {
    fn add_assign(&mut self, rhs: Self) {
        self.messages.extend(
            std::iter::once(Message::rich(rhs.main_message, rhs.main_span)).chain(rhs.messages),
        );
    }
}

impl<M: Into<Message>> std::ops::Add<M> for Error {
    type Output = Error;

    fn add(mut self, rhs: M) -> Self::Output {
        self.messages.push(rhs.into());
        self
    }
}

impl<M: Into<Message>> std::ops::AddAssign<M> for Error {
    fn add_assign(&mut self, rhs: M) {
        self.messages.push(rhs.into());
    }
}

impl ParseError {
    pub fn new(span: Span, reason: impl ToString, kind: ParseErrorKind) -> Self {
        Self {
            error: Error::new(span, reason),
            kind,
        }
    }

    pub fn map(mut self, new_reason: impl ToString) -> Self {
        match self.messages.last_mut().unwrap() {
            Message::Location(_) => self.messages.push(Message::Text(new_reason.to_string())),
            Message::Rich(text, _) | Message::Text(text) => *text = new_reason.to_string(),
        }
        self
    }

    pub fn append(mut self, message: impl Into<Message>) -> Self {
        self.messages.push(message.into());
        self
    }

    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    pub fn to_error(mut self) -> Self {
        self.kind = ParseErrorKind::Semantic;
        self
    }

    pub fn to_unmatch(mut self) -> Self {
        self.kind = ParseErrorKind::Unmatch;
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

    fn make_parse_error(&self, reason: impl ToString, kind: ParseErrorKind) -> ParseError {
        ParseError::new(self.get_span(), reason, kind)
    }

    fn make_error(&self, reason: impl ToString) -> Error {
        Error::new(self.get_span(), reason)
    }

    fn make_message(&self, reason: impl ToString) -> Message {
        Message::Rich(reason.to_string(), self.get_span())
    }

    fn unmatch<T>(&self, reason: impl ToString) -> Result<T, ParseError> {
        Err(self.make_parse_error(reason, ParseErrorKind::Unmatch))
    }

    fn throw<T>(&self, reason: impl ToString) -> Result<T, ParseError> {
        Err(self.make_parse_error(reason, ParseErrorKind::Semantic))
    }

    fn make_pu<P: ParseUnit<S>, S>(&self, target: P::Target) -> PU<P, S> {
        PU::new(self.get_span(), target)
    }
}
