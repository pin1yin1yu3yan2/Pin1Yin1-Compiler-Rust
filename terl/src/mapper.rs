use std::marker::PhantomData;

use crate::*;

pub trait ExtendTuple {
    type Next<T>: ExtendTuple;
    fn extend_one<T>(self, append: T) -> Self::Next<T>;
}

pub trait ParseMapper<P> {
    type Result;
    fn map(self, result: Result<P, ParseError>) -> Self::Result;
}

pub trait ResultMapperExt<P> {
    type Result<Mapper: ParseMapper<P>>;
    fn apply<M: ParseMapper<P>>(self, mapper: M) -> Self::Result<M>;
}

impl<P> ResultMapperExt<P> for Result<P, ParseError> {
    type Result<Mapper: ParseMapper<P>> = Mapper::Result;

    fn apply<M: ParseMapper<P>>(self, mapper: M) -> Self::Result<M> {
        mapper.map(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapError<M>(M)
where
    M: FnOnce(ParseError) -> ParseError;

impl<P, M> ParseMapper<P> for MapError<M>
where
    M: FnOnce(ParseError) -> ParseError,
{
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        result.map_err(self.0)
    }
}

impl<M> MapError<M>
where
    M: FnOnce(ParseError) -> ParseError,
{
    pub fn new(m: M) -> Self {
        Self(m)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MustMatch;

impl<P> ParseMapper<P> for MustMatch {
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        result.map_err(|e| {
            if e.kind() == ParseErrorKind::Unmatch {
                e.to_error()
            } else {
                e
            }
        })
    }
}

pub struct MapMsg<Msg>(pub Msg)
where
    Msg: ToString;

impl<P, Msg> ParseMapper<P> for MapMsg<Msg>
where
    Msg: ToString,
{
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        result.map_err(|e| e.map(self.0))
    }
}

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
    pub fn new(eq: P, or: E) -> Self {
        Self { eq, or }
    }
}

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

#[derive(Debug, Clone)]
pub struct Custom<P, M>
where
    M: FnOnce(Result<P, ParseError>) -> Result<P, ParseError>,
{
    mapper: M,
    _p: PhantomData<P>,
}

impl<P, M> ParseMapper<P> for Custom<P, M>
where
    M: FnOnce(Result<P, ParseError>) -> Result<P, ParseError>,
{
    type Result = Result<P, ParseError>;

    fn map(self, result: Result<P, ParseError>) -> Result<P, ParseError> {
        (self.mapper)(result)
    }
}

impl<P, M> Custom<P, M>
where
    M: FnOnce(Result<P, ParseError>) -> Result<P, ParseError>,
{
    pub fn new(mapper: M) -> Self {
        Self {
            mapper,
            _p: PhantomData,
        }
    }
}
