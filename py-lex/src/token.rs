use terl::*;
#[derive(Debug, Clone)]
pub struct Token {
    pub string: String,
    /// note: span here are span in [`Buffer<char>`]
    span: Span,
}

impl Token {
    pub fn new(string: impl Into<String>, span: Span) -> Self {
        Self {
            string: string.into(),
            span,
        }
    }
}

impl std::ops::Deref for Token {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(f)
    }
}

impl WithSpan for Token {
    #[inline]
    fn get_span(&self) -> Span {
        self.span
    }
}

impl ParseUnit<char> for Token {
    type Target = Self;

    fn parse(p: &mut Parser<char>) -> ParseResult<Self, char> {
        fn w(c: &char) -> bool {
            c.is_ascii_alphanumeric() || *c == '_'
        }

        // skip whitespace
        while p.next_if(|c| !w(c)).is_some() {}

        // get string until whitespace
        let mut string = String::new();
        p.start_taking();
        while let Some(next) = p.next_if(w) {
            string.push(*next);
        }

        // return unmatch if string is empty
        if string.is_empty() {
            return p.unmatch("empty string");
        }

        Ok(Token {
            string,
            span: p.get_span(),
        })
    }
}

impl ParseUnit<Token> for Token {
    type Target = Token;

    #[inline]
    fn parse(p: &mut Parser<Token>) -> Result<Self::Target, ParseError> {
        match p.next().cloned() {
            Some(token) => Ok(token),
            None => p.unmatch("no token left"),
        }
    }
}

impl Source for Token {
    type HandleErrorWith<'b> = (&'b Buffer<char>, &'b Buffer<Token>);

    #[inline]
    fn handle_location<S>(
        with: &Self::HandleErrorWith<'_>,
        buffer: &mut S,
        loc: Span,
        msg: &str,
    ) -> std::fmt::Result
    where
        S: std::fmt::Write,
    {
        let (chars, tokens) = with;
        let loc = tokens[loc.start].get_span() + tokens[loc.end - 1].get_span();
        char::handle_location(chars, buffer, loc, msg)
    }
}

/// a type which implemented [`ParseUnit<S>`] with source code it selected
pub struct PU<P> {
    pub(crate) span: Span,
    pub(crate) item: P,
}

impl<S, P> ParseUnit<S> for PU<P>
where
    P: ParseUnit<S>,
    S: Source,
{
    type Target = PU<P::Target>;

    fn parse(p: &mut Parser<S>) -> Result<Self::Target, ParseError> {
        P::parse(p).map(|item| PU::new(p.get_span(), item))
    }
}

impl<P> WithSpan for PU<P> {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl<P> PU<P> {
    #[inline]
    pub const fn new(span: Span, item: P) -> Self {
        Self { span, item }
    }

    /// take [ParseUnit::Target] from [`PU`]
    #[inline]
    pub fn take(self) -> P {
        self.item
    }

    /// map [ParseUnit::Target]
    #[inline]
    pub fn map<P1, M>(self, mapper: M) -> PU<P1>
    where
        M: FnOnce(P) -> P1,
    {
        PU::new(self.span, mapper(self.item))
    }
}

impl<P> std::fmt::Debug for PU<P>
where
    P: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PU")
            .field("span", &self.span)
            .field("item", &self.item)
            .finish()
    }
}

impl<P> std::fmt::Display for PU<P>
where
    P: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.item, f)
    }
}

impl<P> Clone for PU<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            span: self.span,
            item: self.item.clone(),
        }
    }
}

impl<P> Copy for PU<P> where P: Copy {}

impl<P> std::ops::Deref for PU<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<P> std::ops::DerefMut for PU<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

pub struct RPU<Item>(pub Item);

impl<Item, S: Source> ReverseParseUnit<S> for RPU<Item>
where
    Item: ReverseParseUnit<S>,
{
    type Left = PU<Item::Left>;

    #[inline]
    fn reverse_parse(&self, p: &mut Parser<S>) -> Result<Self::Left, ParseError> {
        self.0
            .reverse_parse(p)
            .map(|item| PU::new(p.get_span(), item))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]

    fn token() {
        use terl::*;
        let file_name = std::any::type_name_of_val(&token).to_owned();
        let src = "123456 abcde \n 114514abc [] ()";
        let buffer = Buffer::new(file_name, src.chars().collect());
        let mut parser = Parser::new(buffer);

        let mut tokens: Vec<_> = vec![];
        while let Some(token) = parser.parse::<PU<Token>>().apply(mapper::Try).unwrap() {
            tokens.push(token)
        }

        let expect = src.chars().enumerate().collect::<Vec<_>>();
        let expect = expect
            .split(|(.., c)| c.is_whitespace())
            .fold(vec![], |mut expect, slice| {
                //  " \n " will generate two empty slice
                if !slice.is_empty() {
                    let span = Span::new(slice.first().unwrap().0, slice.last().unwrap().0);
                    let string = slice.iter().map(|(.., c)| c).collect::<String>();
                    expect.push(Token::new(string, span));
                }
                expect
            });

        for (got, expect) in tokens.into_iter().zip(expect) {
            let (got, expect): (&str, &str) = (&got, &expect);
            assert_eq!(got, expect);
        }
    }
}
