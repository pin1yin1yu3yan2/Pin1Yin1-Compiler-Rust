use crate::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParserCache<'s> {
    /// really cache
    pub(crate) chars: &'s [char],
    /// the idx of the fist character in the cache
    pub(crate) first_index: usize,
    /// the idx of the next character in the cache
    ///
    /// [`Self::chars_cache_idx`] + [`Self::chars_cache.len()`]
    pub(crate) final_index: usize,
}

/// An implementation of the language parser **without** any [`Clone::clone`] call!
///
/// This implementation uses many references to increase performance(maybe...?)
#[derive(Debug, Clone, Copy)]
pub struct Parser<'s, S: Copy = char> {
    /// source codes
    pub(crate) src: &'s Source<S>,
    /// parse state: the index of the first character in this [`ParseUnit`]
    start_idx: Option<usize>,
    /// parse state: the index of the current character in this [`ParseUnit`]
    pub(crate) idx: usize,
    /// cahce for [`ParseUnit`], increse the parse speed for [[char]]
    pub(crate) cache: ParserCache<'s>,
}

impl<S: Copy> WithSelection for Parser<'_, S> {
    fn get_selection(&self) -> Selection {
        if self.start_idx.is_some() {
            Selection::new(self.start_idx(), self.idx)
        } else {
            // while finishing parsing or throwing an error, the taking may not ever be started
            // so, match the case to make error reporting easier&better
            Selection::new(self.idx, self.idx + 1)
        }
    }
}

impl<S: Copy> Parser<'_, S> {
    /// get the next character
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn next(&mut self) -> Option<&S> {
        let next = self.src.get(self.idx)?;
        self.idx += 1;
        Some(next)
    }

    /// peek the next character
    pub(crate) fn peek(&self) -> Option<&S> {
        self.src.get(self.idx)
    }

    /// Returns the [`Parser::start_idx`] of this [`Parser`].
    ///
    /// # Panics
    ///
    /// this method should never panic
    pub(crate) fn start_idx(&self) -> usize {
        self.start_idx.unwrap()
    }

    pub fn is_ending(&self) -> bool {
        self.idx >= self.src.len()
    }
}

impl<'s, S: Copy> Parser<'s, S> {
    /// create a new parser from a slice of [char]
    pub fn new(src: &'s Source) -> Parser<'s> {
        Parser {
            src,
            idx: 0,
            start_idx: None,

            cache: ParserCache {
                chars: &src[..0],
                first_index: usize::MAX,
                final_index: usize::MAX,
            },
        }
    }

    /// start a [`Try`], allow you to try many times until you get a actually [`Error`]
    /// or successfully parse a [`ParseUnit`]
    pub fn r#try<'p, F, P>(&'p mut self, p: F) -> Try<'s, 'p, S, P>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<'s, S>) -> ParseResult<P, S>,
    {
        Try::new(self).or_try(p)
    }

    /// try to parse, mean that [`PuResult::Unmatch`] is allowed
    ///
    /// in this case, [`PuResult::Unmatch`] will be transformed into [`None`]
    ///
    /// so that you can use `?` as usual after using match / if let ~
    pub fn once<P, F>(&mut self, parser: F) -> ParseResult<P, S>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<'s, S>) -> ParseResult<P, S>,
    {
        // create a temp parser and reset its state
        let mut tmp = *self;
        tmp.start_idx = None;

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
        let result = parser(&mut tmp);

        #[cfg(feature = "parser_calling_tree")]
        {
            DEPTH.fetch_sub(1, std::sync::atomic::Ordering::Release);
            for _ in 0..DEPTH.load(std::sync::atomic::Ordering::Acquire) {
                print!("    ")
            }
            match &result {
                Result::Success(_) => println!("Success: {p_name}"),
                Result::Failed(e) => println!("{:?}: {p_name}", e.kind()),
            }
        }
        self.sync_with(&result, &tmp);
        result
    }

    /// try to parse,
    pub fn try_once<P, F>(&mut self, parser: F) -> Option<ParseResult<P, S>>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<'s, S>) -> ParseResult<P, S>,
    {
        let result = self.once(parser);
        if result.is_unmatch() {
            None
        } else {
            Some(result)
        }
    }

    /// sync state with the parsing result from a temp sub parser
    pub(crate) fn sync_with<P>(&mut self, result: &ParseResult<P, S>, tmp: &Parser<'s, S>)
    where
        P: ParseUnit<S>,
    {
        match result {
            // if success,
            ParseResult::Success(..) => {
                // foward tmp parser's work to main parser
                self.idx = tmp.idx;
                self.start_idx = self.start_idx.or(tmp.start_idx);
            }
            _ => {
                self.cache = tmp.cache;
            }
        }
    }

    /// make effort if success or return [`Error`], make no effort if failure
    /// this kind of try...
    pub fn parse<P: ParseUnit<S>>(&mut self) -> ParseResult<P, S> {
        self.once(P::parse)
    }

    /// try to parse, mean that [`PuResult::Unmatch`] is allowed
    ///
    /// in this case, [`PuResult::Unmatch`] will be transformed into [`None`]
    ///
    /// so that you can use `?` as usual after using match / if let ~
    pub fn try_parse<P: ParseUnit<S>>(&mut self) -> Option<ParseResult<P, S>> {
        self.try_once(P::parse)
    }

    /// set [`Self::start_idx`] to set [`Self::idx`] if [`Self::start_idx`] is unset
    ///
    /// like this method, if i dont set some of methods private in crate, someting strange
    /// behaviour will happen because of increment calling
    ///
    /// The existing [`ParseUnit`] implementation is sufficient
    pub(crate) fn start_taking(&mut self) {
        self.start_idx = Some(self.start_idx.unwrap_or(self.idx));
    }

    /// make a new [`PU`] with the given value and parser's selection
    pub fn make_token<I: Into<P::Target>, P: ParseUnit<S>>(&self, t: I) -> PU<P, S> {
        PU::new(self.get_selection(), t.into())
    }

    /// finish the successful parsing, just using the this method to make return easier
    pub fn finish<I: Into<P::Target>, P: ParseUnit<S>>(&mut self, t: I) -> ParseResult<P, S> {
        ParseResult::Success(self.make_token(t))
    }
}

impl<'s> Parser<'s, char> {
    /// skip characters that that follow the given rule
    pub(crate) fn skip_while<Rule>(&mut self, rule: Rule) -> &mut Self
    where
        Rule: Fn(char) -> bool,
    {
        while self.peek().copied().is_some_and(&rule) {
            self.next();
        }
        self
    }

    /// skip whitespaces
    pub(crate) fn skip_whitespace(&mut self) -> &mut Self {
        self.skip_while(|c| c.is_ascii_whitespace());
        self
    }

    /// taking characters that follow the given rule
    pub(crate) fn take_while<Rule>(&mut self, rule: Rule) -> &'s [char]
    where
        Rule: Fn(char) -> bool,
    {
        self.start_taking();
        self.skip_while(&rule);
        &self.src[self.start_idx.unwrap()..self.idx]
    }

    ///  very hot funtion!!!
    pub fn get_chars(&mut self) -> Result<PU<&'s [char]>> {
        let p = self;
        // reparse and cache the result
        if p.cache.first_index != p.idx {
            p.cache.first_index = p.idx;
            p.cache.chars = p.skip_whitespace().take_while(chars_taking_rule);
            p.cache.final_index = p.idx;
        } else {
            // load from cache, call p.start_taking() to perform the right behavior
            p.start_taking();
            p.idx = p.cache.final_index;
        }

        p.finish(p.cache.chars)
    }
}

/// a [`Try`], allow you to try many times until you get a actually [`Error`]
/// or successfully parse a [`ParseUnit`]
pub struct Try<'s, 'p, S: Copy, P: ParseUnit<S>> {
    parser: &'p mut Parser<'s, S>,
    state: Option<ParseResult<P, S>>,
}

impl<'s, 'p, S: Copy, P: ParseUnit<S>> Try<'s, 'p, S, P> {
    pub fn new(parser: &'p mut Parser<'s, S>) -> Self {
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
        F: FnOnce(&mut Parser<'s, S>) -> ParseResult<P1, S>,
    {
        if self.state.is_none()
            || self
                .state
                .as_ref()
                .is_some_and(|result| result.is_unmatch())
        {
            self.state = Some(self.parser.once(parser).map_pu(|t| t));
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
    /// its not recommended to return [`Err`] with [`None`]
    ///
    /// there should be at least one [`Self::or_try`] return [`Err`] with [`Some`] in,
    /// or the parser will throw a message with very bad readability
    pub fn finish(self) -> ParseResult<P, S> {
        self.state.unwrap()
    }
}
