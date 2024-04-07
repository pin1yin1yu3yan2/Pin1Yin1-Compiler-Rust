use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use super::ModScope;

#[derive(Debug, Clone)]
pub struct ResourcesIdx {
    pub idx: usize,
}

#[derive(Default, Debug)]
pub struct DeclareMap {
    groups: Vec<DeclareGroup>,
    /// deps means that [`Bench`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Bench`] will be removed, too
    deps: HashMap<Bench, HashSet<Bench>>,
    // reversed dependencies
    rdeps: HashMap<Bench, HashSet<Bench>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_declare<I>(&mut self, iter: I) -> GroupIdx
    where
        I: IntoIterator<Item = (ResourcesIdx, Vec<Bench>)>,
    {
        let declare_idx = GroupIdx {
            idx: self.groups.len(),
        };

        let mut possiables = HashMap::default();

        for (idx, (kind, deps)) in iter.into_iter().enumerate() {
            possiables.insert(idx, kind);
            let this_node = Bench::new(declare_idx, idx);

            self.deps.insert(this_node, deps.iter().copied().collect());
            self.rdeps.insert(this_node, Default::default());

            for dep in deps {
                self.rdeps.get_mut(&dep).unwrap().insert(this_node);
            }
        }

        self.groups.push(DeclareGroup::new(possiables));

        declare_idx
    }

    pub fn delcare<I>(&mut self, scope: &ModScope, benches: I) -> terl::Result<()>
    where
        I: IntoIterator<Item = BenchBuilder>,
    {
        let gidx = GroupIdx {
            idx: self.groups.len(),
        };

        let mut res = HashMap::new();

        'bench: for (bench_idx, bench) in benches.into_iter().enumerate() {
            let mut deps = HashSet::new();
            let mut used_group = HashSet::new();

            let node = Bench::new(gidx, bench_idx);
            for action in bench.actinons {
                match (action)(self, scope) {
                    // use different bench in a group together
                    Ok(diff) if used_group.contains(&diff.belong_to) && !deps.contains(&diff) => {
                        todo!()
                    }
                    // mul bench matched together
                    Err(_mul_nodes) if !_mul_nodes.is_empty() => todo!(),
                    // non of bench matched, mean that bench failed
                    Err(_no_node) => continue 'bench,
                    // normal case
                    Ok(dep_node) => {
                        used_group.insert(dep_node.belong_to);
                        deps.insert(dep_node)
                    }
                };
            }

            for &dep in &deps {
                self.rdeps.get_mut(&dep).unwrap().insert(node);
            }
            self.deps.insert(node, deps);
            res.insert(bench_idx, bench.res);
        }

        self.check_group(gidx);

        Ok(())
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
    fn delete_bench(&mut self, bench: Bench) {
        if let Some(deps) = self.deps.remove(&bench) {
            for dep in deps {
                self.delete_bench(dep)
            }
        }
        if let Some(rdeps) = self.rdeps.remove(&bench) {
            for rdep in rdeps {
                self.delete_bench(rdep)
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

/// [`Vec<DeclareNode>`] as [`Err`] because only one of possible should be depended
///
/// if [`Vec::is_empty`], this measn that no possiable could find, this is not a kind of error,
/// but means the bench is impossiable
type BenchBuildAction = Box<dyn Fn(&DeclareMap, &ModScope) -> terl::Result<Bench, Vec<Bench>>>;

pub struct BenchBuilder {
    res: ResourcesIdx,

    actinons: Vec<BenchBuildAction>,
}

impl BenchBuilder {
    pub fn new(res: ResourcesIdx) -> Self {
        Self {
            res,

            actinons: vec![],
        }
    }

    pub fn new_filter<K1, F>(mut self, gid: GroupIdx, filter: impl Into<F>) -> Self
    where
        K1: DeclareKind,
        F: DeclareFilter<K1> + 'static,
    {
        let filter: F = filter.into();

        let action = move |map: &DeclareMap, scope: &ModScope| {
            let mut iter = map[gid]
                .res
                .iter()
                .filter(|(.., res)| filter.filter(res, scope));
            let Some((&idx, ..)) = iter.next() else {
                return Err(vec![]);
            };

            if let Some((idx, _res)) = iter.next() {
                let vec = std::iter::once(Bench::new(gid, *idx))
                    .chain(iter.map(|(idx, ..)| Bench::new(gid, *idx)))
                    .collect();
                return Err(vec);
            }

            Ok(Bench::new(gid, idx))
        };

        self.actinons.push(Box::new(action) as _);

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupIdx {
    idx: usize,
}

#[derive(Debug, Clone)]
pub struct DeclareGroup {
    res: HashMap<usize, ResourcesIdx>,
}

impl DeclareGroup {
    pub fn new(res: HashMap<usize, ResourcesIdx>) -> Self {
        Self { res }
    }
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

pub trait DeclareKind: Any {
    type Item: Debug + Clone;
    type Forward: Any + Debug + Clone;

    fn get_item(idx: ResourcesIdx, scope: &ModScope) -> &Self::Item;

    fn forawrd(idx: ResourcesIdx, scope: &ModScope) -> &Self::Forward;
}

pub trait DeclareFilter<K: DeclareKind> {
    fn filter(&self, idx: &ResourcesIdx, scope: &ModScope) -> bool;
}

pub struct DeclareFilterFn<K, F>(F, PhantomData<K>)
where
    K: DeclareKind,
    F: Fn(&ResourcesIdx, &ModScope) -> bool;

impl<K, F> DeclareFilter<K> for DeclareFilterFn<K, F>
where
    K: DeclareKind,
    F: Fn(&ResourcesIdx, &ModScope) -> bool,
{
    fn filter(&self, res: &ResourcesIdx, scope: &ModScope) -> bool {
        (self.0)(res, scope)
    }
}

impl<K, F> DeclareFilterFn<K, F>
where
    K: DeclareKind,
    F: Fn(&ResourcesIdx, &ModScope) -> bool,
{
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature() {
        let mut map = DeclareMap::new();

        macro_rules! t {
            ($idx:literal) => {
                ResourcesIdx { idx: $idx }
            };
        }

        let m1 = map.new_declare([(t!(1), vec![]), (t!(2), vec![]), (t!(3), vec![])]);
        let n1 = map.new_declare([(t!(2), vec![]), (t!(3), vec![]), (t!(4), vec![])]);

        let i = map.new_declare([
            (t!(3), vec![Bench::new(m1, 0), Bench::new(n1, 0)]),
            (t!(4), vec![Bench::new(m1, 1), Bench::new(n1, 1)]),
            (t!(5), vec![Bench::new(m1, 2), Bench::new(n1, 2)]),
        ]);

        let m2 = map.new_declare([(t!(1), vec![]), (t!(2), vec![]), (t!(3), vec![])]);
        let n2 = map.new_declare([(t!(2), vec![]), (t!(3), vec![]), (t!(4), vec![])]);

        let j = map.new_declare([
            (t!(3), vec![Bench::new(m2, 0), Bench::new(n2, 0)]),
            (t!(4), vec![Bench::new(m2, 1), Bench::new(n2, 1)]),
            (t!(5), vec![Bench::new(m2, 2), Bench::new(n2, 2)]),
        ]);

        let k = map.new_declare([(t!(5), vec![Bench::new(i, 0), Bench::new(j, 1)])]);
        map.make_sure(Bench::new(k, 0));

        for group in [m1, n1, i, m2, n2, j, k] {
            let bench_idx = *map.groups[group.idx].res.keys().next().unwrap();
            let bench = Bench::new(group, bench_idx);

            dbg!(&map.deps[&bench]);
            dbg!(&map.rdeps[&bench]);
        }
    }
}
