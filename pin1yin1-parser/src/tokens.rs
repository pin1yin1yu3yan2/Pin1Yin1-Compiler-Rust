use crate::*;
use std::fmt::Debug;

/// an selection, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the selection, and the end of the selection
///
/// as for [`serde`],,, we skip [`Selection`] now
#[derive(Debug, Clone, Copy)]
pub struct Selection<'s, S: Copy = char> {
    pub(crate) src: &'s Source<S>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<'s, S: Copy> Selection<'s, S> {
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

impl<'s, S: Copy> WithSelection<'s, S> for Selection<'s, S> {
    fn get_selection(&self) -> Selection<'s, S> {
        *self
    }
}

/// a type which implemented [`ParseUnit<S>`] with source code it selected
pub struct PU<'s, P: ParseUnit<S>, S: Copy = char> {
    pub(crate) selection: Selection<'s, S>,
    pub(crate) target: P::Target<'s>,
}

impl<'s, P: ParseUnit<S>, S: Copy> WithSelection<'s, S> for PU<'s, P, S> {
    fn get_selection(&self) -> Selection<'s, S> {
        self.selection
    }
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

    /// map [Self::target]
    pub fn map<P2: ParseUnit<S>, M>(self, mapper: M) -> PU<'s, P2, S>
    where
        M: FnOnce(P::Target<'s>) -> P2::Target<'s>,
    {
        PU::new(self.selection, mapper(self.target))
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
