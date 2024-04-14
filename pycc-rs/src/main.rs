pub mod compile;
pub mod primitive;
pub mod scope;
#[cfg(test)]
mod tests;

use crate::compile::CodeGen;
use inkwell::{context::Context, execution_engine::JitFunction};
use py_ast::{
    parse::do_parse,
    semantic::{mangle::DefaultMangler, ModScope},
};
use py_ir::ir::{Statement, Statements};
use terl::*;

fn main() {
    let path = "/home/yiyue/Pin1Yin1-Compiler-Rust/test.py1";
    let src = std::fs::read_to_string(path).unwrap();

    let source = Source::from_iter(path, src.chars());
    let mut parser = Parser::<char>::new(source);

    let pus = do_parse(&mut parser)
        .map_err(|e| eprintln!("{}", parser.handle_error(e.error()).unwrap()))
        .map_err(|_| println!("{}", parser.get_calling_tree()))
        .unwrap();

    let context = Context::create();

    let mut scope = ModScope::<DefaultMangler>::new_with_main();
    scope
        .load_fns(&pus)
        .map_err(|e| eprintln!("{}", parser.handle_error(e).unwrap()))
        .unwrap();

    let ast: Vec<_> = match scope.finish() {
        Ok(fn_defs) => fn_defs.into_iter().map(Statement::from).collect(),
        Err(errors) => {
            for err in errors {
                eprintln!("{}", parser.handle_error(err).unwrap());
            }
            panic!()
        }
    };

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
