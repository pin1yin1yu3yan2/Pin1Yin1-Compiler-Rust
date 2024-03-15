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
        #[cfg_attr(feature = "ser", derive(serde::Serialize,serde::Deserialize))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $(#[$metas])*
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
        pub mod defaults {
            $(

            #[allow(non_snake_case)]
            pub mod $enum_name {
                $(

                pub fn $var<'s>() -> pin1yin1_parser::PU<'s, super::super::$enum_name> {
                    pin1yin1_parser::PU::new_without_selection(super::super::$enum_name::$var)
                }

                )*
            }
            )*
        }


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
macro_rules! complex_pu {
    (
        $(#[$metas:meta])*
        cpu $enum_name:ident {
        $(
            $(#[$v_metas:meta])*
            $variant:ident
        ),*
    }) => {
        #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "ser", serde(bound(deserialize = "'s: 'de, 'de: 's")))]
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
