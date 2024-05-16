use crate::*;

/// possiable error kind of parse error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// unmatched
    Unmatch,
    /// matched, but has semantic error
    Semantic,
}

/// bundle of [`Error`] and [`ParseErrorKind`]
#[derive(Debug, Clone)]
pub struct ParseError {
    pub(crate) error: Error,
    pub(crate) kind: ParseErrorKind,
}

impl ParseError {
    /// create an [`ParseError`]
    pub fn new(span: Span, reason: impl ToString, kind: ParseErrorKind) -> Self {
        Self {
            error: Error::new(span, reason),
            kind,
        }
    }

    /// same as [`Error::append`]
    pub fn append(mut self, message: impl Into<Message>) -> Self {
        self.messages.push(message.into());
        self
    }

    /// return the kind of [`ParseError`]
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    /// take [`Error`] from [`ParseError`]
    pub fn error(self) -> Error {
        self.error
    }
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

/// an error message
#[derive(Debug, Clone)]
pub enum Message {
    /// an error message with only location
    Location(Span),
    /// an error message with only text
    Text(String),
    /// an error message with both location and text
    Rich(String, Span),
}

impl Message {
    /// create a error message with location and text
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

/// an error, with many messages in
#[derive(Debug, Clone)]
pub struct Error {
    pub(crate) messages: Vec<Message>,
}

impl Error {
    /// create an [`Error`] with [`Message::Rich`] in
    pub fn new(main_span: Span, reason: impl ToString) -> Self {
        Self {
            messages: vec![main_span.make_message(reason)],
        }
    }

    /// append an error [`Message`], and return [`Error`] for chain-calling
    pub fn append(mut self, message: impl Into<Message>) -> Self {
        self.messages.push(message.into());
        self
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
        self.messages.extend(rhs.messages);
        self
    }
}

impl std::ops::AddAssign for Error {
    fn add_assign(&mut self, rhs: Self) {
        self.messages.extend(rhs.messages);
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

/// tag types with a location information in sorce file
pub trait WithSpan {
    /// get the location information
    fn get_span(&self) -> Span;

    /// make an [`Error`] at location
    fn make_error(&self, reason: impl ToString) -> Error {
        Error::new(self.get_span(), reason)
    }

    /// make an [`Message`] at location
    fn make_message(&self, reason: impl ToString) -> Message {
        Message::Rich(reason.to_string(), self.get_span())
    }
}

/// making [`ParseError`] extend for [`WithSpan`] trait
pub trait WithSpanExt: WithSpan {
    /// make an [`ParseError`] at location with ordered [`ParseErrorKind`]
    fn make_parse_error(&self, reason: impl ToString, kind: ParseErrorKind) -> ParseError {
        ParseError::new(self.get_span(), reason, kind)
    }

    /// make an Unmatched [`ParseError`] in [`Result`]
    fn unmatch<T>(&self, reason: impl ToString) -> Result<T, ParseError> {
        Err(self.make_parse_error(reason, ParseErrorKind::Unmatch))
    }

    /// make an Semantic [`ParseError`] in [`Result`]
    fn throw<T>(&self, reason: impl ToString) -> Result<T, ParseError> {
        Err(self.make_parse_error(reason, ParseErrorKind::Semantic))
    }
}

impl<W: WithSpan> WithSpanExt for W {}
