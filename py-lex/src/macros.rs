/// use to define some keyword
///
/// you should only use at most one keywords! macro in a mod
#[macro_export]
macro_rules! reverse_parse_keywords {
    ($(
        $(#[$metas:meta])*
        keywords $enum_name:ident
        { $(
            $string:literal -> $var:ident,
        )*}
    )*) => {
        $(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $(#[$metas])*
        pub enum $enum_name {
            $(
                $var,
            )*
        }

        impl std::ops::Deref for $enum_name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                match self {
                    $(Self::$var => $string,)*
                }
            }

        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
                f.write_str(self)
            }
        }

        #[cfg(feature = "parse")]
        impl terl::ReverseParseUnit<$crate::Token> for $enum_name {
            type Left = $enum_name;
            fn reverse_parse(&self, p:&mut terl::Parser<$crate::Token>) -> Result<$enum_name, terl::ParseError> {
                use terl::WithSpanExt;

                let Some(next) = p.next() else {
                    return p.unmatch(format!("expect {}, but non token left", self))
                };

                if &**next != &**self {
                    let msg = format!("expect {}, but {} was got", self, &**next);
                    return p.unmatch(msg)
                }

                Ok(*self)
            }
        }

        )*


        thread_local! {
            pub static KEPPING_KEYWORDS: std::collections::HashSet<&'static str> = {
                let mut set = std::collections::HashSet::<&'static str>::default();
                $($(
                    set.insert($string);
                )*)*
                set
            };
        }
    };
}

#[macro_export]
macro_rules! front_parse_keywords {
    ($(
        $(#[$metas:meta])*
        keywords $enum_name:ident
        { $(
            $string:literal -> $var:ident,
        )*}
    )*) => {
        $crate::reverse_parse_keywords! {
            $(
            $(#[$metas])*
            keywords $enum_name
            { $(
                $string -> $var,
            )*}
            )*
        }
        $(
        $crate::parse_unit_impl! {
            $(#[$metas])*
            $enum_name
            { $(
                $string -> $var,
            )*}
        }
        )*
    }
}
