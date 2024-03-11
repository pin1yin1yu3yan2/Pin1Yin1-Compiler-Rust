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
                    static ref MAP: HashMap<Vec<char>, $enum_name> = {
                        let mut _map = HashMap::new();
                        $(
                            _map.insert($string.chars().collect::<Vec<_>>(), $enum_name::$var);
                        )*
                        _map
                    };
                }

                let s: &[char] = &p.parse::<&[char]>()?;
                MAP.get(s).copied().map(|t| p.new_token(t)).ok_or(None)
            }
        }
        )*
    };
}

#[macro_export]
macro_rules! complex_pu {
    (cpu $enum_name:ident {
        $($variant:ident),*
    }) => {
        #[derive(Debug, Clone)]
        pub enum $enum_name<'s> {
            $(
                $variant($variant<'s>),
            )*
        }

        $(
        impl<'s> From<$variant<'s>> for $enum_name<'s> {
             fn from(v: $variant<'s>) -> $enum_name<'s> {
                <$enum_name>::$variant(v)
            }
        }
        )*

        impl pin1yin1_parser::ParseUnit for $enum_name<'_> {
            type Target<'t> = $enum_name<'t>;

            fn parse<'s>(p: &mut Parser<'s>) -> ParseResult<'s, Self> {
                pin1yin1_parser::Try::new(p)
                $(
                    // whats the meaning of `tae` ???
                    .or_try::<Self, _>(|p| {
                        p.parse::<$variant>()
                            .map(|tae| tae.map(<$enum_name>::$variant))
                    })
                )*
                .finish()
            }
        }
    };
}
