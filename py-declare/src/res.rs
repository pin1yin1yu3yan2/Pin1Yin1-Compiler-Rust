#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverloadIndex(pub usize);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct CacheTypeIndex(pub usize);

use std::any::Any;

use py_ir::ir::TypeDefine;

use crate::Defines;

#[derive(Debug, Clone)]
pub enum Type {
    Overload(Overload),
    Directly(Directly),
}

impl Type {
    pub fn overload(idx: usize) -> Self {
        Self::Overload(Overload(idx))
    }
}

impl Types for Type {}

impl<T: Into<TypeDefine>> From<T> for Type {
    fn from(value: T) -> Self {
        Self::Directly(Directly(value.into()))
    }
}

/// only difference between DeclareKinds are how they are printed
pub trait Types: Any + Sized {
    fn get_type<'t>(&'t self, defs: &'t dyn Defines<Self>) -> &TypeDefine {
        defs.get_type(self)
    }

    fn display(&self, defs: &dyn Defines<Self>) -> String {
        defs.display(self)
    }
}

/// the return type of a fn's overload
#[derive(Debug, Clone)]
pub struct Overload(pub usize);
impl Types for Overload {}

/// directly represent to a val's type
///
/// alyhough its fro'm a function's overload( [`Type::FnRetty`]), it will only display
/// function's return type nor function's full sign
#[derive(Debug, Clone)]
pub struct Directly(pub TypeDefine);
impl Types for Directly {}

pub type Defs = dyn Defines<Type>;
