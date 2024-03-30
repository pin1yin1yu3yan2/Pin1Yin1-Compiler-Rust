pub mod lex;
mod macros;
pub mod parse;
pub mod semantic;

use py_ir::ir;
use py_ir::ops;

#[cfg(test)]
fn parse_test(chars: &str, tester: impl FnOnce(&mut terl::Parser)) {
    use terl::Source;

    let source = Source::from_iter("test.py1", chars.chars());
    let mut parser = terl::Parser::<char>::new(source);

    tester(&mut parser);
}
