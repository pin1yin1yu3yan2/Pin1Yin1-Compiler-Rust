use crate::*;
use py_ir::types::TypeDefine;
use std::collections::{HashMap, HashSet};
use terl::{Span, WithSpan};

/// used to declare which overload of function is called, or which possiable type is
///
#[derive(Default, Debug)]
pub struct DeclareGraph {
    pub(crate) groups: Vec<DeclareGroup>,
    /// deps means that [`Branch`] depend **ALL** of them
    ///
    /// if any of them is impossible, the [`Branch`] will be removed, too
    pub(crate) deps: HashMap<Branch, HashSet<Branch>>,
    pub(crate) rdeps: HashMap<Branch, HashSet<Branch>>,
}

impl DeclareGraph {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert_depends(&mut self, who: Branch, depend: HashSet<Branch>) {
        if depend.is_empty() {
            return;
        }
        for &depend in &depend {
            self.rdeps.entry(depend).or_default().insert(who);
        }
        self.deps.entry(who).or_default().extend(depend);
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

        #[derive(Debug)]
        enum BranchMark {
            Used,
            Error(DeclareError),
        }

        let mut used_branches: HashMap<GroupIdx, HashMap<usize, BranchMark>> = HashMap::new();
        for branch_builder in gb.branches {
            let ty = match branch_builder.state {
                Ok(ty) => ty,
                Err(e) => {
                    failds.insert(alives.len() + failds.len(), e);
                    continue;
                }
            };

            let use_branch = |branch: Branch| -> Branch {
                used_branches
                    .entry(branch.belong_to)
                    .or_default()
                    .insert(branch.branch_idx, BranchMark::Used);
                branch
            };

            match branch_builder.depends.merge_depends(use_branch) {
                // may the branch doesnot depend on any other branches
                Ok(branch_depends) if branch_depends.is_empty() => {
                    let new_branch = Branch::new(gidx, alives.len() + failds.len());
                    alives.insert(new_branch.branch_idx, ty.clone());
                }
                Ok(branch_depends) => {
                    for branch_depends in branch_depends {
                        let new_branch = Branch::new(gidx, alives.len() + failds.len());
                        self.insert_depends(new_branch, branch_depends);
                        alives.insert(new_branch.branch_idx, ty.clone());
                    }
                }
                Err(group_errors) => {
                    for (group, errors) in group_errors {
                        let group = used_branches.entry(group).or_default();
                        for (branch, error) in errors {
                            group.entry(branch).or_insert(BranchMark::Error(error));
                        }
                    }
                }
            };
        }

        let new_group = self.new_group_inner(gb.span, failds, alives.into());

        for (group, mut branch_marks) in used_branches {
            // use hashmap avoiding remove same branch more than once
            let remove: HashMap<_, _> = self[group].filter_alive(|branch, ty| {
                match branch_marks.remove(&branch.branch_idx) {
                    Some(BranchMark::Used) => Ok((branch, ty)),
                    Some(BranchMark::Error(error)) => Err(error.with_location(gb.span)),
                    None => Err(DeclareError::NeverUsed {
                        in_group: gb.span,
                        reason: None,
                    }),
                }
                .map_err(|e| e.into_shared())
            });
            //
            for (branch, reason) in remove {
                self.remove_branch(branch, reason);
            }
        }

        new_group
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
        let removed = self[gidx].remove_branches(|_, ty| !filter.satisfy(ty), reason);

        for (branch, reason) in removed {
            self.remove_branch(branch, reason);
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
        let remove_reason = DeclareError::Filtered.with_location(at).into_shared();

        for remove in removed {
            self.remove_branch(remove, remove_reason.clone());
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
            .with_location(at)
            .into_shared()
        };
        for (branch, remove) in group.remove_branches(|_, ty| ty.get_type() != expect_ty, reason) {
            self.remove_branch(branch, remove);
        }
    }

    /// Zhu double eight: is your Nine Clan([`Branch`]) wholesale?
    ///
    /// `remove` a node, and all node which must depend on it, and then a generate a error
    /// with [`Type`] which previous branch stored in
    ///
    /// # Note:
    ///
    /// make sure the reason passed in are wrapped by rc(by calling [`DeclareError::into_shared`])
    pub(crate) fn remove_branch(&mut self, branch: Branch, reason: DeclareError) {
        // is it impossiable to be a cycle dep in map?
        let group = &mut self[branch.belong_to];
        let group_loc = group.get_span();
        {
            let reason = group.remove_branch(branch.branch_idx, reason.clone());
            group.push_error(branch.branch_idx, reason.clone());
        }

        // remove all branches depend on removed branch
        if let Some(rdeps) = self.rdeps.remove(&branch) {
            for rdep in rdeps {
                self.remove_branch(rdep, reason.clone());
            }
        }
        // remove the record of all branch which removed branch depend on
        if let Some(deps) = self.deps.remove(&branch) {
            for dep in deps {
                match self.rdeps.get_mut(&dep) {
                    Some(rdeps) if rdeps.len() == 1 => {
                        let reason = DeclareError::NeverUsed {
                            in_group: group_loc,
                            reason: Some(reason.clone().into()),
                        };
                        self.remove_branch(dep, reason);
                    }
                    Some(rdeps) => {
                        rdeps.remove(&branch);
                    }
                    None => {}
                }
            }
        }
    }

    pub fn declare_all(&mut self) -> Result<(), Vec<terl::Error>> {
        let mut errors = vec![];
        for group in &self.groups {
            // un-declared group
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

impl std::ops::Index<GroupIdx> for DeclareGraph {
    type Output = DeclareGroup;

    fn index(&self, index: GroupIdx) -> &Self::Output {
        &self.groups[index.idx]
    }
}

impl std::ops::IndexMut<GroupIdx> for DeclareGraph {
    fn index_mut(&mut self, index: GroupIdx) -> &mut Self::Output {
        &mut self.groups[index.idx]
    }
}

impl std::ops::Index<Branch> for DeclareGraph {
    type Output = Type;

    fn index(&self, index: Branch) -> &Self::Output {
        self[index.belong_to].get_branch(index)
    }
}
