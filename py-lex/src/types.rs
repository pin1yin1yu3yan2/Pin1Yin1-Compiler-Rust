crate::front_parse_keywords! {
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
        "she4"     -> Const,
        "zhi3"     -> Pointer,
    }
}
