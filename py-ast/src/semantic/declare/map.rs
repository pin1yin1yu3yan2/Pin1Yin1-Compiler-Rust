use super::super::mir::TypeDefine;
use super::{
    kind::DeclareKind, Bench, BenchFilter, BenchStatus, DeclareGroup, GroupBuilder, GroupIdx, Type,
};
use crate::semantic::{mangle::Mangler, DefineScope};
use std::collections::{HashMap, HashSet};
use terl::WithSpan;

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

        Ok(gidx)
    }

    pub fn decalre_as<K, M>(
        &mut self,
        gidx: GroupIdx,
        defs: &DefineScope<M>,
        as_: &py_ir::ir::TypeDefine,
    ) where
        K: DeclareKind,
        M: Mangler,
    {
        let filter = super::filters::TypeEqual::new(as_);
        let benches: Vec<_> = self[gidx]
            .res
            .iter()
            
            .filter(|(.., status)| matches!(status, BenchStatus::Available(res) if !BenchFilter::<K,M>::satisfy(&filter,&res,defs)))
            .map(|(bench_idx, ..)| Bench::new(gidx, *bench_idx))
            .collect();

        for bench in benches {
            let msg = self[gidx].make_message(format!(
                "expect this to be {}",
                BenchFilter::<K, M>::expect(&filter, &defs)
            ));
            self.delete_bench(bench, msg);
        }
        todo!()
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
        let benches: Vec<_> = self[gidx]
            .res
            .iter()
            .filter(|(.., status)| matches!(status, BenchStatus::Available(res) if !filter.satisfy(&res, defs) ))
            .map(|(bench_idx, ..)| Bench::new(gidx, *bench_idx))
            .collect();

        for bench in benches {
            let msg = self[gidx].make_message(format!("expect this to be {}", filter.expect(defs)));
            self.delete_bench(bench, msg);
        }
    }

    fn get_resources(&self, bench: Bench) -> Result<&Type, &terl::Error> {
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

    /// declare a [`DecalreGroup`]'s result is a type
    ///
    /// return [`Err`] if the type has be decalred and isnot given type,
    /// or non of [`Bench`] match the given tyep
    pub fn declare_type<M: Mangler>(
        &mut self,
        defs: &DefineScope<M>,
        ty: GroupIdx,
        is: &TypeDefine,
        at: terl::Span,
    ) -> terl::Result<()> {
        // TODO: unknown type support
        let assertable = self[ty]
            .res
            .iter()
            .find(|(.., status)| match status {
                BenchStatus::Available(res) => res.as_type(defs) == is,
                _ => false,
            })
            .map(|(idx, ..)| *idx);

        match assertable {
            Some(idx) => {
                let reason = at.make_message(format!("here, type has been declared as {is}"));
                self.make_sure(Bench::new(ty, idx), reason);
                Ok(())
            }
            None => {
                let mut error = at.make_error(format!("type unmatch, expect {is}"));
                for (.., res) in &self[ty].res {
                    match res {
                        BenchStatus::Available(res) => {
                            error = error.append(self[ty].span.make_message(format!(
                                "a bench declared as {}",
                                res.as_type(defs)
                            )));
                        }
                        BenchStatus::Faild(_err) => {
                            error = error.append(
                                self[ty].span.make_message("a bench has been filterd out: "),
                            );
                        }
                    }
                }

                Err(error)
            }
        }
    }

    /// make sure that the [`Bench`] is selected, and give a reasn why other [`Bench`] is not selected
    ///
    /// ... in fact, the reason is where, and which [`Bench`] is selected
    pub(super) fn make_sure(&mut self, bidx: Bench, reason: terl::Message) {
        // take out other benches

        let removed_group = self[bidx.belong_to]
            .res
            .iter()
            .filter(|(.., status)| matches!(status, BenchStatus::Available(..)))
            .filter(|(idx, ..)| **idx != bidx.bench_idx)
            .map(|(idx, ..)| idx)
            .copied()
            .collect::<Vec<_>>();

        for removed in removed_group {
            self.delete_bench(Bench::new(bidx.belong_to, removed), reason.clone())
        }

        // forward delcare result to lower level group
        for sub_bench in self.deps.get(&bidx).unwrap().clone() {
            self.make_sure(sub_bench, reason.clone());
        }
    }

    /// Zhu double eight: is your Nine Clan([`Bench`]) wholesale?
    ///
    /// `delete` a node, and all node which must depend on it
    ///
    /// notice that the `delete` will not remove the [`Bench`], this method
    /// just tag the bench to [`BenchStatus::Faild`] because some reasons
    pub(super) fn delete_bench(&mut self, bench: Bench, reason: terl::Message) {
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
                self.delete_bench(rdep, reason.clone());
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
