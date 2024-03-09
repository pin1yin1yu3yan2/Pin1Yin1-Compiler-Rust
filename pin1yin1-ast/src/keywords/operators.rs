use crate::keywords;

/// however, there are **NO SYMBOL** in Pin1Yin1Yu3Yan2
#[allow(unused_macros)]
macro_rules! symbols {
    (symbols $enum_name:ident {$($str:literal -> $name:ident),*}) => {
        pub enum $enum_name {
            $(
                $name,
            )*
        }

        impl std::str::FromStr for $enum_name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $str => Ok($enum_name::$name),
                    )*
                    _ => Err(()),
                }
            }
        }
        impl ParseUnit for $enum_name {
            type Target<'t> = $enum_name;

            fn select(selector: &mut pin1yin1_parser::Selector) {
                let mut str = String::new();
                while selector.peek().is_some() {
                    str.extend(selector.next());
                    if str.parse::<$enum_name>().is_err() {
                        str.pop();
                        selector.next_back();
                        return;
                    }
                }
            }

            fn generate(
                selection: pin1yin1_parser::Selection,
            ) -> pin1yin1_parser::Result<'_, Self::Target<'_>> {
                selection
                    .iter()
                    .collect::<String>()
                    .parse::<$enum_name>()
                    .map_err(|_| unreachable!())
            }
        }

    };
}

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
