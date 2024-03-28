/// use to define some keyword
///
/// you should only use at most one keywords! macro in a mod
#[macro_export]
macro_rules! keywords {
    ($(
        $(#[$metas:meta])*
        keywords $enum_name:ident
        { $(
            $string:literal -> $var:ident,
        )*}
    )*) => {
        $(
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $(#[$metas])*
        pub enum $enum_name {
            $(
                $var,
            )*
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    $(Self::$var => write!(f, "{}", $string),)*
                }
            }
        }

        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut pin1yin1_parser::Parser) -> pin1yin1_parser::ParseResult<Self> {
                use std::collections::HashMap;
                use pin1yin1_parser::WithSpan;

                $crate::lazy_static::lazy_static! {
                    static ref MAP: HashMap<Vec<char>, $enum_name> = {
                        let mut _map = HashMap::new();
                        $(
                            _map.insert($string.chars().collect::<Vec<_>>(), $enum_name::$var);
                        )*
                        _map
                    };
                }


                let s = p.get_chars()?;
                let s = &**s;
                let opt = MAP.get(s).copied().map(|t| p.make_pu(t));

                let error = || p.make_error(format!("non of {} matched", stringify!($enum_name)),pin1yin1_parser::ErrorKind::Unmatch);
                opt.ok_or_else(error)
            }
        }

        )*


        $crate::lazy_static::lazy_static! {
            pub static ref KEPPING_KEYWORDS: std::collections::HashSet<Vec<char>> = {
                let mut set = std::collections::HashSet::<Vec<char>>::default();
                $($(
                    set.insert($string.chars().collect::<Vec<_>>());
                )*)*
                set
            };
        }
    };
}

#[macro_export]
macro_rules! operators {
    (
        $(#[$metas:meta])*
        $(
            symbols $sub_class:ident {
                $($string:literal -> $var:ident : $ass:ident $priority:expr),*
            }
        )*

    ) => {
        $(#[$metas])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OperatorAssociativity {
            Binary,
            Unary,
        }

        $(#[$metas])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OperatorTypes {
            $($sub_class,)*
        }

        $crate::keywords! {
            $(#[$metas])*
            keywords Operators {
                $(
                    $($string -> $var,)*
                )*
            }
        }

        impl Operators {
            pub fn ty(&self) -> OperatorTypes {
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
