pub mod keywords;
pub mod macros;
pub mod syntax;

#[cfg(test)]
mod tests {

    use crate::keywords::types::PrimitiveType;
    use pin1yin1_parser::*;

    #[test]
    fn it_works() {
        let chars = "zheng3\nxu1".chars().collect::<Vec<_>>();
        let mut parser = Parser::new(&chars);

        let _zheng3 = PrimitiveType::parse(&mut parser).unwrap();
        let _xu1 = PrimitiveType::parse(&mut parser).unwrap();
    }
}
