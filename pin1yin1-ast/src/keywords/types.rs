use crate::keywords;

keywords! {
    // keywords PrimitiveType {
    //     "zheng3" -> Integer,
    //     "fu2"    -> Float,
    //     "zi4"    -> Char,
    //     "bu4"    -> Bool,
    //     "xu1"    -> Complex,
    // }
    keywords BasicExtenWord {
        "zu3"      -> Array,
        "kuan1"    -> Width,
        "you3fu2"  -> Signed,
        "wu2fu2"   -> Unsigned,
        "yin3"     -> Reference,
        "you4yin3" -> RightReference,
        "she4"     -> Const,
        "zhi3"     -> Pointer,
    }
}

// PrimitiveType shouldn't be keeping keywords
// #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum PrimitiveTypes {
//     Integer,
//     Float,
//     Char,
//     Bool,
//     Complex,
// }
// impl pin1yin1_parser::ParseUnit for PrimitiveTypes {
//     type Target<'t> = PrimitiveTypes;
//     fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
//         use std::collections::HashMap;
//         lazy_static::lazy_static! {
//             static ref MAP:HashMap<Vec<char>,PrimitiveTypes>  = {
//                 let mut _map = HashMap::new();
//                 _map.insert("zheng3".chars().collect::<Vec<_>>(),PrimitiveTypes::Integer);
//                 _map.insert("fu2".chars().collect::<Vec<_>>(),PrimitiveTypes::Float);
//                 _map.insert("zi4".chars().collect::<Vec<_>>(),PrimitiveTypes::Char);
//                 _map.insert("bu4".chars().collect::<Vec<_>>(),PrimitiveTypes::Bool);
//                 _map.insert("xu1".chars().collect::<Vec<_>>(),PrimitiveTypes::Complex);
//                 _map
//             };
//         }
//         let s: &[char] = &p.parse::<&[char]>()?;
//         MAP.get(s).copied().map(|t| p.new_token(t)).ok_or(None)
//     }
// }
