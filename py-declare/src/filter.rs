use std::marker::PhantomData;

use terl::{Span, WithSpan};

use crate::{Defs, Type, Types};

pub trait BranchFilter<T: Types>: WithSpan {
    fn satisfy(&self, ty: &Type) -> bool;

    fn expect(&self, defs: &Defs) -> String;
}

pub struct CustomFilter<T: Types, Fs, Fe>
where
    Fs: Fn(&Type) -> bool,
    Fe: Fn(&Defs) -> String,
{
    satisfy: Fs,
    expect: Fe,
    at: Span,
    _p: PhantomData<T>,
}

impl<T: Types, Fs, Fe> CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type) -> bool,
    Fe: Fn(&Defs) -> String,
{
    pub fn new(satisfy: Fs, expect: Fe, at: Span) -> Self {
        Self {
            satisfy,
            expect,
            at,
            _p: PhantomData,
        }
    }
}

impl<T: Types, Fs, Fe> WithSpan for CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type) -> bool,
    Fe: Fn(&Defs) -> String,
{
    fn get_span(&self) -> Span {
        self.at
    }
}

impl<T: Types, Fs, Fe> BranchFilter<T> for CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type) -> bool,
    Fe: Fn(&Defs) -> String,
{
    fn satisfy(&self, ty: &Type) -> bool {
        (self.satisfy)(ty)
    }

    fn expect(&self, defs: &Defs) -> String {
        (self.expect)(defs)
    }
}

pub mod filters {

    use super::*;
    use crate::{Directly, Overload};
    use py_ir::types::TypeDefine;
    use std::any::TypeId;

    pub struct TypeEqual<'t> {
        expect: &'t TypeDefine,
        at: Span,
    }

    impl<'t> TypeEqual<'t> {
        pub fn new(expect: &'t TypeDefine, at: Span) -> Self {
            Self { expect, at }
        }
    }

    impl WithSpan for TypeEqual<'_> {
        fn get_span(&self) -> Span {
            self.at
        }
    }

    impl<T: Types> BranchFilter<T> for TypeEqual<'_> {
        fn satisfy(&self, ty: &Type) -> bool {
            ty.get_type() == self.expect
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

    pub struct FnParamLen<'n> {
        name: Option<&'n str>,
        expect: usize,
        at: Span,
    }

    impl<'n> FnParamLen<'n> {
        pub fn new(name: Option<&'n str>, expect: usize, at: Span) -> Self {
            Self { name, expect, at }
        }
    }

    impl WithSpan for FnParamLen<'_> {
        fn get_span(&self) -> Span {
            self.at
        }
    }

    impl BranchFilter<Overload> for FnParamLen<'_> {
        fn satisfy(&self, ty: &Type) -> bool {
            ty.overload().params.len() == self.expect
        }

        fn expect(&self, defs: &Defs) -> String {
            let mut msg = format!("a funcion with {} parameters", self.expect);

            if let Some(name) = self.name {
                msg += "\nexist overloads whose length is expected:\n";
                let satisfies = defs
                    .get_unmangled(name)
                    .unwrap()
                    .iter()
                    .filter(|ol| ol.params.len() == self.expect)
                    .map(|ol| ol.to_string())
                    .collect::<Vec<_>>();
                if satisfies.is_empty() {
                    msg += "emm, no-overload a matched :("
                } else {
                    msg += &satisfies.join("\n");
                }
            }
            msg
        }
    }

    pub struct NthParamTyEqual<'t> {
        pub at: Span,
        pub nth: usize,
        pub expect: &'t TypeDefine,
    }

    impl<'t> NthParamTyEqual<'t> {
        pub fn new(nth: usize, expect: &'t TypeDefine, at: Span) -> Self {
            Self { at, nth, expect }
        }
    }

    impl WithSpan for NthParamTyEqual<'_> {
        fn get_span(&self) -> Span {
            self.at
        }
    }

    impl BranchFilter<Overload> for NthParamTyEqual<'_> {
        fn satisfy(&self, ty: &Type) -> bool {
            ty.overload()
                .params
                .get(self.nth)
                .is_some_and(|param| &param.ty == self.expect)
        }

        fn expect(&self, _defs: &Defs) -> String {
            format!(
                "a function whose {}th parameter is {}",
                self.nth, self.expect
            )
        }
    }
}
