pub mod compile;
pub mod primitive;
pub mod scope;
#[cfg(test)]
mod tests;

use crate::compile::CodeGen;
use inkwell::{context::Context, execution_engine::JitFunction};
use py_ast::{parse::do_parse, semantic::Global};
use pyir::ir::Statements;
use terl::*;

fn main() {
    let path = "/home/yiyue/Pin1Yin1-rustc/test.py1";
    let src = std::fs::read_to_string(path).unwrap();

    let source = Source::from_iter(path, src.chars());
    let mut parser = Parser::<char>::new(source);

    let pus = do_parse(&mut parser)
        .map_err(|e| parser.handle_error(e))
        .unwrap();

    let context = Context::create();

    let mut global = Global::new();
    global
        .load(&pus)
        .map_err(|e| parser.handle_error(e))
        .unwrap();

    let ast = global.finish();

    let str = serde_json::to_string(&ast).unwrap();
    let ast: Statements = serde_json::from_str(&str).unwrap();
    let str = serde_json::to_string(&ast).unwrap();
    let ast: Statements = serde_json::from_str(&str).unwrap();

    std::fs::write("test.json", str).unwrap();

    let compiler = CodeGen::new(&context, "test", &ast).unwrap();

    std::fs::write("test.ll", compiler.llvm_ir()).unwrap();

    unsafe {
        type TestFn = unsafe extern "C" fn(i64) -> i64;

        let jia: JitFunction<TestFn> = compiler.execution_engine.get_function("jia").unwrap();
        let jian: JitFunction<TestFn> = compiler.execution_engine.get_function("jian").unwrap();

        assert_eq!(jia.call(114513), 114514);
        assert_eq!(jian.call(114515), 114514);
    }
}
