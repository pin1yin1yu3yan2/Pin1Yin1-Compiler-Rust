use std::{borrow::Cow, fmt::Debug};

#[derive(Debug, Clone)]
pub enum ManglePrefix {
    Mod(String),
    Type(String),
}

#[derive(Debug, Clone)]
pub enum MangleItem<'m> {
    Fn {
        name: Cow<'m, str>,
        /// must be [`MangleItem::Type`]
        params: Vec<MangleUnit<'m>>,
    },
    Type {
        ty: Cow<'m, str>,
    },
    Val(),
}

#[derive(Debug, Clone)]
pub struct MangleUnit<'m> {
    pub prefix: Cow<'m, [ManglePrefix]>,
    pub item: MangleItem<'m>,
}

pub trait Mangler: Sized + 'static {
    fn mangle(unit: MangleUnit) -> String;

    fn demangle(str: &str) -> MangleUnit<'static>;
}

pub type DefaultMangler = ChineseMangler;

pub struct ChineseMangler;

impl Mangler for ChineseMangler {
    fn mangle(unit: MangleUnit) -> String {
        fn mangle_prefex(prefix: &[ManglePrefix]) -> String {
            prefix.iter().fold(String::new(), |buffer, pf| match pf {
                ManglePrefix::Mod(s) | ManglePrefix::Type(s) => buffer + &format!("{s}的"),
            })
        }

        let prefix = mangle_prefex(&unit.prefix);

        match unit.item {
            MangleItem::Fn { name, params } => {
                use std::fmt::Write;
                let mut output = format!("{prefix}{name} 参");
                for param in params.into_iter() {
                    write!(&mut output, " {}", Self::mangle(param)).ok();
                }
                format!("{output} 结")
            }
            MangleItem::Type { ty } => prefix + &ty,
            MangleItem::Val() => todo!(),
        }
    }

    fn demangle(_str: &str) -> MangleUnit<'static> {
        todo!()
    }
}
