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

        impl terl::ParseUnit for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut terl::Parser) -> terl::ParseResult<Self> {
                use std::collections::HashMap;
                use terl::WithSpan;

                thread_local! {
                    static MAP: HashMap<Vec<char>, $enum_name> = {
                        let mut _map = HashMap::new();
                        $(
                            _map.insert($string.chars().collect::<Vec<_>>(), $enum_name::$var);
                        )*
                        _map
                    };
                }


                let s = p.get_chars()?;
                let s = &**s;
                let opt = MAP.with(|map| map.get(s).copied()).map(|t| p.make_pu(t));

                let error = || p.make_parse_error(format!("non of {} matched", stringify!($enum_name)),terl::ParseErrorKind::Unmatch);
                opt.ok_or_else(error)
            }
        }

        )*


        thread_local! {
            pub static KEPPING_KEYWORDS: std::collections::HashSet<Vec<char>> = {
                let mut set = std::collections::HashSet::<Vec<char>>::default();
                $($(
                    set.insert($string.chars().collect::<Vec<_>>());
                )*)*
                set
            };
        }
    };
}
