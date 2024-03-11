use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Parser<'s> {
    // src
    pub(crate) src: &'s [char],
    pub(crate) idx: usize,
    // parse state
    start_idx: Option<usize>,
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

    pub(crate) fn start_taking(&mut self) {
        self.start_idx = Some(self.start_idx.unwrap_or(self.idx));
    }

    pub(crate) fn start_idx(&self) -> usize {
        self.start_idx.unwrap()
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
            start_idx: None,
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

    pub fn try_once<P, F>(&mut self, parser: F) -> ParseResult<'s, P>
    where
        P: ParseUnit,
        F: FnOnce(&mut Parser<'s>) -> ParseResult<'s, P>,
    {
        let mut tmp = *self;

        // reset state
        tmp.start_idx = None;
        let result = parser(&mut tmp);

        match &result {
            // if success,
            Ok(..) => {
                // foward tmp parser's work to main parser
                self.idx = tmp.idx;
                self.start_idx = self.start_idx.or(tmp.start_idx);

                // use to skip repeated parse
                if self.done_tries != self.tries {
                    self.done_tries = self.tries;
                }
            }
            Err(opte) => {
                // fault
                if opte.is_some() {
                    // foward tmp parser's work to main parser
                    self.idx = tmp.idx;
                } else {
                    // synchron try cache (for &[char]::parse)
                    self.chars_cache = tmp.chars_cache;
                    self.chars_cache_idx = tmp.chars_cache_idx;
                    self.chars_cache_final = tmp.chars_cache_final;
                }
            }
        }

        result
    }

    /// make effort if success or no error, make no effort if failure
    pub fn parse<P: ParseUnit>(&mut self) -> ParseResult<'s, P> {
        self.try_once(P::parse)
    }

    pub(crate) fn skip_while<Rule>(&mut self, rule: Rule) -> &mut Self
    where
        Rule: Fn(char) -> bool,
    {
        while self.peek().is_some_and(&rule) {
            self.next();
        }
        self
    }

    pub(crate) fn skip_whitespace(&mut self) -> &mut Self {
        self.skip_while(|c| c.is_ascii_whitespace());
        self
    }

    pub(crate) fn take_while<Rule>(&mut self, rule: Rule) -> &'s [char]
    where
        Rule: Fn(char) -> bool,
    {
        self.start_taking();
        self.skip_while(&rule);
        &self.src[self.start_idx.unwrap()..self.idx]
    }

    pub(crate) fn selection(&self) -> Selection<'s> {
        Selection::new(self.src, self.start_idx(), self.idx - self.start_idx())
    }

    pub fn new_token<I: Into<P::Target<'s>>, P: ParseUnit>(&self, t: I) -> Token<'s, P> {
        Token::new(self.selection(), t.into())
    }

    pub fn finish<I: Into<P::Target<'s>>, P: ParseUnit>(&self, t: I) -> ParseResult<'s, P> {
        Ok(self.new_token(t))
    }

    pub fn new_error(&mut self, reason: impl Into<String>) -> Error<'s> {
        Error::new(self.selection(), reason.into())
    }

    pub fn throw<P: ParseUnit>(&mut self, reason: impl Into<String>) -> ParseResult<'s, P> {
        Err(Some(self.new_error(reason)))
    }
}
pub struct Try<'p, 's, P: ParseUnit> {
    parser: &'p mut Parser<'s>,
    state: Option<std::result::Result<Token<'s, P>, Error<'s>>>,
    /// TODO
    #[cfg(feature = "parallel")]
    tasks: tokio::task::JoinSet<ParseResult<'s, P>>,
}

impl<'p, 's, P: ParseUnit> Try<'p, 's, P> {
    pub fn new(parser: &'p mut Parser<'s>) -> Self {
        parser.tries += 1;
        Self {
            parser,
            state: None,
        }
    }

    pub fn or_try<P1, F>(mut self, parser: F) -> Self
    where
        P1: ParseUnit<Target<'s> = P::Target<'s>>,
        F: FnOnce(&mut Parser<'s>) -> ParseResult<'s, P1>,
    {
        if self.state.is_some() {
            return self;
        }

        self.state = match self.parser.try_once(parser) {
            Ok(tk) => Some(Ok(Token::new(tk.selection, tk.target))),
            Err(Some(e)) => Some(Err(e)),
            _ => self.state,
        };

        self
    }

    pub fn or_error(mut self, reason: impl Into<String>) -> Self {
        self.state = self
            .state
            .or_else(|| Some(Err(self.parser.new_error(reason))));
        self
    }

    pub fn finish(self) -> ParseResult<'s, P> {
        self.state.expect("uncatched error").map_err(Some)
    }

    pub fn finish_no_error(self) -> ParseResult<'s, P> {
        match self.state {
            Some(r) => r.map_err(Some),
            None => Err(None),
        }
    }
}
