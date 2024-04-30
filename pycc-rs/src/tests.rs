use inkwell::{context::Context, execution_engine::JitFunction};

use py_ast::{
    parse::do_parse,
    semantic::{mangle::DefaultMangler, ModScope},
};
use py_ir::ir::Item;
use terl::{Parser, Source};

use crate::compile::CodeGen;

fn get_ast(src: &str) -> Vec<Item> {
    let source = Source::from_iter("compiler_test.py1", src.chars());
    let mut parser = Parser::<char>::new(source);

    let pus = do_parse(&mut parser)
        .map_err(|e| eprintln!("{}", parser.handle_error(e.error()).unwrap()))
        .map_err(|_| println!("{}", parser.get_calling_tree()))
        .unwrap();

    let mut scope = ModScope::<DefaultMangler>::default();
    scope
        .load_fns(&pus)
        .map_err(|e| eprintln!("{}", parser.handle_error(e).unwrap()))
        .unwrap();

    match scope.finish() {
        Ok(fn_defs) => fn_defs.into_iter().map(Item::from).collect(),
        Err(errors) => {
            for err in errors {
                eprintln!("{}", parser.handle_error(err).unwrap());
            }
            panic!()
        }
    }
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
    zheng3 jie2guo3 wei2 x jia1 1 fen1
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

        let jia: JitFunction<TestFn> = compiler
            .execution_engine
            .get_function("jia 参 i64 结")
            .unwrap();
        let jian: JitFunction<TestFn> = compiler
            .execution_engine
            .get_function("jian 参 i64 结")
            .unwrap();

        assert_eq!(jia.call(114513), 114514);
        assert_eq!(jian.call(114515), 114514);
    })
}

#[test]
fn serde_test() {
    let ast = get_ast(MORE_OPERATOES);

    let str1 = serde_json::to_string(&ast).unwrap();
    let ast1: Vec<Item> = serde_json::from_str(&str1).unwrap();

    let str2 = serde_json::to_string(&ast).unwrap();
    let ast2: Vec<Item> = serde_json::from_str(&str1).unwrap();

    assert_eq!(ast, ast1);
    assert_eq!(ast, ast2);
    assert_eq!(str1, str2);
}

const TEST_SRC2: &str = "
zheng3 jia can1 zheng3 x jie2
han2
    zheng3 jie2guo3 wei2 x jia1 1 fen1
    fan3 jie2guo3 fen1
jie2

zheng3 jia2 can1 zheng3 x jie2
han2
    zheng3 jie2guo3 wei2
        ya1 x jia1 1 ru4 jia fen1
    fan3 jie2guo3 fen1
jie2   
";

#[test]
fn fn_call() {
    compile_tester(TEST_SRC2, |tester| unsafe {
        type TestFn = unsafe extern "C" fn(i64) -> i64;

        let jia: JitFunction<TestFn> = tester
            .execution_engine
            .get_function("jia 参 i64 结")
            .unwrap();
        let jia2: JitFunction<TestFn> = tester
            .execution_engine
            .get_function("jia2 参 i64 结")
            .unwrap();

        assert_eq!(jia.call(114513), 114514);
        assert_eq!(jia2.call(114512), 114514);
    })
}

const MORE_OPERATOES: &str = "
fu2 cheng can1 fu2 x jie2
han2
    fu2 ret wei2 x cheng2 2.0 fen1
    fan3 ret fen1
jie2

zheng3 yi can1 zheng3 x jie2
han2
    fan3 x zuo3yi2 1 fen1
jie2
";

#[test]
fn more_operations() {
    compile_tester(MORE_OPERATOES, |tester| unsafe {
        type Cheng = unsafe extern "C" fn(f32) -> f32;
        type Yi = unsafe extern "C" fn(i64) -> i64;

        let cheng: JitFunction<Cheng> = tester
            .execution_engine
            .get_function("cheng 参 f32 结")
            .unwrap();
        let yi: JitFunction<Yi> = tester
            .execution_engine
            .get_function("yi 参 i64 结")
            .unwrap();

        assert_eq!(cheng.call(57257.0), 114514.0);
        assert_eq!(yi.call(57257), 114514);
    })
}

const OVERLOAD_TEST: &str = "
zheng3 a can1 zheng3 a jie2
han2
    fan3 a jia1 1 fen1
jie2

fu2 a can1 fu2 a jie2
han2
    fan3 a jia1 1.0 fen1
jie2

zheng3 test can1 zheng3 fuck fen1 fu2 you jie2
han2 
    ya1 fuck ru4 a fen1
    ya1 you  ru4 a fen1
    fan3 114514 fen1
jie2
";

#[test]
fn overload_test() {
    compile_tester(OVERLOAD_TEST, |tester| unsafe {
        type A1 = unsafe extern "C" fn(i64) -> i64;
        type A2 = unsafe extern "C" fn(f32) -> f32;
        type Test = unsafe extern "C" fn(i64, f32) -> i64;

        let a1: JitFunction<A1> = tester.execution_engine.get_function("a 参 i64 结").unwrap();
        let a2: JitFunction<A2> = tester.execution_engine.get_function("a 参 f32 结").unwrap();
        let test: JitFunction<Test> = tester
            .execution_engine
            .get_function("test 参 i64 f32 结")
            .unwrap();
        assert_eq!(a1.call(114513), 114514);
        assert_eq!(a2.call(114513.0), 114514.0);
        assert_eq!(test.call(114514, 114514.0), 114514);
    })
}

const BASIC_CONTROL_FLOW: &str = "
zheng3 odd can1 zheng3 shu1ru4 jie2
han2 
    ruo4 can1 shu1ru4 mo2 2 tong2 0 jie2
    han2
        fan3 1 fen1
    jie2 ze2 han2 
        fan3 0 fen1
    jie2
jie2

zheng3 fio can1 zheng3 n jie2
han2
    ruo4 can1 n tong2 0 huo4 n tong2 1 jie2
    han2
        fan3 1 fen1
    jie2 ze2 han2
        fan3 ya1 n jian3 1 ru4 fio jia1  
             ya1 n jian3 2 ru4 fio fen1
    jie2
jie2
";

#[test]
fn control_flow_test() {
    compile_tester(BASIC_CONTROL_FLOW, |tester| unsafe {
        let ir_code = tester.llvm_ir();
        println!("{}", ir_code);

        type Odd = unsafe extern "C" fn(i64) -> i64;
        type Fio = unsafe extern "C" fn(i64) -> i64;

        let odd: JitFunction<Odd> = tester
            .execution_engine
            .get_function("odd 参 i64 结")
            .unwrap();
        let py_fio: JitFunction<Fio> = tester
            .execution_engine
            .get_function("fio 参 i64 结")
            .unwrap();

        fn native_fio(n: i64) -> i64 {
            match n {
                0 | 1 => 1,
                _ => native_fio(n - 1) + native_fio(n - 2),
            }
        }

        assert_eq!(odd.call(114), 1);
        assert_eq!(odd.call(514), 1);
        assert_eq!(odd.call(11), 0);

        // too big range will make stack overflow
        for n in 0..20 {
            assert_eq!(py_fio.call(n), native_fio(n))
        }
    })
}
