pub mod compile;
pub mod primitive;
pub mod scope;
pub mod types;

use compile::CodeGen;
use inkwell::context::Context;
use pin1yin1_ast::{ast::Statements, parse::do_parse, semantic::Global};
use pin1yin1_parser::*;

fn main() {
    let path = "/home/yiyue/Pin1Yin1-rustc/test.py1";
    let src = std::fs::read_to_string(path).unwrap();

    let source = Source::new(path, src.chars());
    let mut parser = Parser::<'_, char>::new(&source);

    let pus = do_parse(&mut parser).to_result().unwrap();

    let context = Context::create();
    let mut compiler = CodeGen::new(&context, "test").unwrap();

    let mut global = Global::new();
    global.load(&pus).to_result().unwrap();

    let ast = global.finish();

    let str = serde_json::to_string(&ast).unwrap();
    let ast: Statements = serde_json::from_str(&str).unwrap();
    let str = serde_json::to_string(&ast).unwrap();
    std::fs::write("test.json", str).unwrap();
}
