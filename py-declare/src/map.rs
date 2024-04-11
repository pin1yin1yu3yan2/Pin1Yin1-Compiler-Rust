use super::{Bench, BenchFilter, Group, GroupBuilder, GroupIdx};
use crate::{mir::TypeDefine, Defs, Type, Types};

use std::collections::{HashMap, HashSet};
use terl::WithSpan;

/// used to decalre which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareMap {
    pub(super) groups: Vec<Group>,
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

    pub fn new_group(&mut self, defs: &Defs, gb: GroupBuilder) -> terl::Result<GroupIdx> {
        let gidx = GroupIdx::new(self.groups.len());

        let mut bench_res = HashMap::new();

        for (bench_idx, builder) in gb.builders.into_iter().enumerate() {
            let bench = Bench::new(gidx, bench_idx);
            let status = builder.build(gb.span, self, defs).map(|(res, deps)| {
                for &dep in &deps {
                    self.rdeps.get_mut(&dep).unwrap().insert(bench);
                }
                self.deps.insert(bench, deps);
                res
            });
            bench_res.insert(bench_idx, status);
        }

        self.groups.push(Group::new(gb.span, bench_res));

        Ok(gidx)
    }

    pub fn decalre_as<T>(&mut self, gidx: GroupIdx, defs: &Defs, as_: &py_ir::ir::TypeDefine)
    where
        T: Types,
    {
        let filter: &dyn BenchFilter<T> = &super::filters::TypeEqual::new(as_);

        let benches: Vec<_> = self[gidx]
            .res
            .iter()
            .filter_map(|(idx, status)| Some((idx, status.as_ref().ok()?)))
            .filter(|(.., ty)| filter.satisfy(ty, defs))
            .map(|(bench_idx, ..)| Bench::new(gidx, *bench_idx))
            .collect();

        for bench in benches {
            let msg = self[gidx].make_message(format!("expect this to be {}", filter.expect(defs)));
            self.delete_bench(bench, msg);
        }
    }

    pub fn apply_filter<T>(&mut self, gidx: GroupIdx, defs: &Defs, filter: impl BenchFilter<T>)
    where
        T: Types,
    {
        let span = self[gidx].get_span();
        let benches: Vec<_> = self[gidx]
            .apply_filter(defs, &filter, |expect| {
                span.make_error(format!("expect this to be {}", expect))
            })
            .map(|idx| Bench::new(gidx, idx))
            .collect();

        for bench in benches {
            let msg = self[gidx].make_message(format!("expect this to be {}", filter.expect(defs)));
            self.delete_bench(bench, msg);
        }
    }

    pub fn merge_group(
        &mut self,
        stmt_span: terl::Span,
        to: GroupIdx,
        from: GroupIdx,
    ) -> terl::Result<()> {
        Ok(())
    }

    /// declare a [`DecalreGroup`]'s result is a type
    ///
    /// return [`Err`] if the type has be decalred and isnot given type,
    /// or non of [`Bench`] match the given tyep
    pub fn declare_type(
        &mut self,
        defs: &Defs,
        stmt_span: terl::Span,
        val_ty: GroupIdx,
        expect_ty: &TypeDefine,
    ) -> terl::Result<()> {
        // TODO: unknown type support
        let assertable = self[val_ty]
            .res
            .iter()
            .find(|(.., status)| match status {
                Ok(ty) => ty.get_type(defs) == expect_ty,
                _ => false,
            })
            .map(|(idx, ..)| *idx);

        match assertable {
            Some(idx) => {
                let reason =
                    stmt_span.make_message(format!("here, type has been declared as {expect_ty}"));
                self.make_sure(Bench::new(val_ty, idx), reason);
                Ok(())
            }
            None => {
                let mut error = stmt_span.make_error(format!("type unmatch, expect {expect_ty}"));

                for (.., res) in &self[val_ty].res {
                    match res {
                        Ok(decalre_as) => {
                            error += self[val_ty].span.make_message(format!(
                                "a bench declared as {}",
                                decalre_as.get_type(defs)
                            ));
                        }
                        Err(err) => {
                            error += self[val_ty]
                                .span
                                .make_message("a bench has been filterd out: ");
                            error += err.clone();
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
            .filter(|(.., status)| status.is_ok())
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
        //
        // is it impossiable to be a cycle dep in map?
        let err = self[bench.belong_to]
            .make_error("the bench has been filtered out:")
            .append(reason.clone());
        *self[bench.belong_to].res.get_mut(&bench.bench_idx).unwrap() = Err(err);

        for rdep in self.rdeps.remove(&bench).unwrap() {
            self.delete_bench(rdep, reason.clone());
        }

        for dep in self.deps.remove(&bench).unwrap() {
            self.rdeps.get_mut(&dep).unwrap().remove(&bench);
        }
    }
}

impl std::ops::Index<GroupIdx> for DeclareMap {
    type Output = Group;

    fn index(&self, index: GroupIdx) -> &Self::Output {
        &self.groups[index.idx]
    }
}

impl std::ops::IndexMut<GroupIdx> for DeclareMap {
    fn index_mut(&mut self, index: GroupIdx) -> &mut Self::Output {
        &mut self.groups[index.idx]
    }
}

impl std::ops::Index<Bench> for DeclareMap {
    type Output = terl::Result<Type>;

    fn index(&self, index: Bench) -> &Self::Output {
        &self[index.belong_to].res[&index.bench_idx]
    }
}
