use std::marker::PhantomData;

use terl::{Span, WithSpan};

use crate::{Defs, Type, Types};

pub trait BenchFilter<T: Types>: WithSpan {
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
    at: Span,
    _p: PhantomData<T>,
}

impl<T: Types, Fs, Fe> CustomFilter<T, Fs, Fe>
where
    Fs: Fn(&Type, &Defs) -> bool,
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
    Fs: Fn(&Type, &Defs) -> bool,
    Fe: Fn(&Defs) -> String,
{
    fn get_span(&self) -> Span {
        self.at
    }
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

pub mod filters {

    use std::any::TypeId;

    use py_ir::ir::TypeDefine;

    use crate::{Directly, Overload};

    use super::*;

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

    impl<'n> WithSpan for FnParamLen<'n> {
        fn get_span(&self) -> Span {
            self.at
        }
    }

    impl BenchFilter<Overload> for FnParamLen<'_> {
        fn satisfy(&self, ty: &Type, defs: &Defs) -> bool {
            let f = defs.try_get_fn(ty);
            f.params.len() == self.expect
        }

        fn expect(&self, defs: &Defs) -> String {
            let mut msg = format!("a funcion with {} parameters", self.expect);

            if let Some(name) = self.name {
                // TODO: import api so that note could be output here
                msg += "\nexist overloads whose length is expected:\n";
                let satisfies = defs
                    .get_unmangled(name)
                    .iter()
                    .filter(|ol| defs.try_get_fn(ol).params.len() == self.expect)
                    .map(|ol| ol.display(defs))
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
}
