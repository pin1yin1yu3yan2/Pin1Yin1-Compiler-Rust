mod macros;
pub mod parse;
pub mod semantic;

use py_ir as ir;

#[cfg(test)]
fn parse_test(
    chars: &str,
    tester: impl FnOnce(&mut terl::Parser<py_lex::Token>) -> terl::Result<(), terl::ParseError>,
) {
    use terl::{Buffer, ResultMapperExt, Source};

    let source = Buffer::new("test.py1".to_string(), chars.chars().collect());
    let parser = terl::Parser::<char>::new(source);
    let (char_buffer, mut parser) = parser
        .process(|p| {
            let mut tokens = vec![];

            while let Some(token) = p.parse::<py_lex::Token>().apply(terl::mapper::Try)? {
                tokens.push(token);
            }
            Ok(tokens)
        })
        .unwrap_or_else(|_| unreachable!());

    if let Err(error) = tester(&mut parser) {
        let calling_tree = parser.calling_tree();
        eprintln!("{calling_tree}");
        eprintln!("error: {:?}", error);
        let error = py_lex::Token::handle_error(&(&char_buffer, parser.buffer()), error.error());
        eprintln!("{error}");
        panic!("panic as expected")
    }
}
