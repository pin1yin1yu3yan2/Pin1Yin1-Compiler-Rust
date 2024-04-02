use std::{
    any::{Any, TypeId},
    mem::transmute,
};

/// declaration map:
///
///    * determine which overload is used:
///
///        * basic operators around primitive types(determine literals' types)
///
///    implementation:
///
///    * use Rc<RefCell> to store declarations
///
///
///    * store in mir and because unique type in py-ir
///
///    * use rules to do declare
pub struct DeclareMap {
    items: Vec<ReflectDeclareStatus>,
    deps: Vec<Vec<DeclareIdx>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self {
            items: vec![],
            deps: vec![vec![]],
        }
    }

    pub fn new_declare<K: DeclareKind>(&mut self) -> DeclareIdx {
        self.items
            .push(DeclareStatus::Unsolved(Declare::<K>::new()).into());
        self.deps.push(vec![]);

        // bias: deps[0] are always none, and deps[n] is the dependencies of deps[n-1]
        // so, so to do to let that deps[DeclareIdx.0] is the dependencies of deps[n-1]
        DeclareIdx(self.items.len())
    }

    fn build_dep_map(&mut self) {
        // load
        for i in 0..self.items.len() {
            self.deps[i] = self.items[i].deps();
        }
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
                return Err(deps.keys().map(|k| DeclareIdx(*k)).collect());
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
    pub(super) unsafe fn solve_one<K: DeclareKind>(&mut self, idx: DeclareIdx) -> Option<&K::Type> {
        let s: &Self = self;
        #[allow(mutable_transmutes)]
        let s1: &mut Self = transmute(s);
        #[allow(mutable_transmutes)]
        let s2: &mut Self = transmute(s);
        s1.items[idx.0].cast_mut::<K>().solve(s2)
    }

    pub fn solve_all(&mut self) -> Result<(), Vec<DeclareIdx>> {
        self.build_dep_map();
        self.is_cycle()?;

        // for idx in 1..self.deps {}

        todo!()
    }
}

impl Default for DeclareMap {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Declare<K: DeclareKind> {
    pub items: Vec<DeclareItem<K>>,
    pub rules: Vec<Box<dyn DeclareRule<K>>>,
}

pub trait DeclareDeps {
    fn deps(&self) -> Vec<DeclareIdx>;
}

pub trait DeclareAble<K: DeclareKind>: DeclareDeps {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap) -> Option<&'a K::Type>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclareIdx(pub(super) usize);

pub struct ReflectDeclareStatus {
    ty: TypeId,
    /// use box because we cant know [`DeclareStatus`]'s size
    status: Box<DeclareStatus<NotDeclareKind>>,
}

impl ReflectDeclareStatus {
    pub fn cast<K: DeclareKind>(&self) -> &DeclareStatus<K> {
        assert!(self.ty != TypeId::of::<K>());

        // see https://github.com/rust-lang/rust-clippy/issues/12602, this is a wrong suggestion
        #[allow(clippy::borrowed_box)]
        let item: &Box<DeclareStatus<K>> = unsafe { transmute(&self.status) };

        item
    }

    pub fn cast_mut<K: DeclareKind>(&mut self) -> &mut DeclareStatus<K> {
        assert!(self.ty != TypeId::of::<K>());
        let item: &mut Box<DeclareStatus<K>> = unsafe { transmute(&mut self.status) };
        item
    }
}

impl DeclareDeps for ReflectDeclareStatus {
    fn deps(&self) -> Vec<DeclareIdx> {
        // UB?
        self.status.deps()
    }
}

impl<K: DeclareKind> From<DeclareStatus<K>> for ReflectDeclareStatus {
    fn from(value: DeclareStatus<K>) -> Self {
        Self {
            ty: TypeId::of::<K>(),
            status: unsafe { transmute(Box::new(value)) },
        }
    }
}

pub enum DeclareStatus<K: DeclareKind> {
    Solved(K::Type),
    Unsolved(Declare<K>),
}

impl<K: DeclareKind> DeclareDeps for DeclareStatus<K> {
    fn deps(&self) -> Vec<DeclareIdx> {
        match self {
            DeclareStatus::Unsolved(unsolved) => unsolved
                .rules
                .iter()
                .flat_map(|rule| rule.deps())
                .chain(unsolved.items.iter().flat_map(|item| item.deps()))
                .collect::<Vec<_>>(),
            DeclareStatus::Solved(_) => unreachable!(),
        }
    }
}

impl<K: DeclareKind> DeclareAble<K> for DeclareStatus<K> {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap) -> Option<&'a K::Type> {
        let declare = match self {
            DeclareStatus::Solved(result) => return Some(result),
            DeclareStatus::Unsolved(declare) => declare,
        };

        for ty in &mut declare.items {
            ty.solve(map);
        }

        let all_satisfy = |item: &&DeclareItem<K>| -> bool {
            declare
                .rules
                .iter()
                .all(|rule| rule.satisfy(item.solve_result(map)))
        };

        let result = declare.items.iter().find(all_satisfy)?;

        *self = Self::Solved(result.solve_result(map).clone());

        // this is for sorter code compielr should optimze this!!!
        self.solve(map)
    }
}

impl<K: DeclareKind> Declare<K> {
    pub fn new() -> Self {
        Self {
            items: vec![],
            rules: vec![],
        }
    }

    pub fn add_rule(&mut self, rule: impl DeclareRule<K> + 'static) {
        self.rules.push(Box::new(rule));
    }

    pub fn add_types(&mut self, types: &impl Types<K>) {
        self.items.extend(types.types());
    }
}

impl<K, T> From<&T> for Declare<K>
where
    K: DeclareKind,
    T: Types<K>,
{
    fn from(value: &T) -> Self {
        Declare {
            items: value.types(),
            rules: vec![],
        }
    }
}

impl<K: DeclareKind> Default for Declare<K> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum DeclareItem<K: DeclareKind> {
    Exist(K::Type),
    Solved(DeclareIdx),
    Unsolved(DeclareIdx),
}

impl<K: DeclareKind> DeclareDeps for DeclareItem<K> {
    fn deps(&self) -> Vec<DeclareIdx> {
        match self {
            DeclareItem::Exist(_) => vec![],
            DeclareItem::Unsolved(idx) => vec![*idx],
            DeclareItem::Solved(_) => unreachable!(),
        }
    }
}

impl<K: DeclareKind> DeclareAble<K> for DeclareItem<K> {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap) -> Option<&'a <K as DeclareKind>::Type> {
        let (idx, result) = match self {
            DeclareItem::Exist(ty) => return Some(ty),
            DeclareItem::Unsolved(idx) | DeclareItem::Solved(idx) => {
                (*idx, unsafe { map.solve_one::<K>(*idx)? })
            }
        };
        *self = DeclareItem::Solved(idx);
        Some(result)
    }
}

impl<K: DeclareKind> DeclareItem<K> {
    pub fn solved(&self) -> bool {
        matches!(self, DeclareItem::Exist(..) | DeclareItem::Solved(..))
    }

    pub fn solve_result<'a>(&'a self, map: &'a mut DeclareMap) -> &'a K::Type {
        match self {
            DeclareItem::Exist(ty) => ty,
            DeclareItem::Solved(idx) => unsafe { map.solve_one::<K>(*idx).unwrap() },
            DeclareItem::Unsolved(..) => unreachable!(),
        }
    }
}

pub trait DeclareKind: Sized + Any {
    type Type: Clone;
}

#[derive(Debug, Clone)]
pub struct Type;

impl DeclareKind for Type {
    type Type = Self;
}

#[derive(Debug, Clone)]
pub struct FnOverload;

impl DeclareKind for FnOverload {
    type Type = Type;
}

pub trait Types<K: DeclareKind> {
    fn types(&self) -> Vec<DeclareItem<K>>;
}

pub trait DeclareRule<K: DeclareKind>: DeclareAble<K> {
    fn satisfy(&self, types: &K::Type) -> bool;
}

#[derive(Debug, Clone)]
pub struct NotDeclareKind;

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
