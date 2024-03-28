use crate::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParserCache {
    /// really cache
    pub(crate) span: Span,
    /// the idx of the fist character in the cache
    pub(crate) first_index: usize,
    /// the idx of the next character in the cache
    ///
    /// [`Self::chars_cache_idx`] + [`Self::chars_cache.len()`]
    pub(crate) final_index: usize,
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct ParserState {
    start_idx: Option<usize>,
    /// parse state: the index of the current character in this [`ParseUnit`]
    idx: usize,
}

impl ParserState {
    pub(crate) fn fork(&self) -> ParserState {
        Self {
            start_idx: None,
            idx: self.idx,
        }
    }

    /// sync [`ParserState`] with the parsing result from a temp sub parser's [`ParserState`]
    pub(crate) fn sync_with<S, P>(mut self, tmp: &ParserState, result: &ParseResult<P, S>) -> Self
    where
        P: ParseUnit<S>,
    {
        if result.is_ok() {
            // foward tmp parser's work to main parser
            self.idx = tmp.idx;
            self.start_idx = self.start_idx.or(tmp.start_idx);
        }
        self
    }
}

/// An implementation of the language parser **without** any [`Clone::clone`] call!
#[derive(Debug, Clone)]
pub struct Parser<S = char> {
    /// source codes
    src: Source<S>,
    /// parse state: the index of the first character in this [`ParseUnit`]
    state: ParserState,
    /// cahce for [`ParseUnit`], increse the parse speed for [[char]]
    cache: ParserCache,
}

impl<S> WithSpan for Parser<S> {
    fn get_span(&self) -> Span {
        if self.state.start_idx.is_some() {
            Span::new(self.start_idx(), self.state.idx)
        } else {
            // while finishing parsing or throwing an error, the taking may not ever be started
            // so, match the case to make error reporting easier&better
            Span::new(self.state.idx, self.state.idx + 1)
        }
    }
}

impl<S> Parser<S> {
    /// get the next character
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn next(&mut self) -> Option<&S> {
        let next = self.src.get(self.state.idx)?;
        self.state.idx += 1;
        Some(next)
    }

    /// peek the next character
    pub(crate) fn peek(&self) -> Option<&S> {
        self.src.get(self.state.idx)
    }

    /// Returns the [`Parser::start_idx`] of this [`Parser`].
    ///
    /// # Panics
    ///
    /// this method should never panic
    pub(crate) fn start_idx(&self) -> usize {
        self.state.start_idx.unwrap()
    }
}

impl<S> Parser<S> {
    /// create a new parser from a slice of [char]
    pub fn new(src: Source<S>) -> Parser<S> {
        Parser {
            src,
            state: ParserState::default(),
            cache: ParserCache {
                span: Span::new(0, 0),
                first_index: usize::MAX,
                final_index: usize::MAX,
            },
        }
    }

    /// set [`Self::ParserState`] to set [`ParserState::idx`] if [`Self::start_idx`] is unset
    pub(crate) fn start_taking(&mut self) {
        self.state.start_idx = Some(self.state.start_idx.unwrap_or(self.state.idx));
    }

    pub(crate) fn select(&self, span: Span) -> &[S] {
        &self.src[span]
    }

    pub fn process<P, S1>(self, processer: P) -> Result<Parser<S1>>
    where
        P: FnOnce(Self) -> Result<Vec<S1>>,
    {
        let file_name = self.src.file_name().to_owned();
        let new_src = processer(self)?;
        let parser = Parser::<S1>::new(Source::<S1>::new(file_name, new_src));
        Result::Ok(parser)
    }
}

impl<S> Parser<S> {
    /// skip characters that that follow the given rule
    pub(crate) fn skip_while<Rule>(&mut self, rule: Rule) -> &mut Self
    where
        Rule: Fn(&S) -> bool,
    {
        while self.peek().is_some_and(&rule) {
            self.next();
        }
        self
    }

    /// taking characters that follow the given rule
    pub(crate) fn take_while<Rule>(&mut self, rule: Rule) -> Span
    where
        Rule: Fn(&S) -> bool,
    {
        self.start_taking();
        self.skip_while(&rule);
        Span::new(self.state.start_idx.unwrap(), self.state.idx)
    }
}

impl Parser<char> {
    /// skip whitespaces
    pub(crate) fn skip_whitespace(&mut self) -> &mut Self {
        self.skip_while(|c| c.is_ascii_whitespace());
        self
    }

    ///  very hot funtion!!!
    pub fn get_chars(&mut self) -> ParseResult<&[char], char> {
        let mut state = self.state.fork();
        self.state = state;

        let span = {
            let p = &mut *self;
            // reparse and cache the result
            if p.cache.first_index != p.state.idx {
                p.cache.first_index = p.state.idx;
                p.cache.span = p.skip_whitespace().take_while(chars_taking_rule);
                p.cache.final_index = p.state.idx;
            } else {
                // load from cache, call p.start_taking() to perform the right behavior
                p.start_taking();
                p.state.idx = p.cache.final_index;
            }

            p.cache.span
        };
        self.state = {
            let tmp = &self.state;
            // foward tmp parser's work to main parser
            state.idx = tmp.idx;
            state.start_idx = state.start_idx.or(tmp.start_idx);
            state
        };

        self.finish(self.select(span))
    }

    pub fn handle_error(&self, error: Error) -> std::result::Result<String, std::fmt::Error> {
        self.src.handle_error(error)
    }
}

impl<S> Parser<S> {
    /// start a [`Try`], allow you to try many times until you get a actually [`Error`]:
    /// (not [`ErrorKind::Unmatch`]) or successfully parse a [`ParseUnit`]
    pub fn r#try<F, P>(&mut self, p: F) -> Try<'_, P, S>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<S>) -> ParseResult<P, S>,
    {
        Try::new(self).or_try(p)
    }

    /// try to parse, if the parsing Err, there will be no effect made on [`Parser`]
    ///
    /// you should not call [`ParseUnit::parse`] directly, using methods like [`Parser::once`]
    /// instead
    pub fn once<P, F>(&mut self, parser: F) -> ParseResult<P, S>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<S>) -> ParseResult<P, S>,
    {
        // create a temp parser and reset its state

        let state = self.state.fork();

        #[cfg(feature = "parser_calling_tree")]
        let p_name = std::any::type_name::<P>();
        #[cfg(feature = "parser_calling_tree")]
        use std::sync::atomic::AtomicUsize;
        #[cfg(feature = "parser_calling_tree")]
        static DEPTH: AtomicUsize = AtomicUsize::new(0);
        #[cfg(feature = "parser_calling_tree")]
        {
            for _ in 0..DEPTH.load(std::sync::atomic::Ordering::Acquire) {
                print!("    ")
            }
            println!("Start {p_name}");
            DEPTH.fetch_add(1, std::sync::atomic::Ordering::Release);
        }

        // do parsing
        self.state = state;
        let result = parser(self);

        #[cfg(feature = "parser_calling_tree")]
        {
            DEPTH.fetch_sub(1, std::sync::atomic::Ordering::Release);
            for _ in 0..DEPTH.load(std::sync::atomic::Ordering::Acquire) {
                print!("    ")
            }
            match &result {
                Result::Ok(_) => println!("Ok: {p_name}"),
                Result::Err(e) => println!("{:?}: {p_name}", e.kind()),
            }
        }

        self.state = state.sync_with(&self.state, &result);

        result
    }

    #[inline]
    pub fn parse<P: ParseUnit<S>>(&mut self) -> ParseResult<P, S> {
        self.once(P::parse)
    }

    pub fn match_<P>(&mut self, rhs: P) -> ParseResult<P, S>
    where
        // for better type inference
        P: ParseUnit<S, Target = P>,
        P::Target: PartialEq + std::fmt::Display,
    {
        self.once(|p| {
            let lhs = P::parse(p).apply(MapError::new(|e| e.map(format!("exprct `{}`", rhs))))?;
            if *lhs == rhs {
                Ok(lhs)
            } else {
                Err(lhs.make_error(
                    format!("exprct `{}`, but `{}` found", rhs, *lhs),
                    ErrorKind::Unmatch,
                ))
            }
        })
    }

    /// finish the successful parsing, just using the this method to make return easier
    pub fn finish<T: Into<P::Target>, P: ParseUnit<S>>(&self, t: T) -> ParseResult<P, S> {
        ParseResult::Ok(self.make_pu(t.into()))
    }
}

/// a [`Try`], allow you to try many times until you get a actually [`Error`]
/// or successfully parse a [`ParseUnit`]
pub struct Try<'p, P: ParseUnit<S>, S> {
    parser: &'p mut Parser<S>,
    state: Option<ParseResult<P, S>>,
}

impl<'p, S, P: ParseUnit<S>> Try<'p, P, S> {
    pub fn new(parser: &'p mut Parser<S>) -> Self {
        Self {
            parser,
            state: None,
        }
    }

    /// try once again
    ///
    /// do noting if the [`Try`] successfully parsed the [`ParseUnit`],
    /// or got a actually [`Error`]
    pub fn or_try<P1, F>(mut self, parser: F) -> Self
    where
        P1: ParseUnit<S, Target = P::Target>,
        F: FnOnce(&mut Parser<S>) -> ParseResult<P1, S>,
    {
        if self.state.is_none()
            || self.state.as_ref().is_some_and(|result| {
                result
                    .as_ref()
                    .is_err_and(|e| e.kind() == ErrorKind::Unmatch)
            })
        {
            self.state = Some(
                self.parser
                    .once(parser)
                    .map(|pu| PU::new(pu.span, pu.target)),
            );
        }
        self
    }

    /// set the default error
    pub fn or_error(mut self, reason: impl Into<String>) -> Self {
        self.state = self.state.or_else(|| Some(self.parser.unmatch(reason)));
        self
    }

    /// finish parsing tring
    ///
    ///
    /// there should be at least one [`Self::or_try`] return [`Result::Ok`]
    /// or [`Result::Err`] , or panic
    pub fn finish(self) -> ParseResult<P, S> {
        self.state.unwrap()
    }
}
