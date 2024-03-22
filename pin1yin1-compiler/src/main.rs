pub mod compile;
pub mod types;

use pin1yin1_ast::{ast::Statements, parse::do_parse, semantic::definition_pool::GlobalPool};
use pin1yin1_parser::*;

fn main() {
    let path = "/home/twhice/Pin1Yin1-rustc/test.py1";
    let src = std::fs::read_to_string(path).unwrap();

    let source = Source::new(path, src.chars());
    let mut parser = Parser::<'_, char>::new(&source);

    let pus = do_parse(&mut parser).to_result().unwrap();

    let mut global = GlobalPool::new();
    global.load(&pus).to_result().unwrap();

    let ast = global.finish();

    let str = serde_json::to_string(&ast).unwrap();
    let ast: Statements = serde_json::from_str(&str).unwrap();
    let str = serde_json::to_string(&ast).unwrap();
    std::fs::write("test.json", str).unwrap();
}
