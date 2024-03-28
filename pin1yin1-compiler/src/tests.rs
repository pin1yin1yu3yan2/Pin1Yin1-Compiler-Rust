use inkwell::{context::Context, execution_engine::JitFunction};

use pin1yin1_ast::{parse::do_parse, semantic::Global};
use pin1yin1_parser::{Parser, Source};
use pyir::ir::{Statement, Statements};

use crate::compile::CodeGen;

fn get_ast(src: &str) -> Statements {
    let source = Source::from_iter("compiler_test.py1", src.chars());
    let mut parser = Parser::<char>::new(source);

    let pus = do_parse(&mut parser)
        .map_err(|e| parser.handle_error(e))
        .unwrap();

    let mut global = Global::new();
    global
        .load(&pus)
        .map_err(|e| parser.handle_error(e))
        .unwrap();
    global.finish()
}

fn compile_tester(src: &str, tester: impl FnOnce(CodeGen)) {
    let ast = get_ast(src);
    let context = Context::create();
    let compiler = CodeGen::new(&context, "test", &ast).unwrap();

    tester(compiler);
}

const TEST_SRC1: &str = "
zheng3 jia can1 zheng3 x jie2
han2
    zheng3 jie2guo3 deng3 x jia1 1 fen1
    fan3 jie2guo3 fen1
jie2

zheng3 jian can1 zheng3 x jie2
han2
    fan3 x jian3 1 fen1
jie2
";

#[test]
fn jia_jian_around_114514() {
    compile_tester(TEST_SRC1, |compiler| unsafe {
        type TestFn = unsafe extern "C" fn(i64) -> i64;

        let jia: JitFunction<TestFn> = compiler.execution_engine.get_function("jia").unwrap();
        let jian: JitFunction<TestFn> = compiler.execution_engine.get_function("jian").unwrap();

        assert_eq!(jia.call(114513), 114514);
        assert_eq!(jian.call(114515), 114514);
    })
}

#[test]
fn serde_test() {
    let ast = get_ast(TEST_SRC1);

    let str1 = serde_json::to_string(&ast).unwrap();
    let ast1: Vec<Statement> = serde_json::from_str(&str1).unwrap();

    let str2 = serde_json::to_string(&ast).unwrap();
    let ast2: Vec<Statement> = serde_json::from_str(&str1).unwrap();

    assert_eq!(ast, ast1);
    assert_eq!(ast, ast2);
    assert_eq!(str1, str2);
}

const TEST_SRC2: &str = "
zheng3 jia can1 zheng3 x jie2
han2
    zheng3 jie2guo3 deng3 x jia1 1 fen1
    fan3 jie2guo3 fen1
jie2

zheng3 jia2 can1 zheng3 x jie2
han2
    zheng3 jie2guo3 deng3 
        jia can1 x jia1 1 jie2 fen1
    fan3 jie2guo3 fen1
jie2   
";

#[test]
fn fn_call() {
    compile_tester(TEST_SRC2, |tester| unsafe {
        type TestFn = unsafe extern "C" fn(i64) -> i64;

        let jia: JitFunction<TestFn> = tester.execution_engine.get_function("jia").unwrap();
        let jia2: JitFunction<TestFn> = tester.execution_engine.get_function("jia2").unwrap();

        assert_eq!(jia.call(114513), 114514);
        assert_eq!(jia2.call(114512), 114514);
    })
}
