use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

#[derive(Debug, Clone)]
pub enum ManglePrefix {
    Mod(String),
    Type(String),
}

#[derive(Debug, Clone)]
pub enum MangleItem<'m> {
    Fn {
        name: Cow<'m, str>,
        /// must be [`MangleItem::Type`]
        params: Vec<MangleUnit<'m>>,
    },
    Type {
        ty: Cow<'m, str>,
    },
    Val(),
}

#[derive(Debug, Clone)]
pub struct MangleUnit<'m> {
    pub prefix: Cow<'m, [ManglePrefix]>,
    pub item: MangleItem<'m>,
}

pub trait Mangle: Sized + 'static {
    fn mangle(unit: MangleUnit) -> String;

    fn demangle(str: &str) -> MangleUnit<'static>;
}

pub type DefaultMangler = ChineseMangler;

pub struct ChineseMangler;

impl Mangle for ChineseMangler {
    fn mangle(unit: MangleUnit) -> String {
        fn mangle_prefex(prefix: &[ManglePrefix]) -> String {
            prefix.iter().fold(String::new(), |buffer, pf| match pf {
                ManglePrefix::Mod(s) | ManglePrefix::Type(s) => buffer + &format!("{s}的"),
            })
        }

        let prefix = mangle_prefex(&unit.prefix);

        match unit.item {
            MangleItem::Fn { name, params } => {
                use std::fmt::Write;
                let mut output = format!("{prefix}{name} 参");
                for param in params.into_iter() {
                    write!(&mut output, " {}", Self::mangle(param)).ok();
                }
                format!("{output} 结")
            }
            MangleItem::Type { ty } => prefix + &ty,
            MangleItem::Val() => todo!(),
        }
    }

    fn demangle(_str: &str) -> MangleUnit<'static> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Mangler<M: Mangle> {
    prefix: Vec<ManglePrefix>,
    _p: PhantomData<M>,
}

impl<M: Mangle> Default for Mangler<M> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            _p: Default::default(),
        }
    }
}

impl<M: Mangle> Mangler<M> {
    pub fn new(prefix: Vec<ManglePrefix>) -> Mangler<M> {
        Mangler {
            prefix,
            _p: PhantomData,
        }
    }

    fn mangle_unit<'m>(&'m self, item: MangleItem<'m>) -> MangleUnit {
        MangleUnit {
            prefix: Cow::Borrowed(&self.prefix),
            item,
        }
    }

    pub fn mangle(&self, item: MangleItem) -> String {
        let unit = self.mangle_unit(item);
        M::mangle(unit)
    }

    pub fn mangle_ty(&self, ty: &py_ir::ir::TypeDefine) -> MangleUnit {
        match ty {
            py_ir::ir::TypeDefine::Primitive(pty) => self.mangle_unit(MangleItem::Type {
                ty: Cow::Owned(pty.to_string()),
            }),
            py_ir::ir::TypeDefine::Complex(_) => todo!(),
        }
    }

    pub fn mangle_fn(&self, name: &str, sign: &py_declare::defs::FnSign) -> String {
        let params = sign
            .params
            .iter()
            .map(|param| self.mangle_ty(&param.ty))
            .collect::<Vec<_>>();
        self.mangle(MangleItem::Fn {
            name: Cow::Borrowed(name),
            params,
        })
    }
}
