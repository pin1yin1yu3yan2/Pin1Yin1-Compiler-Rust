use std::any::Any;

use crate::{
    error::{ParseResult, Result},
    parser::{Parser, Selector},
    tokens::{Selection, Token},
};

pub trait ParseUnit: Any {
    type Target<'t>;

    fn select(selector: &mut Selector);

    fn generate(selection: Selection) -> Result<'_, Self::Target<'_>>;

    fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self>
    where
        Self: Sized,
    {
        let selector = p.select(Self::select);
        let selection = selector.selection.unwrap_or_else(|| {
            panic!(
                "{} selected nothing!",
                std::any::type_name_of_val(&Self::select)
            )
        });
        let location = selector.location.unwrap();
        let target = Self::generate(selection)?;
        Ok(Token::new(location, selection, target))
    }
}

impl ParseUnit for String {
    type Target<'t> = String;

    fn select(selector: &mut Selector) {
        selector
            .skip_whitespace()
            .take_while(|s| s.is_ascii_alphanumeric())
    }

    fn generate(selection: Selection) -> Result<'_, Self::Target<'_>> {
        Ok(selection.iter().collect())
    }
}

impl ParseUnit for usize {
    type Target<'t> = usize;

    fn select(selector: &mut Selector) {
        selector
            .skip_whitespace()
            .take_while(|c| c.is_ascii_digit())
    }

    fn generate(selection: Selection) -> Result<'_, Self::Target<'_>> {
        Ok(selection
            .iter()
            .rev()
            .enumerate()
            .map(|(fac, c)| (c.to_digit(10).unwrap() as usize) * 10usize.pow(fac as _))
            .sum())
    }
}
