use crate::*;

pub type ParseResult<P, S = char> = Result<PU<P, S>>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// we use this trait to make the [`Result`] type more convenient
pub trait ResultExt<P> {
    type TryResult;

    /// try to parse, mean that [`ErrorKind::Unmatch`] is allowed
    ///
    /// in this case, [`ErrorKind::Unmatch`] will be transformed into [`None`]
    ///
    /// so that you can use `?` as usual after using match / if let ~
    fn r#try(self) -> Self::TryResult;

    fn apply(self, mapper: impl ParseMapper<P>) -> Self;
}

impl<P: ParseUnit<S>, S> ResultExt<PU<P, S>> for Result<PU<P, S>> {
    type TryResult = Result<Option<PU<P, S>>>;

    fn r#try(self) -> Self::TryResult {
        match self {
            Ok(pu) => Ok(Some(pu)),
            Err(e) if e.kind() == ErrorKind::Unmatch => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn apply(self, mapper: impl ParseMapper<PU<P, S>>) -> Self {
        mapper.mapper(self)
    }
}
