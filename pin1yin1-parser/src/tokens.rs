use crate::*;
use std::fmt::Debug;

/// an selection, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the selection, and the end of the selection
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, rhs: Selection) -> Self {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Selection::new(start, end)
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl WithSelection for Selection {
    fn get_selection(&self) -> Selection {
        *self
    }
}

/// a type which implemented [`ParseUnit<S>`] with source code it selected
pub struct PU<P: ParseUnit<S>, S: Copy = char> {
    pub(crate) selection: Selection,
    pub(crate) target: P::Target,
}

impl<P: ParseUnit<S>, S: Copy> WithSelection for PU<P, S> {
    fn get_selection(&self) -> Selection {
        self.selection
    }
}

impl<S: Copy, P: ParseUnit<S>> PU<P, S> {
    pub fn new(selection: Selection, inner: P::Target) -> Self {
        Self {
            selection,
            target: inner,
        }
    }

    /// take [ParseUnit::Target] from [`PU`]
    pub fn take(self) -> P::Target {
        self.target
    }

    /// map [ParseUnit::Target]
    pub fn map<P2: ParseUnit<S>, M>(self, mapper: M) -> PU<P2, S>
    where
        M: FnOnce(P::Target) -> P2::Target,
    {
        PU::new(self.selection, mapper(self.target))
    }
}

impl<S: Copy, P: ParseUnit<S>> Debug for PU<P, S>
where
    P::Target: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PU")
            .field("selection", &"...")
            .field("target", &self.target)
            .finish()
    }
}

impl<S: Copy, P: ParseUnit<S>> Clone for PU<P, S>
where
    P::Target: Clone,
{
    fn clone(&self) -> Self {
        Self {
            selection: self.selection,
            target: self.target.clone(),
        }
    }
}

impl<S: Copy, P: ParseUnit<S>> Copy for PU<P, S> where P::Target: Copy {}

impl<S: Copy, P: ParseUnit<S>> std::ops::Deref for PU<P, S> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<S: Copy, P: ParseUnit<S>> std::ops::DerefMut for PU<P, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.target
    }
}
