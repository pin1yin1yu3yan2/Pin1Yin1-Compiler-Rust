use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Parser<'s> {
    // src
    pub(crate) src: &'s [char],
    pub(crate) idx: usize,
    // parse state
    pub(crate) start_idx: usize,
    // used to compute start_idx
    pub(crate) tries: usize,
    pub(crate) done_tries: usize,

    // for &[char]::parse
    pub(crate) chars_cache: &'s [char],
    pub(crate) chars_cache_idx: usize,
    pub(crate) chars_cache_final: usize,
}

impl Parser<'_> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn next(&mut self) -> Option<char> {
        let next = self.src.get(self.idx).copied()?;
        self.idx += 1;
        Some(next)
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.src.get(self.idx).copied()
    }

    // pub(crate) fn next_back(&mut self) -> Option<char> {
    //     if self.idx == 0 {
    //         return None;
    //     }
    //     self.idx -= 1;
    //     Some(self.src[self.idx])
    // }
}

impl<'s> Parser<'s> {
    pub fn new(src: &[char]) -> Parser<'_> {
        Parser {
            src,
            idx: 0,
            start_idx: 0,
            tries: 0,
            done_tries: 0,
            chars_cache: &src[..0],
            chars_cache_idx: usize::MAX,
            chars_cache_final: usize::MAX,
        }
    }

    pub fn r#try<'p, F, P>(&'p mut self, p: F) -> Try<'p, 's, P>
    where
        P: ParseUnit,
        F: FnOnce(&mut Parser<'s>) -> ParseResult<'s, P>,
    {
        Try::new(self).or_try(p)
    }

    pub fn parse<P: ParseUnit>(&mut self) -> ParseResult<'s, P> {
        P::parse(self)
    }

    pub fn try_parse<'p, P: ParseUnit>(&'p mut self) -> ParseResult<'s, P> {
        Try::new(self).or_try(P::parse).no_error().finish()
    }

    pub fn skip_while<Rule>(&mut self, rule: Rule) -> &mut Self
    where
        Rule: Fn(char) -> bool,
    {
        while self.peek().is_some_and(&rule) {
            self.next();
        }
        self
    }

    pub fn skip_whitespace(&mut self) -> &mut Self {
        self.skip_while(|c| c.is_ascii_whitespace());
        self
    }

    pub fn take_while<Rule>(&mut self, rule: Rule) -> &'s [char]
    where
        Rule: Fn(char) -> bool,
    {
        let start = self.idx;
        self.skip_while(&rule);
        &self.src[start..self.idx]
    }

    pub fn selection(&self) -> Selection<'s> {
        Selection::new(self.src, self.start_idx, self.idx - self.start_idx)
    }

    pub fn new_token<P: ParseUnit>(&self, t: P::Target<'s>) -> Token<'s, P> {
        Token::new(self.selection(), t)
    }

    pub fn finish<P: ParseUnit>(&self, t: P::Target<'s>) -> ParseResult<'s, P> {
        Ok(self.new_token(t))
    }

    pub fn new_error(&mut self, reason: impl Into<String>) -> Error<'s> {
        Error::new(self.selection(), reason.into())
    }

    pub fn throw(&mut self, reason: impl Into<String>) -> Result<'s, ()> {
        Err(Some(self.new_error(reason)))
    }
}

pub struct Try<'p, 's, P: ParseUnit> {
    parser: &'p mut Parser<'s>,
    state: Option<std::result::Result<Token<'s, P>, Error<'s>>>,
}

impl<'p, 's, P: ParseUnit> Try<'p, 's, P> {
    pub fn new(parser: &'p mut Parser<'s>) -> Self {
        parser.tries += 1;
        Self {
            parser,
            state: None,
        }
    }

    pub fn or_try<P1, F>(mut self, p: F) -> Self
    where
        P1: ParseUnit<Target<'s> = P::Target<'s>>,
        F: FnOnce(&mut Parser<'s>) -> ParseResult<'s, P1>,
    {
        if self.state.is_some() {
            return self;
        }

        let mut tmp = *self.parser;
        tmp.start_idx = tmp.idx;

        match p(&mut tmp) {
            Ok(r) => {
                self.state = Some(Ok(Token::new(r.selection, r.target)));
                self.parser.idx = tmp.idx;
                if self.parser.done_tries != self.parser.tries {
                    self.parser.start_idx = tmp.start_idx;
                    self.parser.done_tries = self.parser.tries;
                }
            }
            Err(opte) => {
                if let Some(e) = opte {
                    self.state = Some(Err(e))
                } else {
                    // synchron
                    self.parser.chars_cache = tmp.chars_cache;
                    self.parser.chars_cache_idx = tmp.chars_cache_idx;
                    self.parser.chars_cache_final = tmp.chars_cache_final;
                }
            }
        }

        self
    }

    pub fn or_error(mut self, reason: impl Into<String>) -> Self {
        self.state = self
            .state
            .or_else(|| Some(Err(self.parser.new_error(reason))));
        self
    }

    pub fn no_error(self) -> Self {
        self.or_error("no error")
    }

    pub fn finish(self) -> ParseResult<'s, P> {
        self.state.expect("uncatched error").map_err(Some)
    }
}
