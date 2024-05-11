use std::ops::Index;

use crate::{Error, Message, Span};

pub trait Source: Sized {
    type HandleErrorWith;
    fn handle_error(with: &Self::HandleErrorWith, error: Error) -> String;
}

impl Source for char {
    type HandleErrorWith = Buffer<char>;

    fn handle_error(src_buffer: &Buffer<Self>, error: Error) -> String {
        (|| {
            let mut buffer = String::new();
            src_buffer.message(&mut buffer, error.main_span, error.main_message)?;
            for msg in error.messages {
                src_buffer.handle_message(&mut buffer, msg)?;
            }

            Result::<_, std::fmt::Error>::Ok(buffer)
        })()
        .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Buffer<S = char> {
    name: String,
    src: Vec<S>,
}

impl<S> Buffer<S> {
    pub fn new(name: String, src: Vec<S>) -> Self {
        Self { name, src }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn handle_error(&self, error: Error) -> String
    where
        S: Source<HandleErrorWith = Self>,
    {
        S::handle_error(self, error)
    }
}

impl<S> std::ops::Deref for Buffer<S> {
    type Target = [S];

    fn deref(&self) -> &Self::Target {
        &self.src[..]
    }
}

impl<S> std::ops::Index<Span> for Buffer<S> {
    type Output = [S];

    fn index(&self, index: Span) -> &Self::Output {
        &self.src[index.start..index.end]
    }
}

impl<I, S> std::ops::Index<I> for Buffer<S>
where
    Vec<S>: Index<I>,
{
    type Output = <Vec<S> as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.src[index]
    }
}

use std::fmt::Write;

impl Buffer<char> {
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
        let location = format!("[{}:{}:{}]", src.name(), line_num, row_num,);

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
}
