macro_rules! Operators {
    (
        $(
            symbols $sub_class:ident {
                $($string:literal -> $var:ident : $ass:ident),*
            }
        )*

    ) => {
        #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OperatorAssociativity {
            Binary,
            Unary,
        }

        #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OperatorTypes {
            $($sub_class,)*
        }

        $crate::keywords! {
            keywords Operators {
                $(
                    $($string -> $var,)*
                )*
            }
        }

        impl Operators {
            pub fn type_(&self) -> OperatorTypes {
                match *self {
                    $(
                        $(Self::$var => OperatorTypes::$sub_class,)*
                    )*
                }
            }

            pub fn associativity(&self) -> OperatorAssociativity {
                match *self {
                    $(
                        $(Self::$var => OperatorAssociativity::$ass,)*
                    )*
                }
            }
        }

        pub mod sub_classes {
            use super::*;

            $crate::keywords! {
                $(
                keywords $sub_class {
                    $($string -> $var,)*
                }
                )*
            }

            $(
            impl From<$sub_class> for Operators {
                fn from(value: $sub_class) -> Operators {
                    match value {
                        $($sub_class::$var => Operators::$var,)*
                    }
                }
            }

            impl TryFrom<Operators> for $sub_class {
                type Error = ();

                fn try_from(value: Operators) -> Result<Self, Self::Error> {
                    match value {
                        $(Operators::$var => Ok(Self::$var),)*
                        _ => Err(())
                    }
                }
            }

            impl $sub_class {
                pub fn associativity(&self) -> OperatorAssociativity {
                    match self {
                        $(Self::$var => OperatorAssociativity::$ass,)*
                    }
                }
            }

        )*
        }
    };
}

Operators! {
    symbols AlgebraOperator {
        "jia1"   -> Add : Binary,
        "jian3"  -> Sub : Binary,
        "cheng2" -> Mul : Binary,
        "chu2"   -> Div : Binary,
        "mi4"    -> Pow : Binary,
        "Dui4"   -> Log : Binary
    }
    symbols CompareOperator {
        "tong2"      -> Eq  : Binary,
        "fei1tong2"  -> Neq : Binary,
        "da4"        -> Gt  : Binary,
        "xiao3"      -> Lt  : Binary,
        "da4deng3"   -> Ge  : Binary,
        "xiao3deng3" -> Le  : Binary
    }
    symbols LogicalOperator {
        "yu3"  -> And : Binary,
        "huo4" -> Or  : Binary,
        "fei1" -> Not : Unary
    }
    symbols ArithmeticOperator {
        "wei4yu3"     -> Band : Binary,
        "wei4huo4"    -> Bor  : Binary,
        "wei4fei1"    -> Bnot : Unary,
        "wei4yi4huo4" -> Xor  : Binary,
        "zuo3yi2"     -> Shl  : Binary,
        "you4yi2"     -> Shr  : Binary
    }
    symbols SpecialOperator {
        "qu3zhi3"   -> AddrOf     : Unary,
        "fang3zhi3" -> Deref      : Unary,
        "fang3su4"  -> GetElement : Binary,
        "zhuan3"    -> Cast       : Unary,
        "chang2du4" -> SizeOf     : Unary
    }
}
