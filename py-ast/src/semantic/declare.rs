use std::{
    any::{Any, TypeId},
    mem::transmute,
};

use terl::{Span, WithSpan};

use super::ModScope;

/// declaration map:
///
///     * determine which overload is used:
///
///         * basic operators around primitive types(determine literals' types)
///
/// implementation:
///
///     * use reflection to do decalre together
///
///     * use directed map to solve dependencies of declare items
///
///     * use rules to do declare
pub struct DeclareMap {
    items: Vec<ReflectDeclarer>,
    deps: Vec<Vec<DeclareIdx>>,
}

pub trait DeclareKind: Sized + Any {
    type Type: Clone;
}

impl DeclareMap {
    pub fn new() -> Self {
        Self {
            items: vec![],
            deps: vec![vec![]],
        }
    }

    pub fn new_declare<K, E>(&mut self, span: Span, err_msg: E) -> DeclareIdx
    where
        K: DeclareKind,
        E: Fn(&Declarer<K>) -> String + 'static,
    {
        let idx = DeclareIdx {
            idx: self.items.len(),
        };
        self.items
            .push(Declarer::<K>::new(idx, span, err_msg).into());
        self.deps.push(vec![]);

        idx
    }

    fn build_dep_map(&mut self) {
        // load
        for i in 0..self.items.len() {
            // bias: deps[0] deps none(skiped)
            self.deps[i + 1] = self.items[i].deps(&self);
        }
    }

    /// topo sort is used
    fn is_cycle(&self) -> Result<(), Vec<DeclareIdx>> {
        // nodes are required by idx
        let mut in_degree = vec![vec![]; self.deps.len() + 1];

        for (idx, deps) in self.deps.iter().enumerate() {
            for dep in deps {
                in_degree[dep.idx].push(idx);
            }
        }

        // hashmap is cheap to remove
        use std::collections::HashMap;
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
                return Err(deps.keys().map(|k| DeclareIdx::new(*k)).collect());
            }

            for decrease in empties.iter().flat_map(|k| &in_degree[*k]) {
                *deps.get_mut(decrease).unwrap() -= 1;
            }
        }
    }

    /// # Safety
    ///
    ///  [`Self::build_dep_map`] and [`Self::is_cycle`] must be called before calling this function
    ///
    /// this method may be ub if there is a cycle dependency in [`Self::deps`]
    pub unsafe fn solve_one<K: DeclareKind>(&mut self, idx: DeclareIdx) -> Option<&K::Type> {
        let s: &Self = self;
        #[allow(mutable_transmutes)]
        let s1: &mut Self = transmute(s);
        #[allow(mutable_transmutes)]
        let s2: &mut Self = transmute(s);
        s1[idx].cast_mut::<K>().solve(s2)
    }

    pub fn solve_all(&mut self) -> Result<(), Vec<DeclareIdx>> {
        self.build_dep_map();
        self.is_cycle()?;

        // for idx in 1..self.deps {}

        todo!()
    }
}

impl std::ops::Index<DeclareIdx> for DeclareMap {
    type Output = ReflectDeclarer;

    fn index(&self, index: DeclareIdx) -> &Self::Output {
        &self.items[index.idx]
    }
}

impl std::ops::IndexMut<DeclareIdx> for DeclareMap {
    fn index_mut(&mut self, index: DeclareIdx) -> &mut Self::Output {
        &mut self.items[index.idx]
    }
}

impl Default for DeclareMap {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReflectDeclarer {
    kind_ty: TypeId,
    declarer: Box<Declarer<NotDeclareKind>>,
}

impl ReflectDeclarer {
    pub fn cast<K: DeclareKind>(&self) -> &Declarer<K> {
        assert!(self.kind_ty == TypeId::of::<K>());

        // see https://github.com/rust-lang/rust-clippy/issues/12602, this is a wrong suggestion
        #[allow(clippy::borrowed_box)]
        let item: &Box<Declarer<K>> = unsafe { transmute(&self.declarer) };
        item
    }

    pub fn cast_mut<K: DeclareKind>(&mut self) -> &mut Declarer<K> {
        assert!(self.kind_ty == TypeId::of::<K>());

        let item: &mut Box<Declarer<K>> = unsafe { transmute(&mut self.declarer) };
        item
    }

    fn deps(&self, _map: &DeclareMap) -> Vec<DeclareIdx> {
        self.declarer.deps()
    }
}

impl<K: DeclareKind> From<Declarer<K>> for ReflectDeclarer {
    fn from(value: Declarer<K>) -> Self {
        Self {
            kind_ty: TypeId::of::<K>(),
            declarer: unsafe { transmute(Box::new(value)) },
        }
    }
}

pub struct Declarer<K: DeclareKind> {
    pub idx: DeclareIdx,
    pub span: Span,
    pub items: Vec<DeclareItem<K>>,
    pub rules: Vec<Box<dyn DeclareRule<K>>>,
    pub error_msg: Box<dyn Fn(&Declarer<K>) -> String>,
}

impl<Kind: DeclareKind> Declarer<Kind> {
    pub fn new<E>(idx: DeclareIdx, span: Span, err_msg: E) -> Self
    where
        E: Fn(&Declarer<Kind>) -> String + 'static,
    {
        Self {
            idx,
            span,
            items: vec![],
            rules: vec![],
            error_msg: Box::new(err_msg),
        }
    }

    pub fn add_rule(&mut self, rule: impl DeclareRule<Kind> + 'static) {
        self.rules.push(Box::new(rule));
    }

    pub fn add_types(&mut self, types: &impl Types<Kind>) {
        self.items.extend(types.types());
    }

    pub fn deps(&self) -> Vec<DeclareIdx> {
        self.items
            .iter()
            .map(|item| item.get_declare_idx())
            .chain(self.rules.iter().map(|rule| rule.get_declare_idx()))
            .collect()
    }

    pub fn contain<R>(&self, map: &DeclareMap, rhs: &R) -> bool
    where
        Kind::Type: PartialEq<R>,
    {
        self.items
            .iter()
            .any(|item| item.declare_result(map).is_some_and(|item| item == rhs))
    }

    pub unsafe fn solve<'a>(
        &'a mut self,
        map: &'a mut DeclareMap,
    ) -> Option<&'a <Kind as DeclareKind>::Type> {
        for item in &mut self.items {
            item.solve(map)?;
        }

        let all_satisfy = |item: &DeclareItem<Kind>| -> bool {
            self.rules
                .iter()
                .all(|rule| rule.satisfy(item.declare_result(map).unwrap()))
        };

        let mut all_satisfied = vec![];
        while let Some(item) = self.items.pop() {
            if all_satisfy(&item) {
                all_satisfied.push(item);
            }
        }
        self.items = all_satisfied;
        self.declare_result(map)
    }

    pub fn declare<'a>(&'a mut self, map: &'a mut DeclareMap) -> terl::Result<&'a Kind::Type> {
        match unsafe { self.solve(map) } {
            // wtf??
            Some(..) => return Ok(self.declare_result(map).unwrap()),
            None => Err(self
                .get_span()
                .make_error((self.error_msg)(&self), terl::ErrorKind::Semantic)),
        }
    }

    pub fn declare_result<'a>(
        &'a self,
        map: &'a DeclareMap,
    ) -> Option<&'a <Kind as DeclareKind>::Type> {
        if self.items.len() == 1 {
            // unwrap: always(must) be Some
            Some(self.items[0].declare_result(map).unwrap())
        } else {
            None
        }
    }
}

impl<K: DeclareKind> terl::WithSpan for Declarer<K> {
    fn get_span(&self) -> Span {
        self.span
    }
}

pub trait Declare<Kind: DeclareKind> {
    fn get_declare_idx(&self) -> DeclareIdx;

    fn get_declarer<'a>(&self, map: &'a DeclareMap) -> &'a Declarer<Kind> {
        map[self.get_declare_idx()].cast()
    }

    fn get_declarer_mut<'a>(&self, map: &'a mut DeclareMap) -> &'a mut Declarer<Kind> {
        map[self.get_declare_idx()].cast_mut()
    }

    fn deps(&self, map: &DeclareMap) -> Vec<DeclareIdx> {
        map[self.get_declare_idx()].cast::<Kind>().deps()
    }

    /// # Safety
    ///
    /// see [`DeclareMap::solve_one`]
    unsafe fn solve<'a>(&'a mut self, map: &'a mut DeclareMap) -> Option<&'a Kind::Type> {
        map.solve_one::<Kind>(self.get_declare_idx())
    }

    fn declare<'a>(&'a mut self, map: &'a mut DeclareMap) -> terl::Result<&'a Kind::Type> {
        match unsafe { self.solve(map) } {
            // wtf??
            Some(..) => return Ok(self.declare_result(map).unwrap()),
            None => Err(self.get_declarer(map)).map_err(|dec| {
                dec.get_span()
                    .make_error((dec.error_msg)(dec), terl::ErrorKind::Semantic)
            }),
        }
    }

    fn declare_result<'a>(&'a self, map: &'a DeclareMap) -> Option<&'a Kind::Type> {
        self.get_declarer(map).declare_result(map)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeclareIdx {
    pub(super) idx: usize,
}

impl DeclareIdx {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl Default for DeclareIdx {
    fn default() -> Self {
        // '0' is always ok to be resolved
        // safe to appear in deps map
        Self { idx: 0 }
    }
}

pub enum DeclareItem<K: DeclareKind> {
    Exist(K::Type),
    Solved(DeclareIdx),
    Unsolved(DeclareIdx),
}

impl<K: DeclareKind> Declare<K> for DeclareItem<K> {
    fn get_declare_idx(&self) -> DeclareIdx {
        match self {
            DeclareItem::Exist(_) => DeclareIdx::default(),
            DeclareItem::Solved(idx) | DeclareItem::Unsolved(idx) => *idx,
        }
    }

    fn deps(&self, _map: &DeclareMap) -> Vec<DeclareIdx> {
        match self {
            DeclareItem::Exist(_) => vec![],
            DeclareItem::Solved(idx) => vec![*idx],
            DeclareItem::Unsolved(_) => vec![],
        }
    }

    unsafe fn solve<'a>(
        &'a mut self,
        map: &'a mut DeclareMap,
    ) -> Option<&'a <K as DeclareKind>::Type> {
        match self {
            DeclareItem::Exist(item) => Some(item),
            DeclareItem::Solved(idx) => map[*idx].cast::<K>().declare_result(map),
            DeclareItem::Unsolved(idx) => unsafe {
                map.solve_one::<K>(*idx)
                    .map(|ok| (*self = Self::Solved(self.get_declare_idx()), ok).1)
            },
        }
    }

    fn declare_result<'a>(&'a self, map: &'a DeclareMap) -> Option<&'a <K as DeclareKind>::Type> {
        match self {
            DeclareItem::Exist(exist) => Some(exist),
            DeclareItem::Solved(idx) => Some(map[*idx].cast::<K>().declare_result(map).unwrap()),
            DeclareItem::Unsolved(_) => None,
        }
    }
}

pub trait Types<K: DeclareKind> {
    fn types(&self) -> Vec<DeclareItem<K>>;
}

pub trait DeclareRule<K: DeclareKind>: Declare<K> {
    fn update(&mut self, scope: &mut ModScope);

    fn satisfy(&self, types: &K::Type) -> bool;
}

#[derive(Debug, Clone)]
pub enum NotDeclareKind {}

impl DeclareKind for NotDeclareKind {
    type Type = NotDeclareKind;
}

/*


    fn x(x: i32); // #1
    fn x(x: f32); // #2

    let a = (0..1).sum();

    x(a);
*/

/* rules:
    Len == 1:
        #1: ok
        #2: ok
    P1: allow X
        #1: ok
        #2: not_ok
*/
