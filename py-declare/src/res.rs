#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverloadIndex(pub usize);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct CacheTypeIndex(pub usize);

use std::any::Any;

use py_ir::ir::TypeDefine;

use crate::Defs;

#[derive(Debug, Clone)]
pub enum Type {
    Overload(Overload),
    Directly(Directly),
}

impl Type {
    pub fn new_overload(idx: usize) -> Self {
        Self::Overload(Overload(idx))
    }
}

impl Types for Type {
    fn get_type<'t>(&'t self, defs: &'t Defs) -> &TypeDefine {
        match self {
            Type::Overload(item) => item.get_type(defs),
            Type::Directly(item) => item.get_type(defs),
        }
    }

    fn display(&self, defs: &Defs) -> String {
        match self {
            Type::Overload(item) => item.display(defs),
            Type::Directly(item) => item.display(defs),
        }
    }
}

impl<T: Into<TypeDefine>> From<T> for Type {
    fn from(value: T) -> Self {
        Self::Directly(Directly(value.into()))
    }
}

/// only difference between DeclareKinds are how they are printed
pub trait Types: Any + Sized {
    fn get_type<'t>(&'t self, defs: &'t Defs) -> &TypeDefine;

    fn display(&self, defs: &Defs) -> String;
}

/// the return type of a fn's overload
#[derive(Debug, Clone)]
pub struct Overload(pub usize);
impl Types for Overload {
    fn get_type<'t>(&'t self, defs: &'t Defs) -> &TypeDefine {
        &defs.get_fn(self.0).ty
    }

    fn display(&self, defs: &Defs) -> String {
        format!("{:?}", defs.get_fn(self.0))
    }
}

/// directly represent to a val's type
///
/// alyhough its fro'm a function's overload( [`Type::FnRetty`]), it will only display
/// function's return type nor function's full sign
#[derive(Debug, Clone)]
pub struct Directly(pub TypeDefine);
impl Types for Directly {
    fn get_type<'t>(&'t self, _: &'t Defs) -> &TypeDefine {
        &self.0
    }

    fn display(&self, _: &Defs) -> String {
        self.0.to_string()
    }
}
