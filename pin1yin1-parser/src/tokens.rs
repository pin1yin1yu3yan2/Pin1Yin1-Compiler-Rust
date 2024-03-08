use crate::{
    error::Result,
    parse_unit::ParseUnit,
    parser::{Location, Selection, Selector},
};
use std::{collections::HashMap, fmt::Debug};

pub struct Token<'s, P: ParseUnit> {
    location: Location<'s>,
    selection: Selection<'s>,
    inner: P::Target<'s>,
}

impl<'s, P: ParseUnit> Token<'s, P> {
    pub fn new(location: Location<'s>, selection: Selection<'s>, inner: P::Target<'s>) -> Self {
        Self {
            location,
            selection,
            inner,
        }
    }
}

impl<'s, P: ParseUnit> Debug for Token<'s, P>
where
    P::Target<'s>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("location", &self.location)
            .field("selection", &self.selection)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'s, P: ParseUnit> Clone for Token<'s, P>
where
    P::Target<'s>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            location: self.location,
            selection: self.selection,
            inner: self.inner.clone(),
        }
    }
}

impl<'s, P: ParseUnit> Copy for Token<'s, P> where P::Target<'s>: Copy {}

impl<'s, P: ParseUnit> std::ops::Deref for Token<'s, P> {
    type Target = P::Target<'s>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: ParseUnit> std::ops::DerefMut for Token<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

macro_rules! keywords {
    (keywords $enum_name:ident{ $($string:literal -> $var:ident),*}) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $enum_name {
            $(
                $var,
            )*
        }

        impl ParseUnit for $enum_name {
            type Target<'t> = $enum_name;
            fn select(selector: &mut Selector) {
                String::select(selector)
            }
            fn generate(selection: Selection) -> Result<'_, Self::Target<'_>> {
                lazy_static::lazy_static! {
                    static ref MAP: HashMap<&'static str,$enum_name> = {
                        let mut _map = HashMap::new();
                        $(
                            _map.insert($string, $enum_name::$var);
                        )*
                        _map
                    };
                }
                let str: &str= &String::generate(selection)?;
                MAP.get(str).copied().ok_or(None)
            }
            // fn select<'s>(p: &mut Parser<'s>) -> ParseResult<'s,Self> {



            // }
        }
    };
}

keywords! {
    keywords PrimitiveTypes {
        "zheng3" -> Integer,
        "fu2" -> Float,
        "zi4" -> Char,
        "bu4" -> Bool,
        "xu1" -> Complex
    }
}

#[test]
fn feature() {
    let chars = "zheng3 xu1".chars().collect::<Vec<_>>();
    let mut parser = crate::parser::Parser::new(&chars);

    dbg!(PrimitiveTypes::parse(&mut parser)).ok();
    dbg!(PrimitiveTypes::parse(&mut parser)).ok();
}
