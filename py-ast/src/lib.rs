mod macros;
pub mod parse;
pub mod semantic;

use py_ir as ir;

#[cfg(test)]
fn parse_test(
    chars: &str,
    tester: impl FnOnce(&mut terl::Parser<py_lex::Token>) + std::panic::UnwindSafe,
) {
    use terl::{Buffer, ResultMapperExt};

    let source = Buffer::new("test.py1".to_string(), chars.chars().collect());
    let parser = terl::Parser::<char>::new(source);
    let (_char_buffer, mut parser) = parser
        .process(|p| {
            let mut tokens = vec![];

            while let Some(token) = p.parse::<py_lex::Token>().apply(terl::mapper::Try)? {
                tokens.push(token);
            }
            Ok(tokens)
        })
        .unwrap_or_else(|_| unreachable!());

    tester(&mut parser);
}
