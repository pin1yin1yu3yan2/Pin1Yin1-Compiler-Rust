use crate::*;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct Location<'s> {
    pub src: &'s [char],
    pub idx: usize,
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
    src: &'s [char],
    start: usize,
    len: usize,
}

impl<'s> Selection<'s> {
    pub fn new(location: Location, len: usize) -> Selection<'_> {
        Selection {
            src: location.src,
            start: location.idx,
            len,
        }
    }

    pub fn from_parser(parser: &Parser, start: Location<'s>) -> Selection<'s> {
        Selection::new(start, parser.idx - start.idx)
    }

    pub fn location(&self) -> Location<'_> {
        Location::new(self.src, self.start)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn throw(&self, reason: impl Into<String>) -> Result<'s, ()> {
        Err(Some(Error::new(*self, reason.into())))
    }

    /// its very ugly to use this method!
    fn to_selections(self) -> Selections<'s> {
        Selections::new(self, Vec::with_capacity(0))
    }

    pub fn parse<P: ParseUnit>(&self) -> ParseResult<'s, P> {
        P::generate(&self.to_selections()).map(|target| Token::new(*self, target))
    }
}

impl std::ops::Deref for Selection<'_> {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.src[self.start..self.start + self.len]
    }
}

pub struct Token<'s, P: ParseUnit> {
    selection: Selection<'s>,
    target: P::Target<'s>,
}

impl<'s, P: ParseUnit> Token<'s, P> {
    pub fn new(selection: Selection<'s>, inner: P::Target<'s>) -> Self {
        Self {
            selection,
            target: inner,
        }
    }

    pub fn location(&self) -> Location<'_> {
        Location::new(self.selection.src, self.selection.start)
    }
}

impl<'s, P: ParseUnit> Debug for Token<'s, P>
where
    P::Target<'s>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("selection", &"...")
            .field("target", &self.target)
            .finish()
    }
}

impl<'s, P: ParseUnit> Clone for Token<'s, P>
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

impl<'s, P: ParseUnit> Copy for Token<'s, P> where P::Target<'s>: Copy {}

impl<'s, P: ParseUnit> std::ops::Deref for Token<'s, P> {
    type Target = P::Target<'s>;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<P: ParseUnit> std::ops::DerefMut for Token<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.target
    }
}
