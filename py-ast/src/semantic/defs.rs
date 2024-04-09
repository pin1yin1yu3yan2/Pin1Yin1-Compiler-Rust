use std::{borrow::Cow, collections::HashMap, rc::Rc};

use terl::Span;

use crate::ir;

use super::{
    declare::TypeIdx,
    mangle::{MangleItem, MangleUnit, Mangler},
};

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

    pub fn new_fn(&mut self, unmangled: String, mangled: String, sign: FnSign) -> TypeIdx {
        let idx = self.fn_signs.len();

        self.fn_signs.push(FnSignWithName {
            sign,
            name: mangled.clone(),
        });
        self.unmangled.entry(unmangled).or_default().push(idx);
        self.mangled.insert(mangled, idx);
        TypeIdx::from(idx)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<TypeIdx> {
        self.unmangled
            .get(name)
            .map(|idx| idx.iter().map(|&idx| TypeIdx::from(idx)).collect())
            .unwrap_or_default()
    }

    pub fn get_mangled(&self, name: &str) -> TypeIdx {
        self.mangled
            .get(name)
            .map(|&idx| TypeIdx::from(idx))
            .unwrap()
    }

    pub fn get_fn(&self, res: &TypeIdx) -> &FnSignWithName {
        let TypeIdx::ByIndex(idx) = res else {
            unreachable!(
                "only TypeIdx::ByIndex can be used to represent a function's overload's type"
            )
        };
        &self.fn_signs[*idx]
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

pub struct FnSign {
    /// return type of the function must be cleared (will change in future versions)
    pub ty: ir::TypeDefine,
    pub params: Vec<Param>,
    pub loc: Span,
}

pub struct Param {
    pub name: String,
    pub ty: ir::TypeDefine,
    pub loc: Span,
}

pub struct VarDef {
    pub ty: ir::TypeDefine,
    pub loc: Span,
}

impl VarDef {
    pub fn new(ty: ir::TypeDefine, loc: Span) -> Self {
        Self { ty, loc }
    }
}
