use crate::*;
use py_ir::TypeDefine;
use std::collections::{HashMap, HashSet};
use terl::{Span, WithSpan};

/// used to decalre which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareMap {
    pub(crate) groups: Vec<DeclareGroup>,
    /// deps means that [`Branch`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Branch`] will be removed, too
    pub(crate) deps: HashMap<Branch, HashSet<Branch>>,
    pub(crate) rdeps: HashMap<Branch, HashSet<Branch>>,
}

impl DeclareMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn new_group_inner(
        &mut self,
        span: Span,
        failds: HashMap<usize, DeclareError>,
        status: DeclareState,
    ) -> GroupIdx {
        let idx = GroupIdx {
            idx: self.groups.len(),
        };
        self.groups
            .push(DeclareGroup::new(span, idx, failds, status));
        idx
    }

    pub fn new_static_group<I>(&mut self, at: terl::Span, items: I) -> GroupIdx
    where
        I: IntoIterator<Item = Type>,
    {
        self.new_group_inner(at, Default::default(), DeclareState::from_iter(items))
    }

    pub fn build_group(&mut self, gb: GroupBuilder) -> GroupIdx {
        let gidx = GroupIdx::new(self.groups.len());

        let mut alives = HashMap::new();
        let mut failds = HashMap::new();

        for builder in gb.builders {
            let ty = match builder.branch_state {
                Ok(ty) => ty,
                Err(e) => {
                    failds.insert(alives.len() + failds.len(), e);
                    continue;
                }
            };

            for state in builder.depends_grid {
                let branch = Branch::new(gidx, alives.len() + failds.len());
                match state {
                    Ok(deps) => {
                        for dep in deps.iter() {
                            if let Some(val) = self.rdeps.get_mut(dep) {
                                val.insert(branch);
                            }
                        }
                        self.deps.insert(branch, deps);
                        alives.insert(branch.branch_idx, ty.clone());
                    }
                    Err(err) => {
                        failds.insert(branch.branch_idx, err);
                    }
                }
            }
        }

        self.new_group_inner(gb.span, failds, alives.into())
    }

    pub fn apply_filter<T, B>(&mut self, gidx: GroupIdx, defs: &Defs, filter: B)
    where
        T: Types,
        B: BranchFilter<T>,
    {
        let location = self[gidx].get_span();
        let reason = || {
            DeclareError::Unexpect {
                expect: filter.expect(defs),
            }
            .with_location(location)
            .into_shared()
        };
        let removed = self[gidx].delete_branches(|_, ty| !filter.satisfy(ty), reason);

        for (branch, reason) in removed {
            self.delete_branch(branch, reason);
        }
    }

    pub fn merge_group(&mut self, at: terl::Span, base: GroupIdx, from: GroupIdx) {
        let bases = self[from].alives(|alives| {
            alives
                .map(|(branch, ty)| (branch, ty.get_type()))
                .collect::<Vec<_>>()
        });
        let exists = self[base].alives(|alives| {
            alives
                .map(|(branch, ty)| (branch, ty.get_type()))
                .collect::<Vec<_>>()
        });

        // to_branch, from_branch, type
        let merge = exists
            .iter()
            .flat_map(|&(branch, ty)| {
                bases
                    .iter()
                    .filter(move |(.., f_ty)| *f_ty == ty)
                    .map(move |&(f_branch, ..)| (branch, f_branch, ty))
            })
            .collect::<Vec<_>>();

        let (base_kept, from_kept): (HashSet<_>, HashSet<_>) =
            merge.iter().map(|(base, from, _)| (*base, *from)).unzip();

        let removed = bases
            .iter()
            .map(|(branch, ..)| *branch)
            .filter(|branch| !from_kept.contains(branch))
            .chain(
                exists
                    .iter()
                    .map(|(branch, ..)| *branch)
                    .filter(|branch| !base_kept.contains(branch)),
            )
            .collect::<Vec<_>>();

        // TODO: improve error message here
        let delete_reason = DeclareError::Filtered.with_location(at).into_shared();

        for remove in removed {
            self.delete_branch(remove, delete_reason.clone());
        }
    }

    /// declare a [`DeclareGroup`]'s result is a type
    ///
    /// return [`Err`] if the type has be declared and isn't given type,
    /// or non of [`Branch`] match the given type
    pub fn declare_type(&mut self, at: terl::Span, gidx: GroupIdx, expect_ty: &TypeDefine) {
        let group = &mut self[gidx];
        // TODO: unknown type support

        let reason = || {
            DeclareError::Unexpect {
                expect: expect_ty.to_string(),
            }
            .into_shared()
            .with_location(at)
        };
        for (branch, delete) in group.delete_branches(|_, ty| ty.get_type() != expect_ty, reason) {
            self.delete_branch(branch, delete);
        }
    }

    /// Zhu double eight: is your Nine Clan([`Branch`]) wholesale?
    ///
    /// `delete` a node, and all node which must depend on it
    ///
    /// notice that the `delete` will not remove the [`Branch`], this method
    /// just tag the branch to [`BranchStatus::Faild`] because some reasons
    pub(crate) fn delete_branch(&mut self, branch: Branch, reason: DeclareError) {
        // KILL all branch that depend on removed one
        //
        // is it impossiable to be a cycle dep in map?
        let group = &mut self[branch.belong_to];
        group.delete_branches(|previous, _| previous == branch, || reason.clone());
        group.new_error(branch.branch_idx, reason.clone());

        if let Some(rdeps) = self.rdeps.remove(&branch) {
            for rdep in rdeps {
                self.delete_branch(rdep, reason.clone());
            }
        }
        if let Some(deps) = self.deps.remove(&branch) {
            for dep in deps {
                self.rdeps.get_mut(&dep).unwrap().remove(&branch);
            }
        }
    }

    pub fn declare_all(&mut self) -> Result<(), Vec<terl::Error>> {
        let mut errors = vec![];
        for group in &self.groups {
            // un-decalred group
            if !group.is_declared() {
                errors.push(group.make_error());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn get_type(&self, gidx: GroupIdx) -> &TypeDefine {
        self[gidx].result().get_type()
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

impl std::ops::Index<Branch> for DeclareMap {
    type Output = Type;

    fn index(&self, index: Branch) -> &Self::Output {
        self[index.belong_to].get_branch(index)
    }
}
