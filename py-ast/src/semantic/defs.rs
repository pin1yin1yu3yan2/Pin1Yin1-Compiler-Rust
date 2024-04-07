use std::{borrow::Cow, collections::HashMap, rc::Rc};

use terl::Span;

use crate::ir;

use super::{
    declare::TypeRes,
    mangle::{MangleItem, MangleUnit, Mangler},
};

#[derive(Default)]
pub struct FnSigns {
    pub fn_signs: Vec<FnSignWithName>,
    pub unmangled: HashMap<String, Vec<usize>>,
    pub mangled: HashMap<String, usize>,
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

    pub fn new_fn(&mut self, unmangled: String, mangled: String, sign: FnSign) -> TypeRes {
        let idx = self.fn_signs.len();

        self.fn_signs.push(FnSignWithName {
            sign,
            name: mangled.clone(),
        });
        self.unmangled.entry(unmangled).or_default().push(idx);
        self.mangled.insert(mangled, idx);
        TypeRes::from(idx)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<TypeRes> {
        self.unmangled
            .get(name)
            .map(|idx| idx.iter().map(|&idx| TypeRes::from(idx)).collect())
            .unwrap_or_default()
    }

    pub fn get_mangled(&self, name: &str) -> TypeRes {
        self.mangled
            .get(name)
            .map(|&idx| TypeRes::from(idx))
            .unwrap()
    }

    pub fn get_fn(&self, res: TypeRes) -> &FnSignWithName {
        match res {
            TypeRes::ByIndex(idx) => &self.fn_signs[idx],
            TypeRes::Buitin(_) => todo!("built in functions"),
        }
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
