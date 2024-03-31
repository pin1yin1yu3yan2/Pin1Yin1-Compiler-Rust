use crate::ir;
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::parse;

use super::{Declare, DeclareAble, DeclareIdx, DeclareKind, DeclareStatus, Type};

#[derive(Default)]
pub struct FnDefs<'ast> {
    pub map: HashMap<String, FnDef<'ast>>,
}

pub struct FnDef<'ast> {
    /// functions have same names but different signatures
    ///
    /// unsupport now
    pub overloads: Vec<FnSign<'ast>>,
    pub raw_defines: Vec<&'ast parse::FnDefine>,
    _p: PhantomData<&'ast ()>,
}

impl<'ast> FnDef<'ast> {
    pub fn new(overloads: Vec<FnSign<'ast>>, raw_defines: Vec<&'ast parse::FnDefine>) -> Self {
        Self {
            overloads,

            raw_defines,
            _p: PhantomData,
        }
    }
}

pub struct FnSign<'ast> {
    pub mangle: String,
    pub ty: ir::TypeDefine,
    pub params: Vec<Param<'ast>>,
}

pub struct Param<'ast> {
    pub name: String,
    pub var_def: VarDef<'ast>,
    pub _p: PhantomData<&'ast ()>,
}

impl<'ast> std::ops::Deref for Param<'ast> {
    type Target = VarDef<'ast>;

    fn deref(&self) -> &Self::Target {
        &self.var_def
    }
}

#[derive(Default)]
pub struct VarDefs<'ast> {
    pub map: HashMap<String, VarDef<'ast>>,
}

pub struct VarDef<'ast> {
    pub ty: ir::TypeDefine,

    pub raw_define: &'ast parse::VarDefine,
    _p: PhantomData<&'ast ()>,
}

impl<'ast> VarDef<'ast> {
    pub fn new(ty: ir::TypeDefine, raw_define: &'ast parse::VarDefine) -> Self {
        Self {
            ty,

            raw_define,
            _p: PhantomData,
        }
    }
}

pub struct DeclareMap<K: DeclareKind> {
    items: Vec<DeclareStatus<K>>,
    deps: Vec<Vec<DeclareIdx>>,
}

impl<K: DeclareKind> DeclareMap<K> {
    pub fn new() -> Self {
        Self {
            items: vec![],
            deps: vec![vec![]],
        }
    }

    pub fn new_declare(&mut self) -> DeclareIdx {
        self.items.push(DeclareStatus::Unsolved(Declare::new()));
        self.deps.push(vec![]);

        // bias: deps[0] are always none, and deps[n] is the dependencies of deps[n-1]
        // so, so to do to let that deps[DeclareIdx.0] is the dependencies of deps[n-1]
        DeclareIdx(self.items.len())
    }

    fn is_cycle(&self) -> Result<(), Vec<DeclareIdx>> {
        // nodes are required by idx
        let mut in_degree = vec![vec![]; self.deps.len()];

        for (idx, deps) in self.deps.iter().enumerate() {
            for dep in deps {
                in_degree[dep.0].push(idx);
            }
        }

        // hashmap is cheap to remove
        let mut deps = self
            .deps
            .iter()
            .map(|deps| deps.len())
            .enumerate()
            .collect::<HashMap<_, _>>();

        loop {
            let empties = deps
                .iter()
                .filter(|(_, v)| **v == 0)
                .map(|(k, _)| *k)
                .collect::<Vec<_>>();

            for empty in &empties {
                deps.remove(empty);
            }

            if deps.is_empty() {
                return Ok(());
            }

            if empties.is_empty() {
                return Err(deps.keys().map(|k| DeclareIdx(*k)).collect());
            }

            for decrease in empties.iter().flat_map(|k| &in_degree[*k]) {
                *deps.get_mut(decrease).unwrap() -= 1;
            }
        }
    }

    fn build_dep_map(&mut self) {
        // load
        for i in 0..self.items.len() {
            self.deps[i] = self.items[i].deps();
        }
    }

    /// # Safety
    ///
    ///
    pub(super) unsafe fn solve_one(&mut self, idx: DeclareIdx) -> Option<&K::Type> {
        let s: &Self = self;
        #[allow(mutable_transmutes)]
        let s1: &mut Self = std::mem::transmute(s);
        #[allow(mutable_transmutes)]
        let s2: &mut Self = std::mem::transmute(s);
        s1.items[idx.0].solve(s2)
    }

    pub fn solve_all(&mut self) -> Result<(), Vec<DeclareIdx>> {
        self.build_dep_map();
        self.is_cycle()?;
        todo!()
    }
}

impl<K: DeclareKind> Default for DeclareMap<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: DeclareKind> Index<DeclareIdx> for DeclareMap<K> {
    type Output = DeclareStatus<K>;

    fn index(&self, index: DeclareIdx) -> &Self::Output {
        // bias:
        &self.items[index.0 - 1]
    }
}

impl<K: DeclareKind> IndexMut<DeclareIdx> for DeclareMap<K> {
    fn index_mut(&mut self, index: DeclareIdx) -> &mut Self::Output {
        &mut self.items[index.0 - 1]
    }
}
