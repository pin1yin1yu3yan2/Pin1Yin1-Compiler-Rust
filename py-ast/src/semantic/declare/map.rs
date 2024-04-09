use std::collections::{HashMap, HashSet};

use terl::WithSpan;

use crate::semantic::{mangle::Mangler, DefineScope};

use super::{
    kind::DeclareKind, Bench, BenchFilter, BenchStatus, DeclareGroup, GroupBuilder, GroupIdx,
    TypeIdx,
};

/// used to decalre which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareMap {
    pub(super) groups: Vec<DeclareGroup>,
    /// deps means that [`Bench`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Bench`] will be removed, too
    pub(super) deps: HashMap<Bench, HashSet<Bench>>,
    pub(super) rdeps: HashMap<Bench, HashSet<Bench>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_group<K, M>(
        &mut self,
        defs: &DefineScope<M>,
        gb: GroupBuilder<M>,
    ) -> terl::Result<GroupIdx>
    where
        M: Mangler,
        K: DeclareKind,
    {
        let gidx = GroupIdx::new(self.groups.len());

        let mut bench_res = HashMap::new();

        for (bench_idx, builder) in gb.builders.into_iter().enumerate() {
            let bench = Bench::new(gidx, bench_idx);
            let status = match builder.build::<K>(gb.span, self, defs) {
                Ok((res, deps)) => {
                    for &dep in &deps {
                        self.rdeps.get_mut(&dep).unwrap().insert(bench);
                    }
                    self.deps.insert(bench, deps);
                    BenchStatus::Available(res)
                }
                Err(err) => BenchStatus::Faild(err),
            };
            bench_res.insert(bench_idx, status);
        }

        self.groups.push(DeclareGroup::new(gb.span, bench_res));
        self.check_group(gidx);

        Ok(gidx)
    }

    pub fn apply_filter<K, M>(
        &mut self,
        gidx: GroupIdx,
        defs: &DefineScope<M>,
        filter: impl BenchFilter<K, M>,
    ) where
        K: DeclareKind,
        M: Mangler,
    {
        let group = &mut self[gidx];
        let benches: Vec<_> = group
            .res
            .iter()
            .filter(|(.., status)| matches!(status, BenchStatus::Available(..)))
            .map(|(bench_idx, ..)| Bench::new(gidx, *bench_idx))
            .collect();
        for bench in benches {
            let BenchStatus::Available(ref res) = group.res[&bench.bench_idx] else {
                unreachable!("this knid of case are filtered out in filter up")
            };
            if !filter.satisfy(&res, defs) {
                let msg = format!(
                    "expect this to be {}, but the bench is unsatisfy",
                    filter.expect(defs)
                );

                *group.res.get_mut(&bench.bench_idx).unwrap() =
                    BenchStatus::Faild(group.make_error(msg, terl::ErrorKind::Semantic));
            }
        }
    }

    fn get_resources(&self, bench: Bench) -> Result<&TypeIdx, &terl::Error> {
        match &self[bench.belong_to].res[&bench.bench_idx] {
            BenchStatus::Available(res) => Ok(res),
            BenchStatus::Faild(e) => Err(e),
        }
    }

    pub(super) fn display_bench<K, M>(&self, bench: Bench, defs: &DefineScope<M>) -> String
    where
        K: DeclareKind,
        M: Mangler,
    {
        K::display(&self.get_resources(bench).unwrap(), defs)
    }

    pub(super) fn check_group(&mut self, gidx: GroupIdx) {
        if self[gidx].res.len() == 1 {
            let bench_idx = *self[gidx].res.keys().next().unwrap();
            self.make_sure(Bench::new(gidx, bench_idx));
        }
    }

    /// make sure that the bench is selected
    pub(super) fn make_sure(&mut self, bidx: Bench) {
        // take out other benches
        let declare_group = &mut self[bidx.belong_to].res;
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
    pub(super) fn delete_bench(&mut self, bench: Bench) {
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
