use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use terl::{Span, WithSpan};

use super::{mangle::Mangler, DefineScope};

#[derive(Debug, Clone, Copy)]
pub enum TypeRes {
    ByIndex(usize),
    /// * builtin types: need to be parsed to get [`py_ir::ir::TypeDefine`]
    ///   but only few types which are from [`py_ir::ir::AtomicExpr`]are supported,
    ///   std::prelude should complete more
    ///
    Buitin(&'static str),
}

/// used to decalre which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareMap {
    groups: Vec<DeclareGroup>,
    /// deps means that [`Bench`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Bench`] will be removed, too
    deps: HashMap<Bench, HashSet<Bench>>,
    rdeps: HashMap<Bench, HashSet<Bench>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_group<K, M>(
        &mut self,
        defs: &DefineScope<M>,
        gb: GroupBuidler<M>,
    ) -> terl::Result<GroupIdx>
    where
        M: Mangler,
        K: DeclareKind,
    {
        let gidx = GroupIdx {
            idx: self.groups.len(),
        };

        let mut res = HashMap::new();

        'bench: for (bench_idx, bench) in gb.benches.into_iter().enumerate() {
            let mut deps: HashSet<Bench> = HashSet::new();
            let mut used_group = HashSet::new();

            let node = Bench::new(gidx, bench_idx);
            for action in bench.actinons {
                match (action)(self, defs) {
                    // use different bench in a group together
                    Ok(conflict)
                        if used_group.contains(&conflict.belong_to)
                            && !deps.contains(&conflict) =>
                    {
                        let selected = *deps
                            .iter()
                            .find(|bench| bench.belong_to == conflict.belong_to)
                            .unwrap();

                        let err = BenchBuildError::ConflictSelected(selected, conflict)
                            .make::<K, M>(gb.span, self, defs);
                        res.insert(bench_idx, DeclareStatus::Faild(err));
                        continue 'bench;
                    }

                    Err(error) => {
                        let err = error.make::<K, M>(gb.span, self, defs);
                        res.insert(bench_idx, DeclareStatus::Faild(err));
                        continue 'bench;
                    }

                    // normal case
                    Ok(dep_node) => {
                        used_group.insert(dep_node.belong_to);
                        deps.insert(dep_node);
                    }
                };
            }

            for &dep in &deps {
                self.rdeps.get_mut(&dep).unwrap().insert(node);
            }
            self.deps.insert(node, deps);
            res.insert(bench_idx, DeclareStatus::Available(bench.res));
        }

        self.groups.push(DeclareGroup::new(gb.span, res));
        self.check_group(gidx);

        Ok(gidx)
    }

    fn get_resources(&self, bench: Bench) -> Result<TypeRes, &terl::Error> {
        match &self.groups[bench.belong_to.idx].res[&bench.bench_idx] {
            DeclareStatus::Available(res) => Ok(*res),
            DeclareStatus::Faild(e) => Err(e),
        }
    }

    fn display_bench<K, M>(&self, bench: Bench, defs: &DefineScope<M>) -> String
    where
        K: DeclareKind,
        M: Mangler,
    {
        K::display(self.get_resources(bench).unwrap(), defs)
    }

    fn check_group(&mut self, gidx: GroupIdx) {
        if self.groups[gidx.idx].res.len() == 1 {
            let bench_idx = *self.groups[gidx.idx].res.keys().next().unwrap();
            self.make_sure(Bench::new(gidx, bench_idx));
        }
    }

    /// make sure that the bench is selected
    fn make_sure(&mut self, bidx: Bench) {
        // take out other benches
        let declare_group = &mut self.groups[bidx.belong_to.idx].res;
        let mut removed_group = std::mem::take(declare_group);
        declare_group.insert(
            bidx.bench_idx,
            removed_group.remove(&bidx.bench_idx).unwrap(),
        );

        for removed in removed_group.keys() {
            self.delete_bench(Bench::new(bidx.belong_to, *removed))
        }

        // forward delcare result to lower level group
        for sub_bench in self.deps.get(&bidx).unwrap().clone() {
            self.make_sure(sub_bench);
        }
    }

    /// Zhu double eight: is your Nine Clan(Bench) wholesale?
    ///
    /// delete a node, and all node which must depend on it
    fn delete_bench(&mut self, bench: Bench) {
        // KILL all bench that depend on removed one
        if let Some(rdeps) = self.rdeps.remove(&bench) {
            for dep in self.deps.remove(&bench).unwrap() {
                // dont depend on a bench again:
                //  a. only removed bench depends on the bench in group, just remove rdep simply
                //  b. other bench in group in selected, make_sure will do deleting
                //
                // will get_mut reuten None? no, both deps and rdeps remove only be called here
                // and no ther opreator between then
                self.rdeps.get_mut(&dep).unwrap().remove(&bench);
            }
            for rdep in rdeps {
                self.delete_bench(rdep);
            }
        }
    }
}

impl std::ops::Index<GroupIdx> for DeclareMap {
    type Output = DeclareGroup;

    fn index(&self, index: GroupIdx) -> &Self::Output {
        &self.groups[index.idx]
    }
}

impl std::ops::IndexMut<GroupIdx> for DeclareMap {
    fn index_mut(&mut self, index: GroupIdx) -> &mut Self::Output {
        &mut self.groups[index.idx]
    }
}

pub struct GroupBuidler<M: Mangler> {
    span: Span,
    benches: Vec<BenchBuilder<M>>,
}

impl<M: Mangler> WithSpan for GroupBuidler<M> {
    fn get_span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub enum BenchBuildError {
    NonBenchSelected(GroupIdx),
    MultipleSelected(GroupIdx, Vec<usize>),
    ConflictSelected(Bench, Bench),
}

impl BenchBuildError {
    fn make<K, M>(self, happen: Span, map: &DeclareMap, defs: &DefineScope<M>) -> terl::Error
    where
        M: Mangler,
        K: DeclareKind,
    {
        match self {
            BenchBuildError::NonBenchSelected(gidx) => {
                map[gidx].make_error("non possiable bench of this", terl::ErrorKind::Semantic)
            }
            BenchBuildError::MultipleSelected(gidx, benches) => {
                let mut msg = String::new();

                for bench in benches {
                    msg += &format!(
                        "this can be declare to {}\n",
                        map.display_bench::<K, M>(Bench::new(gidx, bench), defs)
                    );
                }

                let err = happen
                    .make_error(
                        "multiple possible branches are selected to satisfy this bench\n",
                        terl::ErrorKind::Semantic,
                    )
                    .append(map[gidx].span, msg);

                err
            }
            BenchBuildError::ConflictSelected(selected, conflict) => {
                let msg1 = format!(
                    "the bench requires this to be delcared as {}",
                    map.display_bench::<K, M>(selected, defs)
                );
                let msg2 = format!(
                    "but the bench also requires this to be delcared as {}",
                    map.display_bench::<K, M>(conflict, defs)
                );

                happen
                    .make_error(format!("conflict requirements"), terl::ErrorKind::Semantic)
                    .append(map[selected.belong_to].span, msg1)
                    .append(map[conflict.belong_to].span, msg2)
            }
        }
    }
}

impl<M: Mangler> GroupBuidler<M> {
    pub fn new(span: Span, benches: Vec<BenchBuilder<M>>) -> Self {
        Self { span, benches }
    }
}

/// [`Vec<DeclareNode>`] as [`Err`] because only one of possible should be depended
///
/// if [`Vec::is_empty`], this measn that no possiable could find, this is not a kind of error,
/// but means the bench is impossiable
type BenchBuildAction<M: Mangler> =
    Box<dyn Fn(&DeclareMap, &DefineScope<M>) -> terl::Result<Bench, BenchBuildError>>;

pub struct BenchBuilder<M: Mangler> {
    res: TypeRes,
    actinons: Vec<BenchBuildAction<M>>,
}

impl<M: Mangler> BenchBuilder<M> {
    pub fn new(res: TypeRes) -> Self {
        Self {
            res,
            actinons: vec![],
        }
    }

    pub fn new_filter<K1, F>(mut self, gidx: GroupIdx, filter: impl Into<F>) -> Self
    where
        K1: DeclareKind,
        F: DeclareFilter<K1, M> + 'static,
    {
        let filter: F = filter.into();

        let action = move |map: &DeclareMap, scope: &DefineScope<M>| {
            let mut iter = map[gidx].res.iter().filter_map(|(idx, res)| match res {
                DeclareStatus::Available(avalable) if filter.filter(avalable, scope) => Some(idx),
                _ => None,
            });
            let Some(&idx) = iter.next() else {
                return Err(BenchBuildError::NonBenchSelected(gidx));
            };

            if let Some(idx) = iter.next() {
                let benches = std::iter::once(*idx).chain(iter.copied()).collect();
                return Err(BenchBuildError::MultipleSelected(gidx, benches));
            }

            Ok(Bench::new(gidx, idx))
        };

        self.actinons.push(Box::new(action) as _);

        self
    }
}

#[macro_export]
macro_rules! benches {
    {
        $(
            ($($filter:expr),*) => $res:expr
        ),*
    } => {
        {
            vec![$(
                $crate::semantic::declare::BenchBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupIdx {
    idx: usize,
}

#[derive(Debug, Clone)]
pub struct DeclareGroup {
    span: Span,
    res: HashMap<usize, DeclareStatus>,
}

impl WithSpan for DeclareGroup {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl DeclareGroup {
    pub fn new(span: Span, res: HashMap<usize, DeclareStatus>) -> Self {
        Self { span, res }
    }
}

#[derive(Debug, Clone)]
pub enum DeclareStatus {
    Available(TypeRes),
    Faild(terl::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bench {
    /// index of [ReflectDeclare] in [DeclareMap]
    belong_to: GroupIdx,
    /// index of possiable of [Declare]
    bench_idx: usize,
}

impl Bench {
    pub fn new(belong_to: GroupIdx, bench_idx: usize) -> Self {
        Self {
            belong_to,
            bench_idx,
        }
    }
}

pub trait DeclareFilter<K: DeclareKind, M: Mangler> {
    fn filter(&self, idx: &TypeRes, scope: &DefineScope<M>) -> bool;
}

pub struct DeclareFilterFn<K, M: Mangler, F>(F, PhantomData<(K, M)>)
where
    K: DeclareKind,
    F: Fn(&TypeRes, &DefineScope<M>) -> bool;

impl<K, M: Mangler, F> DeclareFilter<K, M> for DeclareFilterFn<K, M, F>
where
    K: DeclareKind,
    F: Fn(&TypeRes, &DefineScope<M>) -> bool,
{
    fn filter(&self, res: &TypeRes, scope: &DefineScope<M>) -> bool {
        (self.0)(res, scope)
    }
}

impl<K, M: Mangler, F> From<F> for DeclareFilterFn<K, M, F>
where
    K: DeclareKind,
    F: Fn(&TypeRes, &DefineScope<M>) -> bool,
{
    fn from(value: F) -> Self {
        Self(value, PhantomData)
    }
}

pub trait DeclareKind: Any + Sized {
    fn is<K1: DeclareKind>() -> bool {
        std::any::TypeId::of::<Self>() == std::any::TypeId::of::<K1>()
    }

    fn display<M: Mangler>(res: TypeRes, defs: &DefineScope<M>) -> String;
}

/// the return type of a fn's overload
struct Overload;
impl DeclareKind for Overload {
    fn display<M: Mangler>(res: TypeRes, defs: &DefineScope<M>) -> String {
        defs.get_fn(res).name.clone()
    }
}

/// a exist type
///
/// literal could be different type, like {number}'s type a any-width number
struct Literal;
impl DeclareKind for Literal {
    fn display<M: Mangler>(res: TypeRes, _defs: &DefineScope<M>) -> String {
        match res {
            TypeRes::ByIndex(_idx) => todo!("custom types"),
            TypeRes::Buitin(builtin) => builtin.to_owned(),
        }
    }
}

impl From<usize> for TypeRes {
    fn from(v: usize) -> Self {
        Self::ByIndex(v)
    }
}

impl From<&'static str> for TypeRes {
    fn from(v: &'static str) -> Self {
        Self::Buitin(v)
    }
}

#[cfg(test)]
mod tests {
    impl DeclareMap {
        fn test_declare<I>(&mut self, iter: I) -> GroupIdx
        where
            I: IntoIterator<Item = (TypeRes, Vec<Bench>)>,
        {
            let declare_idx = GroupIdx {
                idx: self.groups.len(),
            };

            let mut possiables = HashMap::default();

            for (idx, (res, deps)) in iter.into_iter().enumerate() {
                possiables.insert(idx, DeclareStatus::Available(res));
                let this_node = Bench::new(declare_idx, idx);

                self.deps.insert(this_node, deps.iter().copied().collect());
                self.rdeps.insert(this_node, Default::default());

                for dep in deps {
                    self.rdeps.get_mut(&dep).unwrap().insert(this_node);
                }
            }

            self.groups
                .push(DeclareGroup::new(Span::new(0, 0), possiables));

            declare_idx
        }
    }

    use super::*;

    #[test]
    fn feature() {
        let mut map = DeclareMap::new();

        macro_rules! ty {
            ($idx:literal) => {
                TypeRes::ByIndex { 0: $idx }
            };
        }

        // ty!(1-5) is used to emulate the type A-E
        //
        // m() -> A | B | C
        // b() -> B | C | D
        // p(A, B) -> C
        // p(B, C) -> D
        // p(C, D) -> E

        let m1 = map.test_declare([(ty!(1), vec![]), (ty!(2), vec![]), (ty!(3), vec![])]);
        let n1 = map.test_declare([(ty!(2), vec![]), (ty!(3), vec![]), (ty!(4), vec![])]);

        let i = map.test_declare([
            (ty!(3), vec![Bench::new(m1, 0), Bench::new(n1, 0)]),
            (ty!(4), vec![Bench::new(m1, 1), Bench::new(n1, 1)]),
            (ty!(5), vec![Bench::new(m1, 2), Bench::new(n1, 2)]),
        ]);

        let m2 = map.test_declare([(ty!(1), vec![]), (ty!(2), vec![]), (ty!(3), vec![])]);
        let n2 = map.test_declare([(ty!(2), vec![]), (ty!(3), vec![]), (ty!(4), vec![])]);

        let j = map.test_declare([
            (ty!(3), vec![Bench::new(m2, 0), Bench::new(n2, 0)]),
            (ty!(4), vec![Bench::new(m2, 1), Bench::new(n2, 1)]),
            (ty!(5), vec![Bench::new(m2, 2), Bench::new(n2, 2)]),
        ]);

        let k = map.test_declare([(ty!(5), vec![Bench::new(i, 0), Bench::new(j, 1)])]);
        map.make_sure(Bench::new(k, 0));

        for group in [m1, n1, i, m2, n2, j, k] {
            let bench_idx = *map.groups[group.idx].res.keys().next().unwrap();
            let bench = Bench::new(group, bench_idx);

            dbg!(&map.deps[&bench]);
        }
    }
}
