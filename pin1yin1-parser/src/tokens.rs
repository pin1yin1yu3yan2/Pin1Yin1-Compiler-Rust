use crate::*;
use std::fmt::Debug;

/// an selection, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the selection, and the length of the selection
///
/// as for [`serde`],,, we skip [`Selection`] now
#[derive(Debug, Clone, Copy)]
pub struct Selection<'s> {
    pub(crate) src: &'s [char],
    pub(crate) start: usize,
    pub(crate) len: usize,
}

impl<'s> Selection<'s> {
    pub fn new(src: &'s [char], start: usize, len: usize) -> Self {
        Self { src, start, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(feature = "ser")]
/// will and will only, should and should only be used in [`serde`]
impl Selection<'_> {
    const EMPTY_CHARS: &'static [char] = &[];
    fn empty() -> Selection<'static> {
        Selection::new(Self::EMPTY_CHARS, 0, 0)
    }
}

impl std::ops::Deref for Selection<'_> {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.src[self.start..self.start + self.len]
    }
}

/// a type which implemented [`ParseUnit`] with source code it selected
pub struct Token<'s, P: ParseUnit> {
    pub(crate) selection: Selection<'s>,
    pub(crate) target: P::Target<'s>,
}

impl<'s, P: ParseUnit> Token<'s, P> {
    pub fn new(selection: Selection<'s>, inner: P::Target<'s>) -> Self {
        Self {
            selection,
            target: inner,
        }
    }

    /// take [Self::target] from [`Token`]
    pub fn take(self) -> P::Target<'s> {
        self.target
    }

    /// try to map the [Self::target]
    pub fn try_map<P2: ParseUnit, M>(self, mapper: M) -> ParseResult<'s, P2>
    where
        M: FnOnce(P::Target<'s>) -> Result<'s, P2::Target<'s>>,
    {
        mapper(self.target).map(|target| Token::new(self.selection, target))
    }

    /// map [Self::target]
    pub fn map<P2: ParseUnit, M>(self, mapper: M) -> Token<'s, P2>
    where
        M: FnOnce(P::Target<'s>) -> P2::Target<'s>,
    {
        Token::new(self.selection, mapper(self.target))
    }

    /// Check if [Self::target] meets a certain criteria, or call error to generate an [`Error`]
    pub fn which_or<C, E>(self, criteria: C, error: E) -> ParseResult<'s, P>
    where
        C: FnOnce(&P::Target<'s>) -> bool,
        E: FnOnce(Self) -> ParseResult<'s, P>,
    {
        if criteria(&*self) {
            Ok(self)
        } else {
            error(self)
        }
    }

    /// Check if [Self::target] meets a certain criteria, or generate an [`Err`] with [`None`] in it
    pub fn which<C>(self, criteria: C) -> ParseResult<'s, P>
    where
        C: FnOnce(&P::Target<'s>) -> bool,
    {
        self.which_or(criteria, |_| Err(None))
    }

    /// Check if [Self::target] equals to the given value, or call error to generate an [`Error`]
    pub fn is_or<E>(self, rhs: P::Target<'s>, e: E) -> ParseResult<'s, P>
    where
        P::Target<'s>: PartialEq,
        E: FnOnce(Self) -> ParseResult<'s, P>,
    {
        self.which_or(|t| t == &rhs, e)
    }

    /// Check if [Self::target] equals to the given value, or generate an [`Err`] with [`None`] in it
    pub fn is(self, rhs: P::Target<'s>) -> ParseResult<'s, P>
    where
        P::Target<'s>: PartialEq,
    {
        self.which(|t| t == &rhs)
    }

    /// generate an [`Error`] with [`Self::selection`]
    pub fn new_error(&self, reason: impl Into<String>) -> Error<'s> {
        Error::new(self.selection, reason.into())
    }

    /// generate an [`Result`] with an actual [`Error`] in it
    pub fn throw<P1: ParseUnit>(&self, reason: impl Into<String>) -> ParseResult<'s, P1> {
        Err(Some(self.new_error(reason)))
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

#[cfg(feature = "ser")]
impl<'s, P: ParseUnit> serde::Serialize for Token<'s, P>
where
    P::Target<'s>: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.target.serialize(serializer)
    }
}

#[cfg(feature = "ser")]
impl<'s, P: ParseUnit> serde::Deserialize<'s> for Token<'s, P>
where
    P::Target<'s>: serde::Deserialize<'s>,
{
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'s>,
    {
        let taregt = P::Target::deserialize(deserializer)?;
        Ok(Self::new(Selection::empty(), taregt))
    }
}
