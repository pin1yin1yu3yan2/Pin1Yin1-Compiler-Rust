pub mod lex;
mod macros;
pub mod parse;
pub mod semantic;

use pyir::ir;
use pyir::ops;

#[cfg(test)]
fn parse_test(chars: &str, tester: impl FnOnce(&mut pin1yin1_parser::Parser)) {
    use pin1yin1_parser::Source;

    let source = Source::from_iter("test.py1", chars.chars());
    let mut parser = pin1yin1_parser::Parser::<char>::new(source);
    tester(&mut parser);
}
