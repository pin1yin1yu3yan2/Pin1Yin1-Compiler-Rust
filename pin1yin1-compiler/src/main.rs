use pin1yin1_ast::ast::syntax::FunctionDefine;
use pin1yin1_parser::Parser;

fn main() {
    let src = std::fs::read_to_string("/home/twhice/Pin1Yin1-rustc/test.py1").unwrap();

    let chars = src.chars().collect::<Vec<_>>();
    let mut parser = Parser::new(&chars);

    type Target<'t> = FunctionDefine<'t>;

    let pu = parser.parse::<Target>().unwrap();
    let string = serde_json::to_string(&pu).unwrap();
    println!("{}", string);
    let pu = serde_json::from_str::<Target>(&string).unwrap();
    let string = serde_json::to_string(&pu).unwrap();

    std::fs::write("test.json", string).unwrap();
}
