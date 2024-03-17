use crate::*;

pub enum ParseResult<'s, P: ParseUnit<S>, S: Copy = char> {
    Success(PU<'s, P, S>),
    Unmatch(Error<'s, S>),
    Failed(Error<'s, S>),
}

impl<'s, P: ParseUnit<S>, S: Copy> ParseResult<'s, P, S> {
    pub fn from_option<Se>(opt: Option<PU<'s, P, S>>, or: impl FnOnce() -> Se) -> Self
    where
        Se: Into<Self>,
    {
        match opt {
            Some(pu) => Self::Success(pu),
            None => or().into(),
        }
    }

    pub fn to_result(self) -> Result<'s, PU<'s, P, S>, S> {
        match self {
            ParseResult::Success(ok) => Ok(ok),
            ParseResult::Unmatch(e) => Err(ParseError::Unmatch(e)),
            ParseResult::Failed(e) => Err(ParseError::Error(e)),
        }
    }

    pub fn from_result(result: Result<'s, PU<'s, P, S>, S>) -> Self {
        match result {
            Ok(ok) => Self::Success(ok),
            Err(e) => e.into(),
        }
    }

    /// Returns `true` if the  parse result is [`Success`].
    ///
    /// [`Success`]: _ParseResult::Success
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(..))
    }

    /// Returns `true` if the  parse result is [`Unmatch`].
    ///
    /// [`Unmatch`]: _ParseResult::Unmatch
    #[must_use]
    pub fn is_unmatch(&self) -> bool {
        matches!(self, Self::Unmatch(..))
    }

    /// Returns `true` if the  parse result is [`Error`].
    ///
    /// [`Error`]: _ParseResult::Error
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed(..))
    }

    pub fn try_into_success(self) -> std::result::Result<PU<'s, P, S>, Self> {
        if let Self::Success(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn success(self) -> Option<PU<'s, P, S>> {
        self.try_into_success().ok()
    }

    pub fn try_into_unmatch(self) -> std::result::Result<Error<'s, S>, Self> {
        if let Self::Unmatch(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_error(self) -> std::result::Result<Error<'s, S>, Self> {
        if let Self::Failed(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn match_or(self, or: impl Into<String>) -> Self {
        match self.try_into_unmatch() {
            Ok(e) => Self::Failed(e.emit(or)),
            Err(s) => s,
        }
    }

    pub fn map<P1: ParseUnit<S>, M>(self, mapper: M) -> ParseResult<'s, P1, S>
    where
        M: FnOnce(P::Target<'s>) -> P1::Target<'s>,
    {
        match self {
            ParseResult::Success(success) => ParseResult::Success(success.map(mapper)),
            ParseResult::Unmatch(e) => ParseResult::Unmatch(e),
            ParseResult::Failed(e) => ParseResult::Failed(e),
        }
    }

    pub fn map_err(self, mapper: impl FnOnce(ParseError<'s, S>) -> ParseError<'s, S>) -> Self {
        match ParseError::try_from(self) {
            Ok(err) => mapper(err).into(),
            Err(ok) => Self::Success(ok),
        }
    }

    pub fn and_then<M>(self, mapper: M) -> Self
    where
        M: FnOnce(PU<'s, P, S>) -> Self,
    {
        match self.try_into_success() {
            Ok(pu) => mapper(pu),
            Err(s) => s,
        }
    }

    pub fn which_or<Se, C>(self, cond: C, or: impl FnOnce(PU<'s, P, S>) -> Se) -> Self
    where
        P::Target<'s>: PartialEq,
        Se: Into<Self>,
        C: FnOnce(&P::Target<'s>) -> bool,
    {
        self.and_then(|pu| {
            if cond(&pu) {
                Self::Success(pu)
            } else {
                or(pu).into()
            }
        })
    }

    pub fn eq_or<Se>(self, rhs: P::Target<'s>, or: impl FnOnce(PU<'s, P, S>) -> Se) -> Self
    where
        P::Target<'s>: PartialEq,
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
}

pub enum ParseError<'s, S: Copy = char> {
    Unmatch(Error<'s, S>),
    Error(Error<'s, S>),
}

impl<'s, S: Copy> ParseError<'s, S> {
    pub fn error(self, reason: impl Into<String>) -> Self {
        match self {
            ParseError::Unmatch(e) | ParseError::Error(e) => Self::Error(e.emit(reason)),
        }
    }

    pub fn unmatch(self, reason: impl Into<String>) -> Self {
        match self {
            ParseError::Unmatch(e) | ParseError::Error(e) => Self::Unmatch(e.emit(reason)),
        }
    }
}

impl<'s, P: ParseUnit<S>, S: Copy> From<ParseError<'s, S>> for ParseResult<'s, P, S> {
    fn from(value: ParseError<'s, S>) -> Self {
        match value {
            ParseError::Unmatch(e) => Self::Unmatch(e),
            ParseError::Error(e) => Self::Failed(e),
        }
    }
}

impl<'s, P: ParseUnit<S>, S: Copy> TryFrom<ParseResult<'s, P, S>> for ParseError<'s, S> {
    type Error = PU<'s, P, S>;

    fn try_from(
        value: ParseResult<'s, P, S>,
    ) -> std::prelude::v1::Result<
        Self,
        <result::ParseError<'s, S> as TryFrom<ParseResult<'s, P, S>>>::Error,
    > {
        match value {
            ParseResult::Success(pu) => Err(pu),
            ParseResult::Unmatch(e) => Ok(Self::Unmatch(e)),
            ParseResult::Failed(e) => Ok(Self::Error(e)),
        }
    }
}

impl<'s, P: ParseUnit<S>, S: Copy> std::ops::FromResidual for ParseResult<'s, P, S> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        residual.into()
    }
}

impl<'s, P: ParseUnit<S>, S: Copy>
    std::ops::FromResidual<std::result::Result<std::convert::Infallible, ParseError<'s, S>>>
    for ParseResult<'s, P, S>
{
    fn from_residual(
        residual: std::result::Result<std::convert::Infallible, ParseError<'s, S>>,
    ) -> Self {
        residual.err().unwrap().into()
    }
}

impl<'s, P: ParseUnit<S>, S: Copy> std::ops::Try for ParseResult<'s, P, S> {
    type Output = PU<'s, P, S>;

    type Residual = ParseError<'s, S>;

    fn from_output(output: Self::Output) -> Self {
        Self::Success(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            ParseResult::Success(ok) => std::ops::ControlFlow::Continue(ok),
            _ => std::ops::ControlFlow::Break(self.try_into().unwrap()),
        }
    }
}

impl<'s, P: ParseUnit> std::fmt::Debug for ParseResult<'s, P, char> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(arg0) => f.debug_tuple("Success").field(arg0).finish(),
            Self::Unmatch(arg0) => f.debug_tuple("Unmatch").field(arg0).finish(),
            Self::Failed(arg0) => f.debug_tuple("Failed").field(arg0).finish(),
        }
    }
}

impl<'s> std::fmt::Debug for ParseError<'s, char> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unmatch(arg0) => f.debug_tuple("Unmatch").field(arg0).finish(),
            Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
        }
    }
}
