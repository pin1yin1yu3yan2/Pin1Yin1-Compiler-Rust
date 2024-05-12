use crate::*;

/// an span, means some characters that are selected in the source code
///
/// be different from &[char], this type contains
/// two data: the start of the span, and the end of the span
#[derive(Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, rhs: Span) -> Self {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Span::new(start, end)
    }

    pub fn sub_set(self, rhs: Span) -> Self {
        let start = self.start.max(rhs.start);
        let end = self.end.min(rhs.end);
        assert!(start < end, "those two span have no subset");

        Span::new(start, end)
    }

    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::ops::Add for Span {
    type Output = Span;

    fn add(self, rhs: Self) -> Self::Output {
        self.merge(rhs)
    }
}

impl std::ops::Mul for Span {
    type Output = Span;

    fn mul(self, rhs: Self) -> Self::Output {
        self.sub_set(rhs)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}..{}", self.start, self.end)
    }
}

impl WithSpan for Span {
    fn get_span(&self) -> Span {
        *self
    }
}
