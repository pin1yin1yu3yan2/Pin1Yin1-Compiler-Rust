use crate::*;
use py_ir::ir::TypeDefine;
use std::collections::{HashMap, HashSet};
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

    pub fn new_static_group<I>(&mut self, at: terl::Span, items: I) -> GroupIdx
    where
        I: IntoIterator<Item = Type>,
    {
        self.groups.push(Group::new(
            at,
            items.into_iter().enumerate().collect(),
            Default::default(),
        ));
        GroupIdx {
            idx: self.groups.len() - 1,
        }
    }

    pub fn new_group(&mut self, gb: GroupBuilder) -> GroupIdx {
        let gidx = GroupIdx::new(self.groups.len());

        let mut alive = HashMap::new();
        let mut faild = HashMap::new();

        for builder in gb.builders {
            let ty = match builder.main_state {
                Ok(ty) => ty,
                Err(e) => {
                    faild.insert(alive.len() + faild.len(), e);
                    continue;
                }
            };

            for state in builder.states {
                let bench = Bench::new(gidx, alive.len() + faild.len());
                match state {
                    Ok(deps) => {
                        for dep in &deps {
                            if let Some(val) = self.rdeps.get_mut(dep) {
                                val.insert(bench);
                            }
                        }
                        self.deps.insert(bench, deps);
                        alive.insert(bench.bench_idx, ty.clone());
                    }
                    Err(err) => {
                        faild.insert(bench.bench_idx, err);
                    }
                }
            }
        }

        self.groups.push(Group::new(gb.span, alive, faild));

        gidx
    }

    pub fn apply_filter<T, B>(&mut self, gidx: GroupIdx, defs: &Defs, filter: B) -> Result<()>
    where
        T: Types,
        B: BenchFilter<T>,
    {
        let benches: Vec<_> = self[gidx]
            .alive
            .iter()
            .filter(|(.., ty)| !filter.satisfy(ty))
            .map(|(idx, ..)| Bench::new(gidx, *idx))
            .collect();

        let reason = DeclareError::Unexpect {
            expect: filter.expect(defs),
        }
        .with_location(self[gidx].get_span())
        .into_shared();

        for bench in benches {
            self.delete_bench(bench, reason.clone());
        }
        Ok(())
    }

    pub fn merge_group(&mut self, stmt_span: terl::Span, to: GroupIdx, from: GroupIdx) {
        let froms = self[from]
            .alive
            .iter()
            .map(|(&idx, ty)| (Bench::new(from, idx), ty.get_type()))
            .collect::<Vec<_>>();
        let exists = self[to]
            .alive
            .iter()
            .map(|(&idx, ty)| (Bench::new(to, idx), ty.get_type()))
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

        let delete_reason = DeclareError::Filtered
            .with_location(stmt_span)
            .into_shared();

        for remove in removed {
            self.delete_bench(remove, delete_reason.clone());
        }
    }

    /// declare a [`Group`]'s result is a type
    ///
    /// return [`Err`] if the type has be decalred and isn't given type,
    /// or non of [`Bench`] match the given tyep
    pub fn declare_type(
        &mut self,
        stmt_span: terl::Span,
        val_ty: GroupIdx,
        expect_ty: &TypeDefine,
    ) {
        // TODO: unknown type support
        let any_match = self[val_ty]
            .alive
            .iter()
            .find(|(.., ty)| ty.get_type() == expect_ty)
            .map(|(idx, ..)| *idx);

        match any_match {
            Some(matched) => {
                let reason = DeclareError::Declared {
                    declare_as: self[val_ty].alive[&matched].clone(),
                }
                .into_shared();

                self.make_sure(Bench::new(val_ty, matched), reason);
            }
            None => {
                let error = DeclareError::Unexpect {
                    expect: expect_ty.to_string(),
                }
                .into_shared()
                .with_location(stmt_span);

                for (k, previous) in std::mem::take(&mut self[val_ty].alive) {
                    self[val_ty]
                        .faild
                        .insert(k, error.clone().with_previous(previous));
                }
            }
        }
    }

    /// make sure that the [`Bench`] is selected, and give a reasn why other [`Bench`] is not selected
    ///
    /// ... in fact, the reason is where, and which [`Bench`] is selected
    pub(crate) fn make_sure(&mut self, bidx: Bench, reason: DeclareError) {
        // take out other benches

        let removed_group = self[bidx.belong_to]
            .alive
            .iter()
            .filter(|(idx, ..)| **idx != bidx.bench_idx)
            .map(|(idx, ..)| idx)
            .copied()
            .collect::<Vec<_>>();

        for removed in removed_group {
            self.delete_bench(Bench::new(bidx.belong_to, removed), reason.clone());
        }

        // forward delcare result to lower level group
        if let Some(val) = self.deps.get(&bidx) {
            for sub_bench in val.clone() {
                self.make_sure(sub_bench, reason.clone());
            }
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

        let is_unique = self[bench.belong_to].unique().is_some();

        let previous = self[bench.belong_to]
            .alive
            .remove(&bench.bench_idx)
            .unwrap();

        let err = if is_unique {
            DeclareError::UniqueDeleted {
                reason: Box::new(reason.clone()),
            }
        } else {
            reason.clone()
        }
        .with_previous(previous);

        self[bench.belong_to].faild.insert(bench.bench_idx, err);

        if let Some(rdeps) = self.rdeps.remove(&bench) {
            for rdep in rdeps {
                self.delete_bench(rdep, reason.clone());
            }
        }
        if let Some(deps) = self.deps.remove(&bench) {
            for dep in deps {
                self.rdeps.get_mut(&dep).unwrap().remove(&bench);
            }
        }
    }

    pub fn declare_all(&mut self) -> Vec<terl::Error> {
        let mut errors = vec![];
        for group in &self.groups {
            // un-decalred group
            if group.unique().is_none() {
                let mut err = group.make_error("cant infer type");
                if group.alive.is_empty() {
                    err += "this cant be decalred as any type!";
                } else {
                    err += "this cant be decalred as:";
                    for alive in group.alive.values() {
                        err += format!("\t{alive}")
                    }
                }
                for (idx, faild) in group.faild.values().enumerate() {
                    err += format!("faild bench <{idx}>:");
                    err.extend(faild.generate());
                }
                errors.push(err);
            }
        }
        errors
    }

    pub fn get_type(&self, gidx: GroupIdx) -> &TypeDefine {
        self[gidx].unique().unwrap().get_type()
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
    type Output = Type;

    fn index(&self, index: Bench) -> &Self::Output {
        &self[index.belong_to].alive[&index.bench_idx]
    }
}
