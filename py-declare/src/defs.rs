use py_ir::ir::TypeDefine;

use crate::{Directly, Overload, Type, Types};

pub trait Defines<T: Types> {
    fn get_type<'t>(&'t self, item: &'t T) -> &TypeDefine;

    fn display(&self, item: &T) -> String;
}

impl<S> Defines<Type> for S
where
    S: Defines<Overload> + Defines<Directly>,
{
    fn get_type<'a>(&'a self, item: &'a Type) -> &TypeDefine {
        match item {
            Type::Overload(item) => self.get_type(item),
            Type::Directly(item) => self.get_type(item),
        }
    }

    fn display(&self, item: &Type) -> String {
        match item {
            Type::Overload(item) => self.display(item),
            Type::Directly(item) => self.display(item),
        }
    }
}
