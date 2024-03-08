use crate::{parse_unit::ParseUnit, parser::Parser};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct Location<'s> {
    src: &'s [char],
    idx: usize,
}

impl<'s> Location<'s> {
    pub fn new(src: &'s [char], idx: usize) -> Self {
        Self { src, idx }
    }

    pub fn backtrace_line(&self) -> (usize, String) {
        let new_lines = (0..self.idx).filter(|idx| self.src[*idx] == '\n').count();
        let left = (0..self.idx)
            .rev()
            .find(|idx| self.src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let right = (self.idx..self.src.len())
            .find(|idx| self.src[*idx] == '\n')
            .unwrap_or(self.src.len());

        (new_lines + 1, self.src[left..right].iter().collect())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Selection<'s> {
    selections: &'s [char],
}

impl Selection<'_> {
    pub fn new(selections: &[char]) -> Selection<'_> {
        Selection { selections }
    }

    pub fn from_parser<'s>(parser: &Parser<'s>, start: Location) -> Selection<'s> {
        Selection::new(&parser.src[start.idx..parser.idx])
    }
}

impl std::ops::Deref for Selection<'_> {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        self.selections
    }
}

pub struct Token<'s, P: ParseUnit> {
    location: Location<'s>,
    selection: Selection<'s>,
    inner: P::Target<'s>,
}

impl<'s, P: ParseUnit> Token<'s, P> {
    pub fn new(location: Location<'s>, selection: Selection<'s>, inner: P::Target<'s>) -> Self {
        Self {
            location,
            selection,
            inner,
        }
    }

    pub fn location(&self) -> Location<'_> {
        self.location
    }
}

impl<'s, P: ParseUnit> Debug for Token<'s, P>
where
    P::Target<'s>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("location", &self.location)
            .field("selection", &self.selection)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'s, P: ParseUnit> Clone for Token<'s, P>
where
    P::Target<'s>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            location: self.location,
            selection: self.selection,
            inner: self.inner.clone(),
        }
    }
}

impl<'s, P: ParseUnit> Copy for Token<'s, P> where P::Target<'s>: Copy {}

impl<'s, P: ParseUnit> std::ops::Deref for Token<'s, P> {
    type Target = P::Target<'s>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: ParseUnit> std::ops::DerefMut for Token<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
