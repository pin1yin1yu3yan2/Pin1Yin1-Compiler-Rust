use crate::{
    error::{Error, ParseResult},
    parse_unit::ParseUnit,
    tokens::{Location, Selection, Token},
};

#[derive(Debug, Clone, Copy)]
pub struct Parser<'s> {
    pub(crate) src: &'s [char],
    pub(crate) idx: usize,
}

impl Iterator for Parser<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.src.get(self.idx).copied()?;
        self.idx += 1;
        Some(next)
    }
}

impl DoubleEndedIterator for Parser<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            return None;
        }
        self.idx -= 1;
        Some(self.src[self.idx])
    }
}

impl<'s> Parser<'s> {
    pub fn new(src: &[char]) -> Parser<'_> {
        Parser { src, idx: 0 }
    }

    pub fn get_location(&self) -> Location<'s> {
        Location::new(self.src, self.idx)
    }

    pub fn r#try<F, P>(&mut self, p: F) -> Try<'_, 's, P>
    where
        P: ParseUnit,
        F: FnOnce(&mut Parser) -> ParseResult<'s, P>,
    {
        Try {
            parser: self,
            state: None,
        }
        .or_try(p)
    }

    pub fn peek(&self) -> Option<char> {
        self.src.get(self.idx).copied()
    }

    pub fn select<C>(&mut self, selector: C) -> Selector<'s>
    where
        C: FnOnce(&mut Selector),
    {
        let mut c = Selector::new(*self);
        selector(&mut c);
        *self = c.parser;
        c
    }
}

pub struct Try<'p, 's, P: ParseUnit> {
    parser: &'p mut Parser<'s>,
    state: Option<std::result::Result<Token<'s, P>, Error<'s>>>,
}

impl<'p, 's, P: ParseUnit> Try<'p, 's, P> {
    pub fn or_try<F>(mut self, p: F) -> Self
    where
        P: ParseUnit,
        F: FnOnce(&mut Parser<'s>) -> ParseResult<'s, P>,
    {
        if self.state.is_some() {
            return self;
        }

        let mut tmp = *self.parser;

        match p(&mut tmp) {
            Ok(r) => self.state = Some(Ok(r)),
            Err(opte) => {
                if let Some(e) = opte {
                    self.state = Some(Err(e))
                }
            }
        }

        self
    }

    #[inline]
    pub fn or_parse(self) -> Self {
        let p = |p: &mut Parser<'s>| P::parse(p);
        self.or_try(p)
    }
}

pub struct Selector<'s> {
    parser: Parser<'s>,
    pub(crate) location: Vec<Location<'s>>,
    pub(crate) selection: Vec<Selection<'s>>,
}

impl Selector<'_> {
    pub fn new(parser: Parser) -> Selector<'_> {
        Selector {
            parser,
            location: vec![],
            selection: vec![],
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.parser.peek()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        self.parser.next()
    }

    pub fn next_back(&mut self) -> Option<char> {
        self.parser.next_back()
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

    pub fn take_while<Rule>(&mut self, rule: Rule)
    where
        Rule: Fn(char) -> bool,
    {
        let location = self.parser.get_location();
        self.skip_while(rule);
        self.location.push(location);
        self.selection
            .push(Selection::from_parser(&self.parser, location));
    }
}
