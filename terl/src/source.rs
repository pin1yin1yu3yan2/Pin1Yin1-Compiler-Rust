use std::ops::Index;

use crate::{Error, Message, Span};

#[derive(Debug, Clone)]
pub struct Source<S = char> {
    file_name: String,
    inner: Vec<S>,
}

impl<S> Source<S> {
    pub fn new(file_name: String, inner: Vec<S>) -> Self {
        Self { file_name, inner }
    }

    pub fn from_iter(file_name: impl ToString, iter: impl Iterator<Item = S>) -> Self {
        Self {
            file_name: file_name.to_string(),
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

use std::fmt::Write;
impl Source<char> {
    fn message(
        &self,
        buffer: &mut impl Write,
        span: Span,
        reason: String,
    ) -> Result<(), std::fmt::Error> {
        let src = self;
        let start_line_start = (0..span.start)
            .rev()
            .find(|idx| src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let mut line_num = (0..span.start).filter(|idx| src[*idx] == '\n').count() + 1;
        let mut idx = start_line_start;

        let row_num = span.start - start_line_start;
        let location = format!("[{}:{}:{}]", src.file_name(), line_num, row_num,);

        writeln!(buffer, "{location}: {}", reason)?;

        while idx < span.end && idx < src.len() {
            let line_start = idx;

            let head = format!("at line {line_num} | ");

            let mut space_len = head.len();
            while idx < span.start {
                space_len += 1;
                idx += 1;
            }
            let mut hats = (0..space_len).map(|_| ' ').collect::<String>();
            while idx < src.len() && src[idx] != '\n' {
                if idx < span.end {
                    hats.push('^');
                }
                idx += 1;
            }

            let line = src[line_start..idx].iter().collect::<String>();
            if !line.is_empty() {
                writeln!(buffer, "{head}{line}")?;
                writeln!(buffer, "{hats}")?;
            }

            idx = (idx + 1).min(src.len());
            line_num += 1;
        }
        Ok(())
    }

    fn handle_message(&self, buffer: &mut impl Write, msg: Message) -> Result<(), std::fmt::Error> {
        match msg {
            Message::Location(span) => self.message(buffer, span, String::new()),
            Message::Text(reason) => writeln!(buffer, "{reason}"),
            Message::Rich(reason, span) => self.message(buffer, span, reason),
        }?;

        Ok(())
    }

    pub fn handle_error(&self, error: Error) -> Result<String, std::fmt::Error> {
        let mut buffer = String::new();
        self.message(&mut buffer, error.main_span, error.main_message)?;
        for msg in error.messages {
            self.handle_message(&mut buffer, msg)?;
        }

        Ok(buffer)
    }
}
