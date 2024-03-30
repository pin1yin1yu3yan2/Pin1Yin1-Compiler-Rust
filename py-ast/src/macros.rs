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
        #[derive(Debug, Clone)]
        $(#[$metas])*
        pub enum $enum_name {
            $(
                $(#[$v_metas])*
                $variant($variant),
            )*
        }

        $(
        impl From<$variant> for $enum_name {
             fn from(v: $variant) -> $enum_name {
                <$enum_name>::$variant(v)
            }
        }
        )*


        impl terl::ParseUnit for $enum_name {
            type Target = $enum_name;

            fn parse(p: &mut terl::Parser) -> terl::ParseResult<Self>
            {
                terl::Try::new(p)
                $(
                .or_try::<Self, _>(|p| {
                    p.once_no_try($variant::parse)
                        .map(|pu| pu.map(<$enum_name>::$variant))
                })
                )*
                .finish()
            }
        }
    };
}
