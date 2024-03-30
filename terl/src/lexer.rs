use std::fmt::Debug;

use crate::Source;

pub struct Lexer<S, R> {
    source: Source<S>,
    result: Source<R>,
}

impl<S, R> Lexer<S, R> {
    pub fn new(source: impl Into<Source<S>>) -> Self {
        let source = source.into();
        Self {
            result: Source::from_iter(source.file_name(), std::iter::empty()),
            source,
        }
    }
}

impl<S: Debug, R: Debug> Debug for Lexer<S, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("source", &self.source)
            .field("result", &self.result)
            .finish()
    }
}

impl<S: Clone, R: Clone> Clone for Lexer<S, R> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            result: self.result.clone(),
        }
    }
}
