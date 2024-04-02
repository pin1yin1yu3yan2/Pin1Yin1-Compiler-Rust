use inkwell::{context::Context, execution_engine::JitFunction};

use py_ast::{parse::do_parse, semantic::GLobalScope};
use py_ir::ir::{Statement, Statements};
use terl::{Parser, Source};

use crate::compile::CodeGen;

fn get_ast(src: &str) -> Statements {
    let source = Source::from_iter("compiler_test.py1", src.chars());
    let mut parser = Parser::<char>::new(source);

    let pus = do_parse(&mut parser)
        .map_err(|e| parser.handle_error(e).unwrap())
        .map_err(|e| eprintln!("{e}"))
        .unwrap();

    let mut global = GLobalScope::new();
    global
        .load(&pus)
        .map_err(|e| parser.handle_error(e).unwrap())
        .map_err(|e| eprintln!("{e}"))
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

const MORE_OPERATOES: &str = "
zheng3 yi can1 zheng3 x jie2
han2
    fan3 x zuo3yi2 2 fen1
jie2

fu2 cheng can1 fu2 x jie2
han2
    fan3 x cheng2 2 fen1
jie2
";

#[test]
fn more_operations() {
    compile_tester(MORE_OPERATOES, |tester| unsafe {
        type Cheng2 = unsafe extern "C" fn(f32) -> f32;
        type Yi = unsafe extern "C" fn(i64) -> i64;

        let cheng2: JitFunction<Cheng2> = tester.execution_engine.get_function("cheng").unwrap();
        let yi: JitFunction<Yi> = tester.execution_engine.get_function("yi").unwrap();

        assert_eq!(cheng2.call(114514.0 / 2.0), 114514.0);
        assert_eq!(yi.call(114514 / 2), 114514);
    })
}
