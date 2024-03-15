use crate::*;
use std::fmt::Debug;

/// an selection, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the selection, and the end of the selection
///
/// as for [`serde`],,, we skip [`Selection`] now
#[derive(Debug, Clone, Copy)]
pub struct Selection<'s, S = char> {
    pub(crate) src: &'s Source<S>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<'s, S> Selection<'s, S> {
    pub fn new(src: &'s Source<S>, start: usize, end: usize) -> Self {
        Self { src, start, end }
    }

    pub fn merge(self, rhs: Selection<'s, S>) -> Self {
        if !(self.src.as_ptr() == rhs.src.as_ptr() && self.src.len() == rhs.src.len()) {
            panic!("invalid merge")
        }
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Selection::new(self.src, start, end)
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::ops::Deref for Selection<'_> {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.src[self.start..self.end]
    }
}

/// a type which implemented [`ParseUnit<S>`] with source code it selected
pub struct PU<'s, P: ParseUnit<S>, S: Copy = char> {
    pub(crate) selection: Selection<'s, S>,
    pub(crate) target: P::Target<'s>,
}

impl<'s, S: Copy, P: ParseUnit<S>> PU<'s, P, S> {
    pub fn new(selection: Selection<'s, S>, inner: P::Target<'s>) -> Self {
        Self {
            selection,
            target: inner,
        }
    }

    /// take [Self::target] from [`PU`]
    pub fn take(self) -> P::Target<'s> {
        self.target
    }

    pub fn selection(&self) -> Selection<'s, S> {
        self.selection
    }

    /// try to map the [Self::target]
    pub fn try_map<P2: ParseUnit<S>, M>(self, mapper: M) -> ParseResult<'s, P2, S>
    where
        M: FnOnce(P::Target<'s>) -> Result<'s, P2::Target<'s>, S>,
    {
        mapper(self.target).map(|target| PU::new(self.selection, target))
    }

    /// map [Self::target]
    pub fn map<P2: ParseUnit<S>, M>(self, mapper: M) -> PU<'s, P2, S>
    where
        M: FnOnce(P::Target<'s>) -> P2::Target<'s>,
    {
        PU::new(self.selection, mapper(self.target))
    }

    /// Check if [Self::target] meets a certain criteria, or call error to generate an [`Error`]
    pub fn which_or<C, E>(self, criteria: C, error: E) -> ParseResult<'s, P, S>
    where
        C: FnOnce(&P::Target<'s>) -> bool,
        E: FnOnce(Self) -> ParseResult<'s, P, S>,
    {
        if criteria(&*self) {
            Ok(self)
        } else {
            error(self)
        }
    }

    /// Check if [Self::target] meets a certain criteria, or generate an [`Err`] with [`None`] in it
    pub fn which<C>(self, criteria: C) -> ParseResult<'s, P, S>
    where
        C: FnOnce(&P::Target<'s>) -> bool,
    {
        self.which_or(criteria, |_| Err(None))
    }

    /// Check if [Self::target] equals to the given value, or call error to generate an [`Error`]
    pub fn is_or<E>(self, rhs: P::Target<'s>, e: E) -> ParseResult<'s, P, S>
    where
        P::Target<'s>: PartialEq,
        E: FnOnce(Self) -> ParseResult<'s, P, S>,
    {
        self.which_or(|t| t == &rhs, e)
    }

    /// Check if [Self::target] equals to the given value, or generate an [`Err`] with [`None`] in it
    pub fn is(self, rhs: P::Target<'s>) -> ParseResult<'s, P, S>
    where
        P::Target<'s>: PartialEq,
    {
        self.which(|t| t == &rhs)
    }

    /// generate an [`Error`] with [`Self::selection`]
    pub fn new_error(&self, reason: impl Into<String>) -> Error<'s, S> {
        Error::new(self.selection, reason.into())
    }

    /// generate an [`Result`] with an actual [`Error`] in it
    pub fn throw<P1: ParseUnit<S>>(&self, reason: impl Into<String>) -> ParseResult<'s, P1, S> {
        Err(Some(self.new_error(reason)))
    }
}

impl<'s, S: Copy, P: ParseUnit<S>> Debug for PU<'s, P, S>
where
    P::Target<'s>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PU")
            .field("selection", &"...")
            .field("target", &self.target)
            .finish()
    }
}

impl<'s, S: Copy, P: ParseUnit<S>> Clone for PU<'s, P, S>
where
    P::Target<'s>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            selection: self.selection,
            target: self.target.clone(),
        }
    }
}

impl<'s, S: Copy, P: ParseUnit<S>> Copy for PU<'s, P, S> where P::Target<'s>: Copy {}

impl<'s, S: Copy, P: ParseUnit<S>> std::ops::Deref for PU<'s, P, S> {
    type Target = P::Target<'s>;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<S: Copy, P: ParseUnit<S>> std::ops::DerefMut for PU<'_, P, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.target
    }
}
