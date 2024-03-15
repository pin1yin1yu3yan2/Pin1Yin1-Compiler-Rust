pub mod ast;
pub mod keywords;
pub mod macros;
pub mod semantic;

#[cfg(test)]
fn parse_test(chars: &str, tester: impl FnOnce(&mut pin1yin1_parser::Parser)) {
    use pin1yin1_parser::Source;

    let source = Source::new("test.py1", chars.chars());
    let mut parser = pin1yin1_parser::Parser::<'_, char>::new(&source);
    tester(&mut parser);
}
