use super::DeclareMap;

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
pub struct Declare<K: DeclareKind> {
    pub items: Vec<DeclareItem<K>>,
    pub rules: Vec<Box<dyn DeclareRule<K>>>,
}

pub trait DeclareAble<K: DeclareKind> {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap<K>) -> Option<&'a K::Type>;

    fn deps(&self) -> Vec<DeclareIdx>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclareIdx(pub(super) usize);

pub enum DeclareItem<K: DeclareKind> {
    Exist(K::Type),
    Solved(DeclareIdx),
    Unsolved(DeclareIdx),
}

impl<K: DeclareKind> DeclareAble<K> for DeclareItem<K> {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap<K>) -> Option<&'a <K as DeclareKind>::Type> {
        let (idx, result) = match self {
            DeclareItem::Exist(ty) => return Some(ty),
            DeclareItem::Unsolved(idx) | DeclareItem::Solved(idx) => {
                (*idx, unsafe { map.solve_one(*idx)? })
            }
        };
        *self = DeclareItem::Solved(idx);
        Some(result)
    }

    fn deps(&self) -> Vec<DeclareIdx> {
        match self {
            DeclareItem::Exist(_) => vec![],
            DeclareItem::Unsolved(idx) => vec![*idx],
            DeclareItem::Solved(_) => unreachable!(),
        }
    }
}

impl<K: DeclareKind> DeclareItem<K> {
    pub fn solved(&self) -> bool {
        matches!(self, DeclareItem::Exist(..) | DeclareItem::Solved(..))
    }

    pub fn solve_result<'a>(&'a self, map: &'a mut DeclareMap<K>) -> &'a K::Type {
        match self {
            DeclareItem::Exist(ty) => ty,
            DeclareItem::Solved(idx) => unsafe { map.solve_one(*idx).unwrap() },
            DeclareItem::Unsolved(..) => unreachable!(),
        }
    }
}

pub enum DeclareStatus<K: DeclareKind> {
    Solved(K::Type),
    Unsolved(Declare<K>),
}

impl<K: DeclareKind> DeclareAble<K> for DeclareStatus<K> {
    fn solve<'a>(&'a mut self, map: &'a mut DeclareMap<K>) -> Option<&'a K::Type> {
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

pub trait DeclareKind: Sized {
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
