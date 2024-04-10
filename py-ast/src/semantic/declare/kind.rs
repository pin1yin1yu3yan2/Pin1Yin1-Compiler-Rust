use std::any::Any;

use crate::semantic::{mangle::Mangler, DefineScope};

use super::Type;

/// only difference between DeclareKinds are how they are printed
pub trait DeclareKind: Any + Sized {
    fn id() -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn display<M: Mangler>(res: &Type, defs: &DefineScope<M>) -> String;
}

/// the return type of a fn's overload
pub struct Overload;
impl DeclareKind for Overload {
    fn display<M: Mangler>(res: &Type, defs: &DefineScope<M>) -> String {
        // TOOD: display wit full-of-information sign
        defs.get_fn(res.as_fn_retty()).name.clone()
    }
}

/// directly represent to a val's type
///
/// alyhough its fro'm a function's overload( [`Type::FnRetty`]), it will only display
/// function's return type nor function's full sign
pub struct Directly;
impl DeclareKind for Directly {
    fn display<M: Mangler>(res: &Type, defs: &DefineScope<M>) -> String {
        match res {
            // means that the val's type is from a fucntion's overload
            // alse TODO: fn's return type
            Type::FnRetty(..) => defs.get_fn(res.as_fn_retty()).name.clone(),
            Type::Owned(ty) => ty.to_string(),
        }
    }
}
