use crate::defs::FnSignWithName;
use py_ir::types::TypeDefine;
use std::{any::Any, rc::Rc};

#[derive(Debug, Clone)]
pub enum Type {
    Overload(Overload),
    Directly(Directly),
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
        use std::fmt::Display;
        match self {
            Type::Overload(ol) => Display::fmt(ol, f),
            Type::Directly(ty) => Display::fmt(&ty.0, f),
        }
    }
}

impl Types for Type {
    fn get_type(&self) -> &TypeDefine {
        match self {
            Type::Overload(item) => item.get_type(),
            Type::Directly(item) => item.get_type(),
        }
    }
}

impl<T: Into<TypeDefine>> From<T> for Type {
    fn from(value: T) -> Self {
        Self::Directly(Directly(Rc::new(value.into())))
    }
}

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
