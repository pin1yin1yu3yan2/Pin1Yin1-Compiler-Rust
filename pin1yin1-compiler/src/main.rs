use pin1yin1_ast::{ast::*, semantic::check};
use pin1yin1_parser::*;

fn main() {
    let path = "/home/twhice/Pin1Yin1-rustc/test.py1";
    let src = std::fs::read_to_string(path).unwrap();

    let source = Source::new(path, src.chars());
    let mut parser = Parser::<'_, char>::new(&source);

    type Target<'t> = Vec<PU<'t, Statement<'t>>>;

    let pus: Target = do_parse(&mut parser).unwrap();

    // std::fs::write("test.json", string).unwrap();

    check(pus.into_iter().map(|pu| pu.take()));
}
