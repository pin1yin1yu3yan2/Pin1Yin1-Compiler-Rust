use crate::*;
use std::fmt::Debug;

/// an span, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the span, and the end of the span
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, rhs: Span) -> Self {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Span::new(start, end)
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}..{}", self.start, self.end)
    }
}

impl WithSpan for Span {
    fn get_span(&self) -> Span {
        *self
    }
}

/// a type which implemented [`ParseUnit<S>`] with source code it selected
pub struct PU<P: ParseUnit<S>, S = char> {
    pub(crate) span: Span,
    pub(crate) item: P::Target,
}

impl<P: ParseUnit<S>, S> WithSpan for PU<P, S> {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl<S, P: ParseUnit<S>> PU<P, S> {
    pub fn new(span: Span, item: P::Target) -> Self {
        Self { span, item }
    }

    /// take [ParseUnit::Target] from [`PU`]
    pub fn take(self) -> P::Target {
        self.item
    }

    /// map [ParseUnit::Target]
    pub fn map<P2: ParseUnit<S>, M>(self, mapper: M) -> PU<P2, S>
    where
        M: FnOnce(P::Target) -> P2::Target,
    {
        PU::new(self.span, mapper(self.item))
    }
}

impl<S, P: ParseUnit<S>> Debug for PU<P, S>
where
    P::Target: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PU")
            .field("span", &"...")
            .field("item", &self.item)
            .finish()
    }
}

impl<S, P: ParseUnit<S>> Clone for PU<P, S>
where
    P::Target: Clone,
{
    fn clone(&self) -> Self {
        Self {
            span: self.span,
            item: self.item.clone(),
        }
    }
}

impl<S, P: ParseUnit<S>> Copy for PU<P, S> where P::Target: Copy {}

impl<S, P: ParseUnit<S>> std::ops::Deref for PU<P, S> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<S, P: ParseUnit<S>> std::ops::DerefMut for PU<P, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}
