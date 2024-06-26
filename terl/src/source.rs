use std::{fmt::Write, ops::Index};

use crate::*;

/// the source of [`Parser`]
///
/// Indicate how to format the error to String
pub trait Source: Sized {
    /// type is needed for [`Source::handle_error`]
    type HandleErrorWith<'b>;
    /// handle an [`Error`] by formatting it into a string
    ///
    /// This function formats an error into a string. It iterates through the error's messages,
    /// formatting each one and appending it to the `buffer`. If any formatting fails,
    /// it returns an error.
    ///
    /// # Parameters
    ///
    /// * `with` - A reference to the [`Source::HandleErrorWith`], which is used to format the error.
    /// * `error` - The error to be formatted and displayed.
    ///
    /// # Returns
    ///
    /// A [`String`] containing the formatted error message if successful, or panic if the formatting fails.
    fn handle_error(with: &Self::HandleErrorWith<'_>, error: Error) -> String {
        (|| -> Result<_, std::fmt::Error> {
            let mut buffer = String::new();
            for msg in error.messages {
                Self::handle_message(with, &mut buffer, msg)?;
            }
            Ok(buffer)
        })()
        .unwrap()
    }

    /// Handle a message by formatting it into a string.
    ///
    /// This function formats a message into a string. It handles different types of messages and formats them accordingly.
    ///
    /// # Parameters
    ///
    /// * `with` - A reference to the [`Source::HandleErrorWith`], which is used to format the error.
    /// * `buffer` - A mutable reference to a [`Write`] trait object, which is used to write the formatted error message.
    /// * `message` - The message to be formatted and displayed.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the formatted error message as a [`String`] if successful, or an [`std::fmt::Error`] if the formatting fails.
    fn handle_message<S>(
        with: &Self::HandleErrorWith<'_>,
        buffer: &mut S,
        message: Message,
    ) -> std::fmt::Result
    where
        S: Write,
    {
        match message {
            Message::Location(span) => Self::handle_location(with, buffer, span, ""),
            Message::Text(reason) => writeln!(buffer, "{reason}"),
            Message::Rich(reason, span) => Self::handle_location(with, buffer, span, &reason),
        }?;
        Ok(())
    }

    /// Formats a message on given location to [`String`].
    ///
    /// This function formats a message on the given location to a string. It handles different types of messages and formats them accordingly.
    ///
    /// # Parameters
    ///
    /// * `with` - A reference to the [`Source::HandleErrorWith`] trait object, which is used to format the error.
    /// * `buffer` - A mutable reference to a [`Write`] trait object, which is used to write the formatted error message.
    /// * `loc` - The [`Span`] representing the location where the error occurred.
    /// * `msg` - The message to be formatted and displayed.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the formatted error message as a [`String`] if successful, or an [`std::fmt::Error`] if the formatting fails.
    fn handle_location<S>(
        with: &Self::HandleErrorWith<'_>,
        buffer: &mut S,
        loc: Span,
        msg: &str,
    ) -> std::fmt::Result
    where
        S: Write;
}

impl Source for char {
    type HandleErrorWith<'b> = Buffer<char>;

    #[cfg(not(feature = "color"))]
    fn handle_location<S>(
        with: &Buffer<char>,
        buffer: &mut S,
        span: Span,
        msg: &str,
    ) -> std::fmt::Result
    where
        S: Write,
    {
        let src = with;
        let start_line_start = (0..span.start)
            .rev()
            .find(|idx| src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let mut line_num = (0..span.start).filter(|idx| src[*idx] == '\n').count() + 1;
        let mut idx = start_line_start;

        let row_num = span.start - start_line_start + 1;
        let location = format!("[{}:{}:{}]", src.name(), line_num, row_num,);

        writeln!(buffer, "{location}: {}", msg)?;

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

    #[cfg(feature = "color")]
    fn handle_location<S>(
        src: &Self::HandleErrorWith<'_>,
        buffer: &mut S,
        loc: Span,
        msg: &str,
    ) -> std::fmt::Result
    where
        S: Write,
    {
        let start_line_start = (0..loc.start)
            .rev()
            .find(|idx| src[*idx] == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let mut line_num = (0..loc.start).filter(|idx| src[*idx] == '\n').count() + 1;
        let mut idx = start_line_start;

        let row_num = loc.start - start_line_start + 1;
        let location = format!("[{}:{}:{}]", src.name(), line_num, row_num,);

        writeln!(buffer, "{location}: {}", msg)?;
        while idx < loc.end && idx < src.len() {
            buffer.write_fmt(format_args!("at line {line_num} | "))?;
            let end = (idx..src.len())
                .find(|pos| src[*pos] == '\n')
                .unwrap_or(src.len());
            use colored::Colorize;

            if loc.start > idx {
                for idx in idx..loc.start {
                    buffer.write_char(src[idx])?;
                }
            }

            let error = (loc.start.max(idx)..end.min(loc.end))
                .map(|pos| src[pos])
                .collect::<String>()
                .red()
                .underline();

            // use foramt, or the output willnot be colored
            buffer.write_fmt(format_args!("{error}"))?;
            if loc.end < end - 1 {
                for idx in loc.end..end {
                    buffer.write_char(src[idx])?;
                }
            }
            idx = end;
            line_num -= 1;
            writeln!(buffer)?;
        }
        Ok(())
    }
}

/// a buffer,store source whihch is needed by [`Parser`] in
#[derive(Debug, Clone)]
pub struct Buffer<S = char> {
    name: String,
    src: Vec<S>,
}

impl<S> Buffer<S> {
    /// create a new [`Buffer`]
    pub fn new(name: String, src: Vec<S>) -> Self {
        Self { name, src }
    }

    /// Returns the name of the source.
    ///
    /// # Returns
    ///
    /// * `&str` - A reference to the name of the source.
    pub fn name(&self) -> &str {
        &self.name
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
