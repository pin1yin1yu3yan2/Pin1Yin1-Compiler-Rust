use std::{borrow::Cow, collections::HashMap, marker::PhantomData, rc::Rc};

use py_declare::{mir, Defines, Directly, Overload, Type};
use terl::Span;

use crate::ir;

use super::mangle::*;

pub struct DefineScope<M: Mangler> {
    pub(crate) fn_signs: FnSigns,
    prefex: Vec<ManglePrefix>,
    _m: PhantomData<M>,
}

impl<M: Mangler> DefineScope<M> {
    pub fn new() -> Self {
        Self {
            fn_signs: FnSigns::default(),
            prefex: Vec::default(),
            _m: PhantomData,
        }
    }

    pub fn new_with_main() -> Self {
        // Fns::new_with_main must mangel "main" to "main"
        Self {
            fn_signs: FnSigns::new_with_main::<M>(),
            prefex: Vec::default(),
            _m: PhantomData,
        }
    }
    fn mangle_unit<'m>(&'m self, item: MangleItem<'m>) -> MangleUnit {
        MangleUnit {
            prefix: std::borrow::Cow::Borrowed(&self.prefex),
            item,
        }
    }

    pub fn mangle(&self, item: MangleItem) -> String {
        let unit = self.mangle_unit(item);
        M::mangle(unit)
    }

    pub fn mangle_ty(&self, ty: &mir::TypeDefine) -> MangleUnit {
        match ty {
            mir::TypeDefine::Primitive(pty) => self.mangle_unit(MangleItem::Type {
                ty: Cow::Owned(pty.to_string()),
            }),
            mir::TypeDefine::Complex(_) => todo!(),
        }
    }

    pub fn mangle_fn(&self, name: &str, sign: &FnSign) -> String {
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

    pub fn get_fn(&self, res: usize) -> &FnSignWithName {
        self.fn_signs.get_fn(res)
    }

    pub fn get_mangled(&self, name: &str) -> Type {
        self.fn_signs.get_mangled(name)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<Type> {
        self.fn_signs.get_unmangled(name)
    }
}

impl<M: Mangler> Defines<Overload> for DefineScope<M> {
    fn get_type(&self, item: &Overload) -> &ir::TypeDefine {
        &self.get_fn(item.0).ty
    }

    fn display(&self, item: &Overload) -> String {
        format!("{:?}", self.get_fn(item.0))
    }
}

impl<M: Mangler> Defines<Directly> for DefineScope<M> {
    fn get_type<'a>(&'a self, item: &'a Directly) -> &ir::TypeDefine {
        &item.0
    }

    fn display(&self, item: &Directly) -> String {
        item.0.to_string()
    }
}

#[derive(Default)]
pub struct FnSigns {
    fn_signs: Vec<FnSignWithName>,
    unmangled: HashMap<String, Vec<usize>>,
    mangled: HashMap<String, usize>,
}

impl FnSigns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_main<M: Mangler>() -> Self {
        let mut s = Self::default();
        let main_sign = FnSign {
            ty: ir::TypeDefine::Primitive(ir::PrimitiveType::I32),
            params: vec![],
            // no location
            loc: Span::new(0, 0),
        };

        let mangle = MangleUnit {
            prefix: Cow::Borrowed(&[]),
            item: MangleItem::Fn {
                name: Cow::Borrowed("main"),
                params: vec![],
            },
        };

        s.new_fn("main".to_owned(), M::mangle(mangle), main_sign);

        s
    }

    pub fn new_fn(&mut self, unmangled: String, mangled: String, sign: FnSign) -> Type {
        let idx = self.fn_signs.len();

        self.fn_signs.push(FnSignWithName {
            sign,
            name: mangled.clone(),
        });
        self.unmangled.entry(unmangled).or_default().push(idx);
        self.mangled.insert(mangled, idx);
        Type::overload(idx)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<Type> {
        self.unmangled
            .get(name)
            .map(|idx| idx.iter().map(|&idx| Type::overload(idx)).collect())
            .unwrap_or_default()
    }

    pub fn get_mangled(&self, name: &str) -> Type {
        self.mangled
            .get(name)
            .map(|&idx| Type::overload(idx))
            .unwrap()
    }

    pub fn get_fn(&self, res: usize) -> &FnSignWithName {
        &self.fn_signs[res]
    }

    // pub fn search_fns
}

pub struct FnDef {
    // Vec is enough
    pub overloads: Vec<Rc<FnSign>>,
}

impl FnDef {
    pub fn new(overloads: Vec<Rc<FnSign>>) -> Self {
        Self { overloads }
    }
}

#[derive(Debug, Clone)]
pub struct FnSignWithName {
    sign: FnSign,
    pub name: String,
}

impl std::ops::Deref for FnSignWithName {
    type Target = FnSign;

    fn deref(&self) -> &Self::Target {
        &self.sign
    }
}

impl std::ops::DerefMut for FnSignWithName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sign
    }
}

#[derive(Debug, Clone)]
pub struct FnSign {
    /// return type of the function must be cleared (will change in future versions)
    pub ty: ir::TypeDefine,
    pub params: Vec<Param>,
    pub loc: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: ir::TypeDefine,
    pub loc: Span,
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub ty: Type,
    pub loc: Span,
    pub mutable: bool,
}

impl VarDef {
    pub fn new(ty: Type, loc: Span, mutable: bool) -> Self {
        Self { ty, loc, mutable }
    }
}
