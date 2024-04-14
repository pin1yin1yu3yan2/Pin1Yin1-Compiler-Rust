use crate::*;

/// cahce for [`ParseUnit`], increse the parse speed for [[char]]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParserCache {
    /// really cache
    pub(crate) span: Span,
    /// the idx of the fist character in the cache
    pub(crate) first_index: usize,
    /// be different from [`Self::first_index`], this is the index that after [`Parser::skip_whitespace`]
    pub(crate) start_index: usize,
    /// the idx of the next character in the cache
    ///
    /// [`Self::chars_cache_idx`] + [`Self::chars_cache.len()`]
    pub(crate) final_index: usize,
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct ParserState {
    /// the index of the first character in this [`ParseUnit`]
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

    /// foward tmp parser's work to main parser

    pub(crate) fn force_sync(mut self, tmp: &ParserState) -> Self {
        self.idx = tmp.idx;
        self.start_idx = self.start_idx.or(tmp.start_idx);
        self
    }

    /// sync [`ParserState`] with the parsing result from a temp sub parser's [`ParserState`]
    pub(crate) fn sync_with<S, P>(self, tmp: &ParserState, result: &ParseResult<P, S>) -> Self
    where
        P: ParseUnit<S>,
    {
        if result.is_ok() {
            self.force_sync(tmp)
        } else {
            self
        }
    }
}

#[cfg(feature = "parser_calling_tree")]
mod calling_tree {
    use crate::{ParseErrorKind, ParseUnit, Span, WithSpan};

    #[derive(Clone, Copy)]
    pub enum Calling {
        Start,
        Success(Span),
        Err(ParseErrorKind, Span),
    }

    impl Calling {
        /// Returns `true` if the calling is [`Start`].
        ///
        /// [`Start`]: Calling::Start
        #[must_use]
        pub fn is_start(&self) -> bool {
            matches!(self, Self::Start)
        }
    }

    impl<P: ParseUnit<S>, S> From<&super::ParseResult<P, S>> for Calling {
        fn from(value: &super::ParseResult<P, S>) -> Self {
            match value {
                Ok(o) => Self::Success(o.get_span()),
                Err(e) => Self::Err(e.kind(), e.get_span()),
            }
        }
    }

    impl std::fmt::Debug for Calling {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Start => write!(f, "Start: "),
                Self::Success(span) => write!(f, "Success<{span:?}>"),
                Self::Err(arg0, span) => write!(f, "{:?}<{span:?}>", arg0),
            }
        }
    }

    #[derive(Debug, Clone)]
    enum Record {
        Normal { pu: &'static str, call: Calling },
        Custom { msg: String },
    }

    impl Record {
        fn print(&self, depth: &mut usize, f: &mut impl std::fmt::Write) -> std::fmt::Result {
            match self {
                Record::Normal { pu, call } => {
                    if call.is_start() {
                        for _ in 0..*depth {
                            write!(f, "    ")?;
                        }
                        *depth += 1;
                        writeln!(f, "{:?}{}", call, pu)
                    } else {
                        *depth -= 1;

                        for _ in 0..*depth {
                            write!(f, "    ")?;
                        }
                        writeln!(f, "{:?} {}", call, pu)
                    }
                }
                Record::Custom { msg } => writeln!(f, "{msg}"),
            }
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct CallingTree {
        records: Vec<Record>,
    }

    impl CallingTree {
        pub fn record_normal<P>(&mut self, call: Calling) {
            self.records.push(Record::Normal {
                pu: std::any::type_name::<P>(),
                call,
            });
        }

        pub fn record_custom(&mut self, msg: impl std::fmt::Display) {
            self.records.push(Record::Custom {
                msg: msg.to_string(),
            });
        }
    }

    impl std::fmt::Display for CallingTree {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut depth = 0;
            for record in &self.records {
                record.print(&mut depth, f)?;
            }
            Ok(())
        }
    }
}

/// An implementation of the language parser **without** any [`Clone::clone`] call!
#[derive(Debug, Clone)]
pub struct Parser<S = char> {
    /// source codes
    src: Source<S>,

    state: ParserState,
    cache: ParserCache,
    #[cfg(feature = "parser_calling_tree")]
    calling_tree: calling_tree::CallingTree,
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
}

impl<S> Parser<S> {
    /// create a new parser from a slice of [char]
    pub fn new(src: Source<S>) -> Parser<S> {
        Parser {
            src,
            state: ParserState::default(),
            cache: ParserCache {
                span: Span::new(0, 0),
                first_index: usize::MAX, //this is not rusty enough(
                start_index: usize::MAX,
                final_index: usize::MAX,
            },
            #[cfg(feature = "parser_calling_tree")]
            calling_tree: Default::default(),
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
    /// Returns the [`Parser::start_idx`] of this [`Parser`].
    ///
    /// # Panics
    ///
    /// this method should never panic
    pub(crate) fn start_idx(&self) -> usize {
        self.state.start_idx.unwrap()
    }

    #[cfg(feature = "parser_calling_tree")]
    pub fn get_calling_tree(&self) -> &calling_tree::CallingTree {
        &self.calling_tree
    }

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
        let state = self.state;
        self.state = self.state.fork();

        let span = {
            let p = &mut *self;
            // reparse and cache the result
            if p.cache.first_index != p.state.idx {
                p.cache.first_index = p.state.idx;
                p.skip_whitespace().start_taking();
                p.cache.start_index = p.start_idx();
                p.cache.span = p.take_while(chars_taking_rule);
                p.cache.final_index = p.state.idx;
            } else {
                // load from cache, call p.start_taking() to perform the right behavior
                p.start_taking();
                p.state.start_idx = p.state.start_idx.or(Some(p.cache.first_index));
                p.state.idx = p.cache.final_index;
            }

            p.cache.span
        };

        self.state = state.force_sync(&self.state);

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

    /// be different from directly call, this kind of parse will log
    /// (if parser_calling_tree feature enabled)
    pub fn once_no_try<P, F>(&mut self, parser: F) -> ParseResult<P, S>
    where
        P: ParseUnit<S>,
        F: FnOnce(&mut Parser<S>) -> ParseResult<P, S>,
    {
        #[cfg(feature = "parser_calling_tree")]
        self.calling_tree
            .record_normal::<P>(calling_tree::Calling::Start);

        // do parsing
        let result = parser(self);

        #[cfg(feature = "parser_calling_tree")]
        self.calling_tree
            .record_normal::<P>(calling_tree::Calling::from(&result));

        #[cfg_attr(not(feature = "parser_calling_tree"), allow(clippy::let_and_return))]
        result
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

        let state = self.state;
        self.state = self.state.fork();

        let result = self.once_no_try(parser);
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
            let lhs = P::parse(p).apply(MapError::new(|e| e.map(format!("expect `{}`", rhs))))?;
            if *lhs == rhs {
                Ok(lhs)
            } else {
                Err(lhs.make_parse_error(
                    format!("expect `{}`, but `{}` found", rhs, *lhs),
                    ParseErrorKind::Unmatch,
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
        let is_unmatch = self.state.as_ref().is_some_and(|result| {
            result
                .as_ref()
                .is_err_and(|e| e.kind() == ParseErrorKind::Unmatch)
        });

        if self.state.is_none() || is_unmatch {
            let state = self.parser.once(parser).map(|pu| PU::new(pu.span, pu.item));
            self.state = Some(state);
        }
        self
    }

    /// set the default error
    pub fn or_error(mut self, reason: impl ToString) -> Self {
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
