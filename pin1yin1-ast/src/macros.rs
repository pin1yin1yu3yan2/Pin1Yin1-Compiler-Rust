#[macro_export]
macro_rules! keywords {
    ($(

        keywords $enum_name:ident
        { $(
            $string:literal -> $var:ident
        ),*}
    )*) => {
        $(
        #[derive(Debug, Clone, Copy)]
        pub enum $enum_name {
            $(
                $var,
            )*
        }

        impl pin1yin1_parser::ParseUnit for $enum_name {
            type Target<'t> = $enum_name;
            fn select(selector: &mut pin1yin1_parser::Selector) {
                String::select(selector)
            }
            fn generate(selection: pin1yin1_parser::Selection) -> pin1yin1_parser::Result<'_, Self::Target<'_>> {
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
                let str: &str= &String::generate(selection)?;
                MAP.get(str).copied().ok_or(None)
            }
        }
        )*
    };
}
