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
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

        #[cfg(feature = "parser")]
        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut pin1yin1_parser::Parser) -> pin1yin1_parser::ParseResult<Self> {
                use std::collections::HashMap;
                use pin1yin1_parser::WithSpan;

                lazy_static::lazy_static! {
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


        lazy_static::lazy_static! {
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
