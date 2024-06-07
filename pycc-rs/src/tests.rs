use py_codegen::Backend;
use py_codegen_llvm::{
    inkwell::{
        execution_engine::{ExecutionEngine, JitFunction},
        OptimizationLevel,
    },
    LLVMBackend,
};
use py_ir as ir;

fn test_generate_ir(src: &str) -> Vec<ir::Item> {
    let (error_handler, ast) = crate::generate_ast("compiler-test.py1".to_owned(), src.to_owned());
    let error_handler = (&error_handler.0, &error_handler.1);
    crate::generate_ir(error_handler, &ast)
}

fn compile_tester(src: &str, tester: impl FnOnce(&ExecutionEngine)) {
    let ir = test_generate_ir(src);
    let backend = LLVMBackend::init(());
    let module = backend.module(src, &ir).unwrap();
    let ee = module
        .create_jit_execution_engine(OptimizationLevel::Default)
        .unwrap();
    tester(&ee);
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
    compile_tester(TEST_SRC1, |ee| unsafe {
        type TestFn = unsafe extern "C" fn(i64) -> i64;

        let jia: JitFunction<TestFn> = ee.get_function("jia 参 i64 结").unwrap();
        let jian: JitFunction<TestFn> = ee.get_function("jian 参 i64 结").unwrap();

        fn native_jia(n: i64) -> i64 {
            n + 1
        }
        fn native_jian(n: i64) -> i64 {
            n - 1
        }

        for n in 114514..1919810 {
            assert_eq!(jia.call(n), native_jia(n));
            assert_eq!(jian.call(n), native_jian(n));
        }
    })
}

#[test]
fn serde_test() {
    let mir = test_generate_ir(MORE_OPERATOES);

    let str1 = serde_json::to_string(&mir).unwrap();
    let ast1: Vec<ir::Item> = serde_json::from_str(&str1).unwrap();

    let str2 = serde_json::to_string(&ast1).unwrap();
    let ast2: Vec<ir::Item> = serde_json::from_str(&str1).unwrap();

    let str3 = serde_json::to_string(&ast2).unwrap();
    let _ast3: Vec<ir::Item> = serde_json::from_str(&str1).unwrap();

    assert_eq!(str1, str2);
    assert_eq!(str2, str3);
}

const MORE_OPERATOES: &str = "
fu2 cheng can1 fu2 x jie2
han2
    fu2 ret wei2 x cheng2 2f0 fen1
    fan3 ret fen1
jie2

zheng3 yi can1 zheng3 x jie2
han2
    fan3 x zuo3yi2 1 fen1
jie2
";

#[test]
fn more_operations() {
    compile_tester(MORE_OPERATOES, |ee| unsafe {
        type Cheng = unsafe extern "C" fn(f32) -> f32;
        type Yi = unsafe extern "C" fn(i64) -> i64;

        let cheng: JitFunction<Cheng> = ee.get_function("cheng 参 f32 结").unwrap();
        let yi: JitFunction<Yi> = ee.get_function("yi 参 i64 结").unwrap();

        fn native_cheng(x: f32) -> f32 {
            x * 2.0
        }
        fn native_yi(x: i64) -> i64 {
            x << 1
        }

        for n in 114514..1919810 {
            assert_eq!(cheng.call(n as f32), native_cheng(n as f32));
            assert_eq!(yi.call(n), native_yi(n));
        }
    })
}

const OVERLOAD_TEST: &str = "
zheng3 a can1 zheng3 a jie2
han2
    fan3 a jia1 1 fen1
jie2

fu2 a can1 fu2 a jie2
han2
    fan3 a jia1 1f0 fen1
jie2

zheng3 test can1 zheng3 oac fen1 fu2 amin jie2
han2 
    ya1 oac ru4 a fen1
    ya1 amin  ru4 a fen1
    fan3 114514 fen1
jie2
";

#[test]
fn overload_test() {
    compile_tester(OVERLOAD_TEST, |ee| unsafe {
        type Test = unsafe extern "C" fn(i64, f32) -> i64;

        let test: JitFunction<Test> = ee.get_function("test 参 i64 f32 结").unwrap();

        assert_eq!(test.call(114514, 114514.0), 114514);
    })
}

const BASIC_CONTROL_FLOW: &str = "
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
    compile_tester(BASIC_CONTROL_FLOW, |ee| unsafe {
        type Fio = unsafe extern "C" fn(i64) -> i64;

        let py_fio: JitFunction<Fio> = ee.get_function("fio 参 i64 结").unwrap();

        fn native_fio(n: i64) -> i64 {
            match n {
                0 | 1 => 1,
                _ => native_fio(n - 1) + native_fio(n - 2),
            }
        }

        // too big range will make stack overflow
        for n in 0..20 {
            assert_eq!(py_fio.call(n), native_fio(n))
        }
    })
}
