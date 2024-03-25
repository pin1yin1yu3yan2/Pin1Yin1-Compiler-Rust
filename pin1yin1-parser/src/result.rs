use crate::*;

pub type ParseResult<P, S = char> = Result<PU<P, S>>;

pub enum Result<T> {
    Success(T),
    Failed(ParseError),
}

impl<T> Result<T> {
    pub fn from_option<Se>(opt: Option<T>, or: impl FnOnce() -> Se) -> Self
    where
        Se: Into<Self>,
    {
        match opt {
            Some(pu) => Self::Success(pu),
            None => or().into(),
        }
    }

    pub fn to_result(self) -> std::result::Result<T, ParseError> {
        match self {
            Result::Success(ok) => Ok(ok),
            Result::Failed(e) => Err(e),
        }
    }

    pub fn from_result(result: std::result::Result<T, ParseError>) -> Self {
        match result {
            Ok(ok) => Self::Success(ok),
            Err(e) => e.into(),
        }
    }

    pub fn is_failed_and(&self, cond: impl FnOnce(&ParseError) -> bool) -> bool {
        match self {
            Result::Success(_) => false,
            Result::Failed(e) => cond(e),
        }
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(..))
    }

    #[must_use]
    pub fn is_unmatch(&self) -> bool {
        self.is_failed_and(|e| e.kind() == &ErrorKind::Unmatch)
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        self.is_failed_and(|e| e.kind() == &ErrorKind::OtherError)
    }

    pub fn try_into_success(self) -> std::result::Result<T, Self> {
        if let Self::Success(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn success(self) -> Option<T> {
        self.try_into_success().ok()
    }

    pub fn map<T1, M>(self, mapper: M) -> Result<T1>
    where
        M: FnOnce(T) -> T1,
    {
        match self {
            Result::Success(success) => Result::Success(mapper(success)),
            Result::Failed(e) => Result::Failed(e),
        }
    }
}
impl<P: ParseUnit<S>, S: Copy> Result<PU<P, S>> {
    pub fn map_pu<P1: ParseUnit<S>, M>(self, mapper: M) -> ParseResult<P1, S>
    where
        M: FnOnce(P::Target) -> P1::Target,
    {
        match self {
            Result::Success(success) => Result::Success(success.map(mapper)),
            Result::Failed(e) => Result::Failed(e),
        }
    }

    pub fn map_err(self, mapper: impl FnOnce(ParseError) -> ParseError) -> Self {
        match ParseError::try_from(self) {
            Ok(err) => mapper(err).into(),
            Err(ok) => Self::Success(ok),
        }
    }

    pub fn and_then<M>(self, mapper: M) -> Self
    where
        M: FnOnce(PU<P, S>) -> Self,
    {
        match self.try_into_success() {
            Ok(pu) => mapper(pu),
            Err(s) => s,
        }
    }

    pub fn which_or<Se, C>(self, cond: C, or: impl FnOnce(PU<P, S>) -> Se) -> Self
    where
        P::Target: PartialEq,
        Se: Into<Self>,
        C: FnOnce(&P::Target) -> bool,
    {
        self.and_then(|pu| {
            if cond(&pu) {
                Self::Success(pu)
            } else {
                or(pu).into()
            }
        })
    }

    pub fn eq_or<Se>(self, rhs: P::Target, or: impl FnOnce(PU<P, S>) -> Se) -> Self
    where
        P::Target: PartialEq,
        Se: Into<Self>,
    {
        self.and_then(|pu| {
            if *pu == rhs {
                Self::Success(pu)
            } else {
                or(pu).into()
            }
        })
    }

    pub fn must_match(self) -> Self {
        self.map_err(|e| e.to_error())
    }

    pub fn match_or<Se: Into<Self>>(self, or: impl FnOnce(Selection) -> Se) -> Self {
        match ParseError::try_from(self) {
            Ok(error) => {
                if error.kind() == &ErrorKind::Unmatch {
                    or(error.inner.get_selection()).into()
                } else {
                    Self::Failed(error)
                }
            }
            Err(ok) => Self::Success(ok),
        }
    }
}

pub struct ParseError {
    pub(crate) inner: Error,
    pub(crate) kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unmatch,
    Semantic,
    OtherError,
}

impl ParseError {
    pub fn new(inner: Error, kind: ErrorKind) -> Self {
        Self { inner, kind }
    }

    pub fn to_error(mut self) -> Self {
        self.kind = ErrorKind::OtherError;
        self
    }

    pub fn to_unmatch(mut self) -> Self {
        self.kind = ErrorKind::Unmatch;
        self
    }

    pub fn error(self, reason: impl Into<String>) -> Self {
        Self {
            inner: self.inner.emit(reason),
            kind: ErrorKind::OtherError,
        }
    }

    pub fn unmatch(self, reason: impl Into<String>) -> Self {
        Self {
            inner: self.inner.emit(reason),
            kind: ErrorKind::Unmatch,
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl<T> From<ParseError> for Result<T> {
    fn from(value: ParseError) -> Self {
        Result::Failed(value)
    }
}

impl<T> TryFrom<Result<T>> for ParseError {
    type Error = T;

    fn try_from(
        value: Result<T>,
    ) -> std::prelude::v1::Result<Self, <result::ParseError as TryFrom<Result<T>>>::Error> {
        match value {
            Result::Success(success) => Err(success),
            Result::Failed(e) => Ok(e),
        }
    }
}

impl<T> std::ops::FromResidual<std::result::Result<std::convert::Infallible, ParseError>>
    for Result<T>
{
    fn from_residual(residual: std::result::Result<std::convert::Infallible, ParseError>) -> Self {
        residual.err().unwrap().into()
    }
}

impl<T> std::ops::FromResidual for Result<T> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        residual.into()
    }
}

impl<T> std::ops::Try for Result<T> {
    type Output = T;

    type Residual = ParseError;

    fn from_output(output: Self::Output) -> Self {
        Self::Success(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Result::Success(ok) => std::ops::ControlFlow::Continue(ok),
            _ => std::ops::ControlFlow::Break(self.try_into().unwrap_or_else(|_| unreachable!())),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Result<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Success(arg0) => f.debug_tuple("Success").field(arg0).finish(),
            Self::Failed(arg0) => f.debug_tuple("Failed").field(arg0).finish(),
        }
    }
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ParseError")
            .field("inner", &self.inner)
            .field("kind", &self.kind)
            .finish()
    }
}
