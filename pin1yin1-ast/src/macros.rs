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

        impl $enum_name {
            pub fn parse_or_unmatch<'s>(self, p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
                use pin1yin1_parser::WithSelection;
                p.parse::<Self>()
                    .match_or(|e| e.unmatch(format!("expect {} `{self}`, but unmatched", stringify!($enum_name))))
                    .eq_or(self, |t| t.unmatch(format!("expect `{self}`")))
            }

            pub fn parse_or_failed<'s>(self, p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
                self.parse_or_unmatch(p).must_match()
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    $(Self::$var => write!(f, "{}", $string),)*
                }
            }
        }

        #[cfg(feature = "parser")]
        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target<'t> = $enum_name;

            fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self> {
                use std::collections::HashMap;
                use pin1yin1_parser::WithSelection;

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
                let opt = MAP.get(s).copied().map(|t| p.make_token(t));
                pin1yin1_parser::ParseResult::from_option(opt,|| p.unmatch(format!("non of {} matched", stringify!($enum_name))))
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

/// use to define a complex parse unit which could be one of its variants

#[macro_export]
#[cfg(feature = "parser")]
macro_rules! complex_pu {
    (
        $(#[$metas:meta])*
        cpu $enum_name:ident {
        $(
            $(#[$v_metas:meta])*
            $variant:ident
        ),*
    }) => {
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub enum $enum_name<'s> {
            $(
                $(#[$v_metas])*
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

            fn parse<'s>(p: &mut pin1yin1_parser::Parser<'s>) -> pin1yin1_parser::ParseResult<'s, Self>
            {
                pin1yin1_parser::Try::new(p)
                //TODO: optimize chain calling, make less Parser::once call
                $(

                    .or_try::<Self, _>(|p| {
                        p.parse::<$variant>()
                            .map_pu(<$enum_name>::$variant)
                    })
                )*
                .finish()
            }
        }
    };
}
