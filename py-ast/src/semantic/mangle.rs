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
    fn mangle(_unit: MangleUnit) -> String {
        todo!()
    }

    fn demangle(_str: &str) -> MangleUnit<'static> {
        todo!()
    }
}
