use std::ops::Index;

use crate::{Error, Span};

#[derive(Debug, Clone)]
pub struct Source<S = char> {
    file_name: String,
    inner: Vec<S>,
}

impl<S> Source<S> {
    pub fn new(file_name: String, inner: Vec<S>) -> Self {
        Self { file_name, inner }
    }

    pub fn from_iter(file_name: impl Into<String>, iter: impl Iterator<Item = S>) -> Self {
        Self {
            file_name: file_name.into(),
            inner: iter.collect(),
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl<S> std::ops::Deref for Source<S> {
    type Target = [S];

    fn deref(&self) -> &Self::Target {
        &self.inner[..]
    }
}

impl<S> std::ops::Index<Span> for Source<S> {
    type Output = [S];

    fn index(&self, index: Span) -> &Self::Output {
        &self.inner[index.start..index.end]
    }
}

impl<I, S> std::ops::Index<I> for Source<S>
where
    Vec<S>: Index<I>,
{
    type Output = <Vec<S> as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.inner[index]
    }
}

impl Source<char> {
    pub fn handle_error(&self, error: Error) -> std::result::Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut buffer = String::new();

        let Error { span, reason, kind } = error;

        let src = self;

        let left = (0..span.start)
            .rev()
            .find(|idx| src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);

        let right = (span.start..src.len())
            .find(|idx| src[*idx] == '\n')
            .unwrap_or(src.len());

        let line_num = (0..span.start).filter(|idx| src[*idx] == '\n').count() + 1;

        let row_num = span.start - left;

        let line = src[left..right].iter().collect::<String>();

        let location = format!("[{}:{}:{}]", src.file_name(), line_num, row_num,);

        writeln!(buffer)?;
        // there is an error happend
        writeln!(buffer, "{location} {kind:?} Error: {}", reason)?;
        // at line {line_num}
        let head = format!("at line {line_num} | ");
        writeln!(buffer, "{head}{line}")?;
        let ahead = (0..head.len() + span.start - left)
            .map(|_| ' ')
            .collect::<String>();
        let point = (0..span.len()).map(|_| '^').collect::<String>();
        writeln!(buffer, "{ahead}{point}")?;
        Ok(buffer)
    }
}
