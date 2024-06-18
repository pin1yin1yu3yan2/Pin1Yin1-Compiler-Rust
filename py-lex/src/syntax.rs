crate::reverse_parse_keywords! {
    keywords StructsDefinition {
        "lei4"     -> Classs,
        "mei2"     -> Enum,
        "lian2"    -> Union,
        "jie2gou4" -> Struct,
    }
    keywords Symbol {
        "dao3chu1" -> Export,

        "ya1"      -> FnCallL,
        "ru4"      -> FnCallR,

        "jie2"     -> BracketL,
        "he2"      -> BracketR,

        "zu3"      -> ArrayL,
        "he2"      -> ArrayR,

        "han2"     -> Block,
        "can1"     -> Parameter,
        "shi4"     -> Comment,
        "jie2"     -> EndOfBlock,

        "fen1"     -> Semicolon,
        "wei2"     -> Assign,
        "de1"      -> GetElement,
        "biao1"    -> Label,
        "wen2"     -> Char,
        "chuan4"   -> String,
    }
    keywords ControlFlow {
        "ruo4"      -> If,
        "ze2"       -> Else,
        "chong2"    -> Repeat,
        "qie4huan4" -> Switch,
        "tiao4"     -> Jump,
        "fan3"      -> Return,
    }
}
