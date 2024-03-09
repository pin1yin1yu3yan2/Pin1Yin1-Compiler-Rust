#[macro_export]
macro_rules! keywords {
    ($(
        keywords $enum_name:ident
        { $(
            $string:literal -> $var:ident
        ),*}
    )*) => {
        $(
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $(
                $var,
            )*
        }

        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target<'t> = $enum_name;

            fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
                use std::collections::HashMap;
                lazy_static::lazy_static! {
                    static ref MAP: HashMap<&'static str,$enum_name> = {
                        let mut _map = HashMap::new();
                        $(
                            _map.insert($string, $enum_name::$var);
                        )*
                        _map
                    };
                }


                let s: &str = &p.parse::<String>()?;
                MAP.get(s).copied().map(|t| p.gen_token(t)).ok_or(None)
            }
        }
        )*
    };
}
