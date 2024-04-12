use crate::*;
use py_ir::ir::TypeDefine;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use terl::WithSpan;

/// used to decalre which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareMap {
    pub(crate) groups: Vec<Group>,
    /// deps means that [`Bench`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Bench`] will be removed, too
    pub(crate) deps: HashMap<Bench, HashSet<Bench>>,
    pub(crate) rdeps: HashMap<Bench, HashSet<Bench>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_static_group(
        &mut self,
        at: terl::Span,
        items: impl IntoIterator<Item = TypeDefine>,
    ) -> GroupIdx {
        self.groups.push(Group::new(
            at,
            items
                .into_iter()
                .map(|ty| Ok(ty.into()))
                .enumerate()
                .collect(),
        ));
        GroupIdx {
            idx: self.groups.len() - 1,
        }
    }

    pub fn new_group(&mut self, defs: &Defs, gb: GroupBuilder) -> Result<GroupIdx> {
        let gidx = GroupIdx::new(self.groups.len());

        let mut bench_res = HashMap::new();

        for (bench_idx, builder) in gb.builders.into_iter().enumerate() {
            let bench = Bench::new(gidx, bench_idx);
            let status = builder.build(self, defs).map(|(res, deps)| {
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

    pub fn apply_filter<T>(
        &mut self,
        gidx: GroupIdx,
        defs: &Defs,
        filter: impl BenchFilter<T>,
    ) -> Result<()>
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
            self.delete_bench(bench, msg)?;
        }
        Ok(())
    }

    pub fn merge_group(
        &mut self,
        defs: &Defs,
        stmt_span: terl::Span,
        to: GroupIdx,
        from: GroupIdx,
    ) -> Result<()> {
        let froms = self[from]
            .available()
            .map(|(idx, ty)| (Bench::new(from, idx), ty.get_type(defs)))
            .collect::<Vec<_>>();
        let exists = self[to]
            .available()
            .map(|(idx, ty)| (Bench::new(to, idx), ty.get_type(defs)))
            .collect::<Vec<_>>();
        // to_bench, from_bench, type
        let merge = exists
            .iter()
            .flat_map(|&(bench, ty)| {
                froms
                    .iter()
                    .filter(move |(.., f_ty)| *f_ty == ty)
                    .map(move |&(f_bench, ..)| (bench, f_bench, ty))
            })
            .collect::<Vec<_>>();

        let (to_keeped, from_keeped): (HashSet<_>, HashSet<_>) =
            merge.iter().map(|(to, from, _)| (*to, *from)).unzip();

        let delete_reason = stmt_span.make_message("filtered here");

        let removed = froms
            .iter()
            .map(|(bidx, ..)| *bidx)
            .filter(|bidx| !from_keeped.contains(bidx))
            .chain(
                exists
                    .iter()
                    .map(|(bidx, ..)| *bidx)
                    .filter(|bidx| !to_keeped.contains(bidx)),
            )
            .collect::<Vec<_>>();

        for remove in removed {
            self.delete_bench(remove, delete_reason.clone())?;
        }

        Ok(())
    }

    /// declare a [`Group`]'s result is a type
    ///
    /// return [`Err`] if the type has be decalred and isn't given type,
    /// or non of [`Bench`] match the given tyep
    pub fn declare_type(
        &mut self,
        defs: &Defs,
        stmt_span: terl::Span,
        val_ty: GroupIdx,
        expect_ty: &TypeDefine,
    ) -> Result<()> {
        // TODO: unknown type support
        let any_match = self[val_ty]
            .available()
            .find(|(.., ty)| ty.get_type(defs) == expect_ty)
            .map(|(idx, ..)| idx);

        match any_match {
            Some(matched) => {
                let reason = DeclareError::ReasonDeclared {
                    declare_as: Rc::new(self[val_ty].res[&matched].unwrap().clone()),
                };
                self.make_sure(Bench::new(val_ty, matched), reason);
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
    pub(crate) fn make_sure(&mut self, bidx: Bench, reason: DeclareError) {
        // take out other benches
        self[bidx.belong_to].declared = true;

        let removed_group = self[bidx.belong_to]
            .res
            .iter()
            .filter(|(.., status)| status.is_ok())
            .filter(|(idx, ..)| **idx != bidx.bench_idx)
            .map(|(idx, ..)| idx)
            .copied()
            .collect::<Vec<_>>();

        for removed in removed_group {
            self.delete_bench(Bench::new(bidx.belong_to, removed), reason.clone());
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
    pub(crate) fn delete_bench(&mut self, bench: Bench, reason: DeclareError) {
        // KILL all bench that depend on removed one
        //
        // is it impossiable to be a cycle dep in map?

        // TOOD: use custom error type to replace make_error everywhere

        if let Some(unique) = self[bench.belong_to].unique() {
            let empty = Err(DeclareError::Empty);
            let previous = std::mem::replace(unique, empty).unwrap();
            let err = DeclareError::UniqueDeleted {
                previous,
                reason: Box::new(reason.clone()),
            };
            *unique = Err(err);
            // "Ok"
            return;
        }

        let err = DeclareError::RemovedDuoDeclared {
            reason: Box::new(reason.clone()),
        };
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
    type Output = Result<Type, DeclareError>;

    fn index(&self, index: Bench) -> &Self::Output {
        &self[index.belong_to].res[&index.bench_idx]
    }
}
