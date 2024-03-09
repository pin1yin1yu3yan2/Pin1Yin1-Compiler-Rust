use crate::keywords;

// /// however, there are **NO SYMBOL** in Pin1Yin1Yu3Yan2
// #[allow(unused_macros)]
// macro_rules! symbols {}

keywords! {
    keywords AlgebraOperator {
        "jia1"   -> Add,
        "jian3"  -> Sub,
        "cheng2" -> Mul,
        "chu2"   -> Div,
        "mi4"    -> Pow,
        "Dui4"   -> Log
    }
    keywords CompareOperator {
        "tong2"      -> Eq,
        "fei1tong2"  -> Neq,
        "da4"        -> Gt,
        "xiao3"      -> Lt,
        "da4deng3"   -> Ge,
        "xiao3deng3" -> Le
    }
    keywords LogicalOperator {
        "yu3"  -> And,
        "huo4" -> Or,
        "fei1" -> Not
    }
    keywords ArithmeticOperator {
        "wei4yu3"     -> Band,
        "wei4huo4"    -> Bor,
        "wei4fei1"    -> Bnot,
        "wei4yi4huo4" -> Xor,
        "zuo3yi2"     -> Shl,
        "you4yi2"     -> Shr
    }
    keywords SpecialOperator {
        "qu3zhi3"   -> AddrOf,
        "fang3zhi3" -> Deref,
        "fang3su4"  -> GetElement,
        "zhuan3"    -> Cast,
        "chang2du4" -> SizeOf
    }
}
