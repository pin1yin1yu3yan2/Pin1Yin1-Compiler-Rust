use crate::*;
use py_ir::ir::TypeDefine;
use std::collections::HashMap;
use terl::Span;

#[derive(Default)]
pub struct Defs {
    pub(crate) fn_signs: FnSigns,
}

impl Defs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_main() -> Self {
        Self {
            fn_signs: FnSigns::new_with_main(),
        }
    }

    pub fn try_get_fn(&self, ty: &Type) -> &defs::FnSignWithName {
        match ty {
            Type::Overload(overload) => self.get_fn(overload.0),
            Type::Directly(_) => panic!(),
        }
    }

    pub fn get_fn(&self, res: usize) -> &defs::FnSignWithName {
        self.fn_signs.get_fn(res)
    }

    pub fn get_mangled(&self, name: &str) -> Type {
        self.fn_signs.get_mangled(name)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<Type> {
        self.fn_signs.get_unmangled(name)
    }

    pub fn new_fn(&mut self, unmangled: String, mangled: String, sign: defs::FnSign) -> Type {
        self.fn_signs.new_fn(unmangled, mangled, sign)
    }
}

#[derive(Default)]
pub struct FnSigns {
    fn_signs: Vec<defs::FnSignWithName>,
    unmangled: HashMap<String, Vec<usize>>,
    mangled: HashMap<String, usize>,
}

impl FnSigns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_main() -> Self {
        use py_ir::ir::PrimitiveType;
        let mut s = Self::default();
        let main_sign = defs::FnSign {
            ty: TypeDefine::Primitive(PrimitiveType::I32),
            params: vec![],
            // no location
            loc: Span::new(0, 0),
        };

        s.new_fn("main".to_owned(), "main".to_owned(), main_sign);

        s
    }

    pub fn new_fn(&mut self, unmangled: String, mangled: String, sign: defs::FnSign) -> Type {
        let idx = self.fn_signs.len();

        self.fn_signs.push(defs::FnSignWithName {
            sign,
            name: mangled.clone(),
        });
        self.unmangled.entry(unmangled).or_default().push(idx);
        self.mangled.insert(mangled, idx);
        Type::new_overload(idx)
    }

    pub fn get_unmangled(&self, name: &str) -> Vec<Type> {
        self.unmangled
            .get(name)
            .map(|idx| idx.iter().map(|&idx| Type::new_overload(idx)).collect())
            .unwrap_or_default()
    }

    pub fn get_mangled(&self, name: &str) -> Type {
        self.mangled
            .get(name)
            .map(|&idx| Type::new_overload(idx))
            .unwrap()
    }

    pub fn get_fn(&self, res: usize) -> &defs::FnSignWithName {
        &self.fn_signs[res]
    }

    // pub fn search_fns
}

#[derive(Debug, Clone)]
pub struct FnSignWithName {
    pub(crate) sign: FnSign,
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
    pub ty: TypeDefine,
    pub params: Vec<Param>,
    pub loc: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeDefine,
    pub loc: Span,
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub ty: GroupIdx,
    pub loc: Span,
    pub mutable: bool,
}

impl VarDef {
    pub fn new(ty: GroupIdx, loc: Span, mutable: bool) -> Self {
        Self { ty, loc, mutable }
    }
}
