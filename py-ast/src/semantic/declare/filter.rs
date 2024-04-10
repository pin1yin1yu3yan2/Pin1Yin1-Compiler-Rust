use super::{kind::DeclareKind, Type};
use crate::semantic::{mangle::Mangler, DefineScope};
use std::marker::PhantomData;

pub trait BenchFilter<K: DeclareKind, M: Mangler> {
    fn satisfy(&self, res: &Type, defs: &DefineScope<M>) -> bool;

    fn expect(&self, defs: &DefineScope<M>) -> String;
}

pub struct CustomFilter<K, M, Fs, Fe>
where
    K: DeclareKind,
    M: Mangler,
    Fs: Fn(&Type, &DefineScope<M>) -> bool,
    Fe: Fn(&DefineScope<M>) -> String,
{
    satisfy: Fs,
    expect: Fe,
    _p: PhantomData<(K, M)>,
}

impl<K, M, Fs, Fe> BenchFilter<K, M> for CustomFilter<K, M, Fs, Fe>
where
    K: DeclareKind,
    M: Mangler,
    Fs: Fn(&Type, &DefineScope<M>) -> bool,
    Fe: Fn(&DefineScope<M>) -> String,
{
    fn satisfy(&self, res: &Type, defs: &DefineScope<M>) -> bool {
        (self.satisfy)(res, defs)
    }

    fn expect(&self, defs: &DefineScope<M>) -> String {
        (self.expect)(defs)
    }
}

impl<K, M, Fs, Fe> CustomFilter<K, M, Fs, Fe>
where
    K: DeclareKind,
    M: Mangler,
    Fs: Fn(&Type, &DefineScope<M>) -> bool,
    Fe: Fn(&DefineScope<M>) -> String,
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

    use py_ir::ir::TypeDefine;

    use super::*;
    use crate::semantic::declare::kind::*;

    pub struct FnParmLenFilter<'n> {
        expect_len: usize,
        name: Option<&'n str>,
    }

    impl FnParmLenFilter<'_> {
        pub fn new(len: usize) -> Self {
            Self {
                expect_len: len,
                name: None,
            }
        }

        pub fn with_name(len: usize, name: &str) -> FnParmLenFilter {
            FnParmLenFilter {
                expect_len: len,
                name: Some(name),
            }
        }
    }

    impl<M: Mangler> BenchFilter<Overload, M> for FnParmLenFilter<'_> {
        fn satisfy(&self, res: &Type, defs: &DefineScope<M>) -> bool {
            defs.get_fn(res.as_fn_retty()).params.len() == self.expect_len
        }

        fn expect(&self, defs: &DefineScope<M>) -> String {
            let base = format!(
                "a function's overload whose parameters len is {}",
                self.expect_len
            );

            match self.name {
                Some(name) => {
                    let matched_len_count = defs
                        .fn_signs
                        .get_unmangled(name)
                        .iter()
                        .filter(|res| {
                            defs.get_fn(res.as_fn_retty()).params.len() == self.expect_len
                        })
                        .count();
                    format!(
                        "{base}, {} overload of function {} exist",
                        matched_len_count, self.expect_len
                    )
                }
                None => base,
            }
        }
    }

    pub struct TypeEqual<'t> {
        expect: &'t TypeDefine,
    }

    impl<'t> TypeEqual<'t> {
        pub fn new(expect: &'t TypeDefine) -> Self {
            Self { expect }
        }
    }

    impl<K: DeclareKind, M: Mangler> BenchFilter<K, M> for TypeEqual<'_> {
        fn satisfy(&self, res: &Type, defs: &DefineScope<M>) -> bool {
            match res {
                Type::FnRetty(ol) => &defs.get_fn(*ol).ty == self.expect,
                Type::Owned(ty) => ty == self.expect,
            }
        }

        fn expect(&self, _defs: &DefineScope<M>) -> String {
            match K::id() {
                id if id == Overload::id() => {
                    format!("a function whose return type is {}", self.expect)
                }
                id if id == Directly::id() => {
                    format!("a val whose type is {}", self.expect)
                }
                _ => unreachable!(),
            }
        }
    }
}
