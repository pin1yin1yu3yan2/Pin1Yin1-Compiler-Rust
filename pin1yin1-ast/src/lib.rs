pub mod ast;
pub mod keywords;
pub mod macros;

#[cfg(test)]
fn parse_test(chars: &str, tester: impl FnOnce(&mut pin1yin1_parser::Parser)) {
    let chars = chars.chars().collect::<Vec<_>>();
    let mut parser = pin1yin1_parser::Parser::new(&chars);
    tester(&mut parser);
}
