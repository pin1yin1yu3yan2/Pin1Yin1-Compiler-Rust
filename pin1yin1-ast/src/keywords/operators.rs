macro_rules! Operators {
    (
        $(
            symbols $sub_class:ident {
                $($string:literal -> $var:ident : $ass:ident $priority:expr),*
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

            pub fn priority(&self) -> usize {
                match *self {
                    $(
                        $(Self::$var => $priority,)*
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

                pub fn priority(&self) -> usize {
                    match self {
                        $(Self::$var => $priority,)*
                    }
                }
            }

        )*
        }
    };
}

Operators! {
    symbols AlgebraOperator {
        "jia1"   -> Add : Binary 6,
        "jian3"  -> Sub : Binary 6,
        "cheng2" -> Mul : Binary 5,
        "chu2"   -> Div : Binary 5,
        "mo2"    -> Mod : Binary 5,
        "mi4"    -> Pow : Binary 4,
        "dui4"   -> Log : Binary 4
    }
    symbols CompareOperator {
        "tong2"      -> Eq  : Binary 10,
        "fei1tong2"  -> Neq : Binary 10,
        "da4"        -> Gt  : Binary 8,
        "xiao3"      -> Lt  : Binary 8,
        "da4deng3"   -> Ge  : Binary 8,
        "xiao3deng3" -> Le  : Binary 8
    }
    symbols LogicalOperator {
        "yu3"  -> And : Binary 14,
        "huo4" -> Or  : Binary 15,
        "fei1" -> Not : Unary  3
    }
    symbols ArithmeticOperator {
        "wei4yu3"     -> Band : Binary 11,
        "wei4huo4"    -> Bor  : Binary 13,
        "wei4fei1"    -> Bnot : Unary  3,
        "wei4yi4huo4" -> Xor  : Binary 12,
        "zuo3yi2"     -> Shl  : Binary 7,
        "you4yi2"     -> Shr  : Binary 7
    }
    symbols SpecialOperator {
        "qu3zhi3"   -> AddrOf     : Unary  3,
        "fang3zhi3" -> Deref      : Unary  3,
        "fang3su4"  -> GetElement : Binary 2,
        "zhuan3"    -> Cast       : Unary  2,
        "chang2du4" -> SizeOf     : Unary  3
    }
}
