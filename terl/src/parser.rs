use crate::*;

#[derive(Debug, Clone, Copy)]
enum StartIdx {
    Init(usize),
    Skiped(usize),
}

impl std::ops::Deref for StartIdx {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            StartIdx::Init(idx) | StartIdx::Skiped(idx) => idx,
        }
    }
}

impl std::ops::DerefMut for StartIdx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            StartIdx::Init(idx) | StartIdx::Skiped(idx) => idx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParserState {
    /// the index of the first character in this [`ParseUnit`]
    start: StartIdx,
    /// parse state: the index of the current character in this [`ParseUnit`]
    idx: usize,
}

impl ParserState {
    pub(crate) fn fork(&self) -> ParserState {
        Self {
            start: StartIdx::Init(self.idx),
            idx: self.idx,
        }
    }

    /// foward tmp parser's work to main parser

    pub(crate) fn force_sync(mut self, tmp: &ParserState) -> Self {
        self.idx = tmp.idx;
        self.start = match (self.start, tmp.start) {
            (StartIdx::Init(..), StartIdx::Skiped(idx)) => StartIdx::Skiped(idx),
            (idx, ..) => StartIdx::Skiped(*idx),
        };
        self
    }

    /// sync [`ParserState`] with the parsing result from a temp sub parser's [`ParserState`]
    pub(crate) fn sync_with<T, E>(self, tmp: &ParserState, result: &Result<T, E>) -> Self {
        if result.is_ok() {
            self.force_sync(tmp)
        } else {
            self
        }
    }
}

#[cfg(feature = "parser_calling_tree")]
mod calling_tree {
    use crate::{ParseErrorKind, Span};

    #[derive(Clone, Copy)]
    pub enum Calling {
        Start,
        Success(Span),
        Err(ParseErrorKind, Span),
    }

    impl Calling {
        pub fn new<P>(result: &Result<P, super::ParseError>, span: Span) -> Self {
            match result {
                Ok(_) => Self::Success(span),
                Err(e) => Self::Err(e.kind(), span),
            }
        }

        /// Returns `true` if the calling is [`Start`].
        ///
        /// [`Start`]: Calling::Start
        #[must_use]
        pub fn is_start(&self) -> bool {
            matches!(self, Self::Start)
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
pub struct Parser<S: Source = char> {
    /// source codes
    src: Buffer<S>,
    state: ParserState,
    #[cfg(feature = "parser_calling_tree")]
    calling_tree: calling_tree::CallingTree,
}

impl<S: Source> WithSpan for Parser<S> {
    fn get_span(&self) -> Span {
        // while finishing parsing or throwing an error, the taking may not ever be started
        // so, match the case to make error reporting easier&better
        if self.start_idx() == self.current_idx() {
            Span::new(self.start_idx(), self.start_idx() + 1)
        } else {
            Span::new(self.start_idx(), self.current_idx())
        }
    }
}

impl<S: Source> Parser<S> {
    /// create a new parser from a slice of [char]
    pub fn new(src: Buffer<S>) -> Parser<S> {
        Parser {
            src,
            state: ParserState {
                start: StartIdx::Init(0),
                idx: 0,
            },
            #[cfg(feature = "parser_calling_tree")]
            calling_tree: Default::default(),
        }
    }

    /// retuen slice to elements which [`Span`] selected
    ///
    /// # Panic
    ///
    /// panic if [`Span`] is out range
    #[inline]
    pub fn select(&self, span: Span) -> &[S] {
        &self.src[span]
    }

    /// get the next character
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&S> {
        let next = self.src.get(self.current_idx())?;
        self.state.idx += 1;
        Some(next)
    }

    /// peek the next character
    #[inline]
    pub fn peek(&self) -> Option<&S> {
        self.src.get(self.current_idx())
    }

    #[inline]
    pub(crate) fn start_idx(&self) -> usize {
        *self.state.start
    }

    #[inline]
    pub(crate) fn current_idx(&self) -> usize {
        self.state.idx
    }

    /// start taking items
    ///
    /// if start idx is unset, it will be set to current idx,
    /// or the calling makes no effort
    #[inline]
    pub fn start_taking(&mut self) {
        if let StartIdx::Init(..) = self.state.start {
            self.state.start = StartIdx::Skiped(self.current_idx());
        }
    }

    #[inline]
    pub fn process<P, S1>(mut self, processer: P) -> Result<(Buffer<S>, Parser<S1>), ParseError>
    where
        P: FnOnce(&mut Self) -> Result<Vec<S1>, ParseError>,
        S1: Source,
    {
        let name = self.src.name().to_owned();
        let new_src = processer(&mut self)?;

        Result::Ok((self.src, Parser::new(Buffer::<S1>::new(name, new_src))))
    }

    #[inline]
    #[cfg(feature = "parser_calling_tree")]
    pub fn calling_tree(&self) -> &calling_tree::CallingTree {
        &self.calling_tree
    }

    pub fn buffer(&self) -> &Buffer<S> {
        &self.src
    }

    /// be different from directly call, this kind of parse will log
    /// (if parser_calling_tree feature enabled)
    ///
    /// if the feature is disable, this method has no different with directly call
    pub fn once_no_try<P, F>(&mut self, parser: F) -> Result<P, ParseError>
    where
        F: FnOnce(&mut Parser<S>) -> Result<P, ParseError>,
    {
        #[cfg(feature = "parser_calling_tree")]
        self.calling_tree
            .record_normal::<P>(calling_tree::Calling::Start);

        // do parsing

        let result = parser(self);

        #[cfg(feature = "parser_calling_tree")]
        self.calling_tree
            .record_normal::<P>(calling_tree::Calling::new(&result, self.get_span()));

        #[cfg_attr(not(feature = "parser_calling_tree"), allow(clippy::let_and_return))]
        result
    }

    /// try to parse, if the parsing Err, there will be no effect made on [`Parser`]
    pub fn once<P, F>(&mut self, parser: F) -> Result<P, ParseError>
    where
        F: FnOnce(&mut Parser<S>) -> Result<P, ParseError>,
    {
        // create a temp parser and reset its state

        let state = self.state;
        self.state = self.state.fork();

        let result = self.once_no_try::<P, _>(parser);
        self.state = state.sync_with(&self.state, &result);

        result
    }

    #[inline]
    pub fn parse<P: ParseUnit<S>>(&mut self) -> ParseResult<P, S> {
        self.once(P::parse)
    }

    #[inline]
    pub fn match_<P>(&mut self, rhs: P) -> Result<P::Left, ParseError>
    where
        P: ReverseParser<S>,
    {
        // also a `try`
        self.once(|p| rhs.reverse_parse(p))
    }

    #[inline]
    pub fn handle_error(&self, error: Error) -> String
    where
        S: for<'b> Source<HandleErrorWith<'b> = Buffer<S>>,
    {
        S::handle_error(&self.src, error)
    }
}

/// a [`Try`], allow you to try many times until you get a actually [`Error`]
/// or successfully parse a [`ParseUnit`]
pub struct Try<'p, P: ParseUnit<S>, S: Source> {
    parser: &'p mut Parser<S>,
    state: Option<ParseResult<P, S>>,
}

impl<'p, S: Source, P: ParseUnit<S>> Try<'p, P, S> {
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
        F: FnOnce(&mut Parser<S>) -> Result<P1::Target, ParseError>,
    {
        let is_unmatch = self.state.as_ref().is_some_and(|result| {
            result
                .as_ref()
                .is_err_and(|e| e.kind() == ParseErrorKind::Unmatch)
        });

        if self.state.is_none() || is_unmatch {
            let state = self.parser.once(parser);
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
