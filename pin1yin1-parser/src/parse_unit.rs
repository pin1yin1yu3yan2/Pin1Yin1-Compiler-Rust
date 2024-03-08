use crate::{
    error::{ParseResult, Result},
    parser::{Parser, Selector},
    tokens::{Selection, Token},
};

pub struct Selections<'s> {
    pub sum: Selection<'s>,
    subs: Vec<Selection<'s>>,
}

impl<'s> Selections<'s> {
    pub fn new(sum: Selection<'s>, subs: Vec<Selection<'s>>) -> Self {
        Self { sum, subs }
    }

    pub fn len(&self) -> usize {
        self.sum.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sum.is_empty()
    }

    pub fn throw(&self, reason: impl Into<String>) -> Result<'s, ()> {
        self.sum.throw(reason)
    }
}

impl<'s> std::ops::Index<usize> for Selections<'s> {
    type Output = Selection<'s>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.subs[index]
    }
}

pub trait ParseUnit {
    type Target<'t>;

    fn select(selector: &mut Selector);

    fn generate<'s>(selections: &Selections<'s>) -> Result<'s, Self::Target<'s>>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self>
    where
        Self: Sized,
    {
        let selector = p.select(Self::select);
        let location = *selector.location.first().unwrap_or_else(|| {
            panic!(
                "{} selected nothing!",
                std::any::type_name_of_val(&Self::select)
            )
        });
        let selections = Selections::new(
            Selection::new(location, selector.selection.last().unwrap().len()),
            selector.selection,
        );

        let target = Self::generate(&selections)?;

        Ok(Token::new(selections.sum, target))
    }
}

impl ParseUnit for String {
    type Target<'t> = String;

    fn select(selector: &mut Selector) {
        selector
            .skip_whitespace()
            .take_while(|s| s.is_ascii_alphanumeric())
    }

    fn generate<'s>(selection: &Selections<'s>) -> Result<'s, Self::Target<'s>> {
        Ok(selection.sum.iter().collect())
    }
}

impl ParseUnit for usize {
    type Target<'t> = usize;

    fn select(selector: &mut Selector) {
        selector
            .skip_whitespace()
            .take_while(|c| c.is_ascii_digit())
    }

    fn generate<'s>(selection: &Selections<'s>) -> Result<'s, Self::Target<'s>> {
        Ok(selection
            .sum
            .iter()
            .rev()
            .enumerate()
            .map(|(fac, c)| (c.to_digit(10).unwrap() as usize) * 10usize.pow(fac as _))
            .sum())
    }
}
