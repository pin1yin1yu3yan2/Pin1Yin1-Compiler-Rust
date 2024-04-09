use std::any::Any;

use crate::semantic::{mangle::Mangler, DefineScope};

use super::TypeIdx;

pub trait DeclareKind: Any + Sized {
    fn id() -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn display<M: Mangler>(res: &TypeIdx, defs: &DefineScope<M>) -> String;
}

/// the return type of a fn's overload
pub struct Overload;
impl DeclareKind for Overload {
    fn display<M: Mangler>(res: &TypeIdx, defs: &DefineScope<M>) -> String {
        defs.get_fn(res).name.clone()
    }
}

/// a exist type
///
/// literal could be different type, like {number}'s type a any-width number
pub struct Literal;
impl DeclareKind for Literal {
    fn display<M: Mangler>(res: &TypeIdx, _defs: &DefineScope<M>) -> String {
        match res {
            TypeIdx::ByIndex(_idx) => unreachable!(),
            TypeIdx::Direct(ty) => ty.to_string(),
        }
    }
}

pub struct Cached;
impl DeclareKind for Cached {
    fn display<M: Mangler>(_res: &TypeIdx, _defs: &DefineScope<M>) -> String {
        todo!()
    }
}
