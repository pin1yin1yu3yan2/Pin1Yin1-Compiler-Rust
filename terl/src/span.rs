use crate::*;

/// An `Span` represents a range of items in the source code.
///
/// It contains two data: the start of the span, and the end of the span.
///
/// # Examples
///
/// ```no_run
/// use terl::Span;
///
/// let span = Span::new(3, 7);
/// assert_eq!(span.len(), 4);
/// assert_eq!(span.is_empty(), false);
/// assert_eq!(format!("{:?}", span), "@3..7");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
}

impl Span {
    /// Creates a new `Span` with the given start and end positions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span = Span::new(3, 7);
    /// assert_eq!(span.len(), 4);
    /// assert_eq!(span.is_empty(), false);
    /// assert_eq!(format!("{:?}", span), "@3..7");
    /// ```
    ///
    /// # Panic
    ///
    /// panic if the start of the span is after the end of the span
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    /// Merges this span with another span, returning a new span that includes all items from both spans.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span1 = Span::new(1, 5);
    /// let span2 = Span::new(3, 7);
    /// assert_eq!(span1.merge(span2), Span::new(1, 7));
    /// ```
    pub fn merge(self, rhs: Span) -> Self {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);

        Span::new(start, end)
    }

    /// Returns a new span that includes only the items from this span that are also in the given span.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span1 = Span::new(1, 5);
    /// let span2 = Span::new(3, 7);
    /// assert_eq!(span1.subset(span2), Span::new(3, 5));
    /// ```
    pub fn subset(self, rhs: Span) -> Self {
        let start = self.start.max(rhs.start);
        let end = self.end.min(rhs.end);
        assert!(start < end, "those two span have no subset");

        Span::new(start, end)
    }

    /// Returns the length of this span, which is the difference between the end and start positions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span = Span::new(3, 7);
    /// assert_eq!(span.len(), 4);
    /// ```
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns `true` if this span is empty, i.e., if its start and end positions are the same.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span = Span::new(3, 3);
    /// assert_eq!(span.is_empty(), true);
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// alias for [`Span::merge`]
impl std::ops::Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.merge(rhs)
    }
}

/// alias for [`Span::subset`]
impl std::ops::Mul for Span {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.subset(rhs)
    }
}

impl std::fmt::Debug for Span {
    /// Formats a `Span` as a string in the format "@start..end".
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use terl::Span;
    ///
    /// let span = Span::new(3, 7);
    /// assert_eq!(format!("{:?}", span), "@3..7");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}..{}", self.start, self.end)
    }
}

impl WithSpan for Span {
    fn get_span(&self) -> Span {
        *self
    }
}
