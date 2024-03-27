mod macros;
pub mod parse;
pub mod semantic;

use pin1yin1_ast::ast;
use pin1yin1_ast::keywords;

#[cfg(test)]
fn parse_test(chars: &str, tester: impl FnOnce(&mut pin1yin1_parser::Parser)) {
    use pin1yin1_parser::Source;

    let source = Source::from_iter("test.py1", chars.chars());
    let mut parser = pin1yin1_parser::Parser::<char>::new(source);
    tester(&mut parser);
}
