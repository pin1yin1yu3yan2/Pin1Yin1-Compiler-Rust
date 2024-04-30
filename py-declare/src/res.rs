#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverloadIndex(pub usize);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct CacheTypeIndex(pub usize);

use std::{any::Any, rc::Rc};

use py_ir::ir::TypeDefine;

use crate::defs::FnSignWithName;

#[derive(Debug, Clone)]
pub enum Type {
    Overload(Overload),
    Directly(Directly),
    #[cfg(test)]
    Number(usize),
}

impl Type {
    pub fn overload(&self) -> &Overload {
        if let Type::Overload(ol) = self {
            ol
        } else {
            panic!()
        }
    }

    pub fn directly(&self) -> &TypeDefine {
        if let Type::Directly(ty) = self {
            &ty.0
        } else {
            panic!()
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Overload(ol) => std::fmt::Display::fmt(ol, f),
            Type::Directly(ty) => std::fmt::Display::fmt(&ty.0, f),
            #[cfg(test)]
            Type::Number(_) => unreachable!(),
        }
    }
}

impl Types for Type {
    fn get_type(&self) -> &TypeDefine {
        match self {
            Type::Overload(item) => item.get_type(),
            Type::Directly(item) => item.get_type(),
            #[cfg(test)]
            Type::Number(_) => unreachable!(),
        }
    }
}

impl<T: Into<TypeDefine>> From<T> for Type {
    fn from(value: T) -> Self {
        Self::Directly(Directly(Rc::new(value.into())))
    }
}

/// only difference between DeclareKinds are how they are printed
pub trait Types: Any + Sized {
    fn get_type(&self) -> &TypeDefine;
}

pub type Overload = Rc<FnSignWithName>;
impl Types for Overload {
    fn get_type(&self) -> &TypeDefine {
        &self.ty
    }
}

/// directly represent to a val's type
#[derive(Debug, Clone)]
pub struct Directly(pub Rc<TypeDefine>);
impl Types for Directly {
    fn get_type(&self) -> &TypeDefine {
        &self.0
    }
}
