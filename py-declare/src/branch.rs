use crate::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Branch {
    /// index of [ReflectDeclare] in [DeclareMap]
    pub(crate) belong_to: GroupIdx,
    /// index of possiable of [Declare]
    pub(crate) branch_idx: usize,
}

impl Branch {
    pub fn new(belong_to: GroupIdx, branch_idx: usize) -> Self {
        Self {
            belong_to,
            branch_idx,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct BranchDepend {
    pub(crate) depends: HashMap<GroupIdx, HashSet<usize>>,
    pub(crate) failds: GroupError,
}

type GroupError = HashMap<GroupIdx, HashMap<usize, DeclareError>>;

impl BranchDepend {
    pub fn new_depends<E>(&mut self, group: GroupIdx, branches: HashSet<usize>, filtered: E)
    where
        E: Fn(Branch) -> DeclareError,
    {
        let new_depend = match self.depends.remove(&group) {
            Some(previous) => {
                let new_depend = previous
                    .into_iter()
                    .filter(|previous| {
                        if branches.contains(previous) {
                            true
                        } else {
                            let branch = Branch::new(group, *previous);
                            self.failds
                                .entry(group)
                                .or_default()
                                .insert(branch.branch_idx, filtered(branch));
                            false
                        }
                    })
                    .collect();

                new_depend
            }
            None => branches,
        };

        self.depends.insert(group, new_depend);
    }

    pub fn merge_depends<U>(mut self, mut use_branch: U) -> Result<Vec<HashSet<Branch>>, GroupError>
    where
        U: FnMut(Branch) -> Branch,
    {
        {
            let mut errors = GroupError::new();
            for (group, ..) in self
                .depends
                .iter()
                .filter(|(_, branches)| branches.is_empty())
            {
                errors.insert(*group, self.failds.remove(group).unwrap_or_default());
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }

        let mut groups: Vec<HashSet<Branch>> = vec![HashSet::default()];
        for (group, depend) in self.depends {
            let len = groups.len();
            // resize depends
            //
            // depend.len() >= 1 because depend.len() == 0 is an error and solved above
            for _ in 1..depend.len() {
                groups.extend(groups.clone());
            }
            // insert
            for (new_depend_idx, new_depend) in depend.into_iter().enumerate() {
                let branch = use_branch(Branch::new(group, new_depend));
                for depends_idx in 0..len {
                    groups[new_depend_idx * len + depends_idx].insert(branch);
                }
            }
        }

        Ok(groups)
    }
}

pub struct BranchesBuilder {
    pub(crate) state: Result<Type>,
    pub(crate) depends: BranchDepend,
}

impl From<Type> for BranchesBuilder {
    fn from(value: Type) -> Self {
        Self::new(value)
    }
}

impl BranchesBuilder {
    pub fn new(ty: Type) -> Self {
        Self {
            state: Ok(ty),
            depends: Default::default(),
        }
    }

    /// # Return
    ///
    /// is self.state ok
    pub fn filter_self<T, B>(&mut self, defs: &Defs, filter: &B) -> bool
    where
        T: Types,
        B: BranchFilter<T>,
    {
        if self.state.as_ref().is_ok_and(|ty| !filter.satisfy(ty)) {
            // update state to error
            let privious = std::mem::replace(&mut self.state, Err(DeclareError::Empty)).unwrap();
            let err = DeclareError::Unexpect {
                expect: filter.expect(defs),
            }
            .with_previous(privious);
            self.state = Err(err)
        }
        self.state.is_ok()
    }

    /// let the bench depend on benches which satisfy the filter in group
    pub fn new_depend<T, B>(
        mut self,
        map: &mut DeclareGraph,
        defs: &Defs,
        depend: GroupIdx,
        filter: &B,
    ) -> Self
    where
        T: Types,
        B: BranchFilter<T>,
    {
        if self.state.is_err() {
            return self;
        }

        let satisfy_branches = map[depend].alives(|alives| {
            alives
                .filter(|(.., ty)| filter.satisfy(ty))
                .map(|(branch, ..)| branch.branch_idx)
                .collect::<HashSet<_>>()
        });

        let filtered_reason = DeclareError::Unexpect {
            expect: filter.expect(defs),
        }
        .with_location(filter.get_span())
        .into_shared();

        let filtered_reason =
            |branch: Branch| filtered_reason.clone().with_previous(map[branch].clone());

        self.depends
            .new_depends(depend, satisfy_branches, filtered_reason);

        self
    }
}

#[macro_export]
macro_rules! branches {
    {
        $(
            ($($filter:expr),*) => $res:expr
        ),*
    } => {
        {
            vec![$(
                $crate::BranchesBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}
