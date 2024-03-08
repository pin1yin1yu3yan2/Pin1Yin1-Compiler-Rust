mod error;
mod parse_unit;
mod parser;
mod tokens;

pub use self::{error::*, parse_unit::*, parser::*, tokens::*};

#[cfg(test)]
pub fn test_parse(src: &str, tester: impl FnOnce(&mut Parser)) {
    let chars = src.chars().collect::<Vec<_>>();
    let mut parser = Parser::new(&chars);
    tester(&mut parser);
}
