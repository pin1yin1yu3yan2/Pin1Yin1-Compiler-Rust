use std::marker::PhantomData;

use crate::*;

pub trait ExtendTuple {
    type Next<T>: ExtendTuple;
    fn extend_one<T>(self, append: T) -> Self::Next<T>;
}

pub trait ParseMapper<P> {
    fn mapper(self, result: Result<P>) -> Result<P>;
}

#[derive(Debug, Clone, Copy)]
pub struct MapError<M>(M)
where
    M: FnOnce(Error) -> Error;

impl<P, M> ParseMapper<P> for MapError<M>
where
    M: FnOnce(Error) -> Error,
{
    fn mapper(self, result: Result<P>) -> Result<P> {
        result.map_err(self.0)
    }
}

impl<M> MapError<M>
where
    M: FnOnce(Error) -> Error,
{
    pub fn new(m: M) -> Self {
        Self(m)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MustMatch;

impl<P> ParseMapper<P> for MustMatch {
    fn mapper(self, result: Result<P>) -> Result<P> {
        result.map_err(|e| {
            if e.kind() == ErrorKind::Unmatch {
                e.to_error()
            } else {
                e
            }
        })
    }
}

pub struct MapMsg<S>(S)
where
    S: ToString;

impl<P: ParseUnit<Src>, Src, S> ParseMapper<PU<P, Src>> for MapMsg<S>
where
    S: ToString,
{
    fn mapper(self, result: Result<PU<P, Src>>) -> Result<PU<P, Src>> {
        result.map_err(|e| e.map(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct Equal<P: ParseUnit<S>, S, E>
where
    P::Target: PartialEq,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    eq: P::Target,
    or: E,
}

impl<P: ParseUnit<S>, S, E> ParseMapper<PU<P, S>> for Equal<P, S, E>
where
    P::Target: PartialEq,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    fn mapper(self, result: ParseResult<P, S>) -> ParseResult<P, S> {
        let result = result?;
        if *result == self.eq {
            Ok(result)
        } else {
            (self.or)(result.get_span())
        }
    }
}

impl<P: ParseUnit<S>, S, E> Equal<P, S, E>
where
    P::Target: PartialEq,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    pub fn new(eq: P::Target, or: E) -> Self {
        Self { eq, or }
    }
}

pub struct Satisfy<P: ParseUnit<S>, S, C, E>
where
    C: FnOnce(&P::Target) -> bool,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    cond: C,
    or: E,
    _p: PhantomData<(P, S)>,
}

impl<P: ParseUnit<S>, S, C, E> ParseMapper<PU<P, S>> for Satisfy<P, S, C, E>
where
    C: FnOnce(&P::Target) -> bool,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    fn mapper(self, result: ParseResult<P, S>) -> ParseResult<P, S> {
        let result = result?;
        if (self.cond)(&result) {
            Ok(result)
        } else {
            (self.or)(result.get_span())
        }
    }
}

impl<P: ParseUnit<S>, S, C, E> Satisfy<P, S, C, E>
where
    C: FnOnce(&P::Target) -> bool,
    E: FnOnce(Span) -> ParseResult<P, S>,
{
    pub fn new(cond: C, or: E) -> Self {
        Self {
            cond,
            or,
            _p: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Custom<P, M>
where
    M: FnOnce(Result<P>) -> Result<P>,
{
    mapper: M,
    _p: PhantomData<P>,
}

impl<P, M> ParseMapper<P> for Custom<P, M>
where
    M: FnOnce(Result<P>) -> Result<P>,
{
    fn mapper(self, result: Result<P>) -> Result<P> {
        (self.mapper)(result)
    }
}

impl<P, M> Custom<P, M>
where
    M: FnOnce(Result<P>) -> Result<P>,
{
    pub fn new(mapper: M) -> Self {
        Self {
            mapper,
            _p: PhantomData,
        }
    }
}
