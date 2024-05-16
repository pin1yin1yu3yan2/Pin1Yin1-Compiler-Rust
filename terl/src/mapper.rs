use std::marker::PhantomData;

use crate::*;

/// an mapper that can be applied on [`ParseResult`]
pub trait ParseMapper<P> {
    /// mapped result
    type Result;
    /// map a [`Result<P, ParseError>`] to [`ParseMapper::Result`]
    fn map(self, result: Result<P, ParseError>) -> Self::Result;
}

/// an extend for [`Result`]
pub trait ResultMapperExt<P> {
    /// binding for [`ParseMapper::Result`]
    type Result<Mapper: ParseMapper<P>>;
    /// apply a [`ParseMapper`] to a [`Result<P, ParseError>`]
    fn apply<M: ParseMapper<P>>(self, mapper: M) -> Self::Result<M>;
}

impl<P> ResultMapperExt<P> for Result<P, ParseError> {
    type Result<Mapper: ParseMapper<P>> = Mapper::Result;

    fn apply<M: ParseMapper<P>>(self, mapper: M) -> Self::Result<M> {
        mapper.map(self)
    }
}

/// turn [`ParseErrorKind`] in [`ParseResult`] into [`ParseErrorKind::Semantic`]
///
/// so that [`Try`] will not filter out [`Error`]
#[derive(Debug, Clone, Copy)]
pub struct MustMatch;

impl<P> ParseMapper<P> for MustMatch {
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        result.map_err(|e| {
            if e.kind() == ParseErrorKind::Unmatch {
                ParseError {
                    error: e.error(),
                    kind: ParseErrorKind::Semantic,
                }
            } else {
                e
            }
        })
    }
}

/// testfor is result equal to expect value
#[derive(Debug, Clone)]
pub struct Equal<P, E> {
    eq: P,
    or: E,
}

impl<P, E> ParseMapper<P> for Equal<P, E>
where
    P: PartialEq + WithSpan,
    E: FnOnce(Span) -> Result<P, ParseError>,
{
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        let result = result?;
        if result == self.eq {
            Ok(result)
        } else {
            (self.or)(result.get_span())
        }
    }
}

impl<P, E> Equal<P, E>
where
    P: PartialEq + WithSpan,
    E: FnOnce(Span) -> Result<P, ParseError>,
{
    /// create a new [`Equal`] mapper
    ///
    /// * eq: expect val
    ///
    /// * or: Error generator
    pub fn new(eq: P, or: E) -> Self {
        Self { eq, or }
    }
}

/// testfor is result satisfy the condition
pub struct Satisfy<P, C, E>
where
    C: FnOnce(&P) -> bool,
    E: FnOnce(Span) -> Result<P, ParseError>,
{
    cond: C,
    or: E,
    _p: PhantomData<P>,
}

impl<P, C, E> ParseMapper<P> for Satisfy<P, C, E>
where
    P: WithSpan,
    C: FnOnce(&P) -> bool,
    E: FnOnce(Span) -> Result<P, ParseError>,
{
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        let result = result?;
        if (self.cond)(&result) {
            Ok(result)
        } else {
            (self.or)(result.get_span())
        }
    }
}

impl<P, C, E> Satisfy<P, C, E>
where
    C: FnOnce(&P) -> bool,
    E: FnOnce(Span) -> Result<P, ParseError>,
{
    /// create a new [`Satisfy`] mapper
    ///
    /// * cond: condition
    ///
    /// * or: error generator for the value which not satisfy the condition
    pub fn new(cond: C, or: E) -> Self {
        Self {
            cond,
            or,
            _p: PhantomData,
        }
    }
}

/// try to parse
///
/// * [`ParseErrorKind::Unmatch`] will be transformed into [`None`],
///
/// * [`ParseErrorKind::Semantic`] will still be [`Err`]
///
/// * otherwith, [`Ok`] with [`Some`] in will be return
///
/// so that you can use `?` as usual after using match / if let ~
pub struct Try;

impl<P> ParseMapper<P> for Try {
    type Result = Result<Option<P>, ParseError>;

    #[inline]
    fn map(self, result: Result<P, ParseError>) -> Result<Option<P>, ParseError> {
        match result {
            Ok(p) => Ok(Some(p)),
            Err(e) if e.kind() == ParseErrorKind::Unmatch => Ok(None),
            Err(e) => Err(e),
        }
    }
}
