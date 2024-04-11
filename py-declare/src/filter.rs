use std::marker::PhantomData;

use crate::{Defs, Type, Types};

pub trait BenchFilter<T: Types> {
    fn satisfy(&self, ty: &Type, defs: &Defs) -> bool;

    fn expect(&self, defs: &Defs) -> String;
}

pub struct CustomFilter<T: Types, Fs, Fe>
where
    Fs: Fn(&Type, &Defs) -> bool,
    Fe: Fn(&Defs) -> String,
{
    satisfy: Fs,
    expect: Fe,
    _p: PhantomData<T>,
}

impl<T: Types, Fs, Fe> BenchFilter<T> for CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type, &Defs) -> bool,
    Fe: Fn(&Defs) -> String,
{
    fn satisfy(&self, ty: &Type, defs: &Defs) -> bool {
        (self.satisfy)(ty, defs)
    }

    fn expect(&self, defs: &Defs) -> String {
        (self.expect)(defs)
    }
}

impl<T: Types, Fs, Fe> CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type, &Defs) -> bool,
    Fe: Fn(&Defs) -> String,
{
    pub fn new(satisfy: Fs, expect: Fe) -> Self {
        Self {
            satisfy,
            expect,
            _p: PhantomData,
        }
    }
}

pub mod filters {

    use std::any::TypeId;

    use py_ir::ir::TypeDefine;

    use crate::{Directly, Overload};

    use super::*;

    pub struct TypeEqual<'t> {
        expect: &'t TypeDefine,
    }

    impl<'t> TypeEqual<'t> {
        pub fn new(expect: &'t TypeDefine) -> Self {
            Self { expect }
        }
    }

    impl<T: Types> BenchFilter<T> for TypeEqual<'_> {
        fn satisfy(&self, ty: &Type, defs: &Defs) -> bool {
            ty.get_type(defs) == self.expect
        }

        fn expect(&self, _: &Defs) -> String {
            if TypeId::of::<T>() == TypeId::of::<Overload>() {
                format!("a function whose return type is {}", self.expect)
            } else if TypeId::of::<T>() == TypeId::of::<Directly>() {
                format!("a val whose type is {}", self.expect)
            } else {
                unreachable!()
            }
        }
    }
}
