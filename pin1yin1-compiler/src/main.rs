use pin1yin1_ast::ast::types::TypeDeclare;
use pin1yin1_parser::Parser;

fn main() {
    let chars = "yin3 zu3 114514 kuan1 32 wu2fu2 zheng3"
        .chars()
        .collect::<Vec<_>>();
    let mut parser = Parser::new(&chars);

    let type_declare = parser.parse::<TypeDeclare>().unwrap();

    println!("{}", serde_json::to_string(&type_declare).unwrap());
}
