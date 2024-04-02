use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum ManglePrefix<'i> {
    Mod(&'i str),
    Type(&'i str),
}

#[derive(Debug, Clone)]
pub enum MangleItem<'i> {
    Fn(&'i str, &'i [MangleItem<'i>]),
    Type(),
    Val(),
}

#[derive(Debug, Clone)]
pub struct MangleUnit<'m> {
    prefix: Vec<ManglePrefix<'m>>,
    item: MangleItem<'m>,
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
