use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ManglePrefix {
    Mod(String),
    Type(String),
}

#[derive(Debug, Clone)]
pub enum MangleItem<'p> {
    Fn(&'p str, &'p [MangleUnit<'p>]),
    Type(),
    Val(),
}

#[derive(Debug, Clone)]
pub struct MangleUnit<'p> {
    prefix: &'p [ManglePrefix],
    item: MangleItem<'p>,
}

pub trait MangleAble<M: Mangler> {
    fn mangle_unit(&self) -> MangleUnit;
}

pub trait Mangler: Sized {
    fn mangle(unit: MangleUnit) -> String;
}

pub type DefaultMangler = ChineseMangler;

pub struct ChineseMangler;

impl Mangler for ChineseMangler {
    fn mangle(unit: MangleUnit) -> String {
        todo!()
    }
}
