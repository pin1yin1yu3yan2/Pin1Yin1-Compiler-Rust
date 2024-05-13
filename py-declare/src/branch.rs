use crate::*;
use std::collections::HashSet;

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

pub struct BranchBuilder {
    pub(crate) branch_state: Result<Type>,
    pub(crate) used_groups: HashSet<GroupIdx>,
    pub(crate) depends_grid: Vec<Result<HashSet<Branch>>>,
}

impl From<Type> for BranchBuilder {
    fn from(value: Type) -> Self {
        Self::new(value)
    }
}

impl BranchBuilder {
    pub fn new(ty: Type) -> Self {
        Self {
            branch_state: Ok(ty),
            used_groups: Default::default(),
            depends_grid: vec![Ok(HashSet::new())],
        }
    }

    pub const fn is_ok(&self) -> bool {
        self.branch_state.is_ok()
    }

    pub fn filter_self<T, B>(&mut self, defs: &Defs, filter: &B)
    where
        T: Types,
        B: BranchFilter<T>,
    {
        if matches!(self.branch_state,Ok(ref ty) if !filter.satisfy(ty) ) {
            let privious =
                std::mem::replace(&mut self.branch_state, Err(DeclareError::Empty)).unwrap();
            let err = DeclareError::Unexpect {
                expect: filter.expect(defs),
            }
            .with_previous(privious);
            self.branch_state = Err(err)
        }
    }

    pub fn new_depend<T, B>(mut self, map: &mut DeclareMap, depend: GroupIdx, filter: &B) -> Self
    where
        T: Types,
        B: BranchFilter<T>,
    {
        if self.branch_state.is_err() {
            return self;
        }

        let satisfy_branches = map[depend].alives(|alives| {
            alives
                .filter(|(.., ty)| filter.satisfy(ty))
                .map(|(branch, ..)| branch)
                .collect::<Vec<_>>()
        });

        self.depends_grid = self
            .depends_grid
            .into_iter()
            .fold(vec![], |mut states, state| {
                match state {
                    Ok(previous) => {
                        let at = filter.get_span();
                        let used_groups = &self.used_groups;
                        update_deps(
                            at,
                            used_groups,
                            &satisfy_branches,
                            previous,
                            map,
                            &mut states,
                        );
                    }
                    Err(e) => states.push(Err(e)),
                }
                states
            });

        self.used_groups.insert(depend);
        self
    }
}

fn update_deps(
    at: terl::Span,
    used_groups: &HashSet<GroupIdx>,
    new_deps: &[Branch],
    mut previous: HashSet<Branch>,
    map: &mut DeclareMap,
    states: &mut Vec<Result<HashSet<Branch>>>,
) {
    if new_deps.len() == 1 {
        let dep = new_deps[0];
        if used_groups.contains(&dep.belong_to) && !previous.contains(&dep) {
            let previous = previous
                .iter()
                .find(|p| p.belong_to == dep.belong_to)
                .unwrap();

            let conflict_error = DeclareError::ConflictSelected {
                conflict_with: map[*previous].clone(),
                this: map[dep].clone(),
            }
            .with_location(at);
            states.push(Err(conflict_error));
        } else {
            previous.insert(dep);
            states.push(Ok(previous));
        }
    } else {
        new_deps.iter().for_each(|&dep| {
            if used_groups.contains(&dep.belong_to) && !previous.contains(&dep) {
                let previous = previous
                    .iter()
                    .find(|p| p.belong_to == dep.belong_to)
                    .unwrap();

                let conflict_error = DeclareError::ConflictSelected {
                    conflict_with: map[*previous].clone(),
                    this: map[dep].clone(),
                }
                .with_location(at);
                states.push(Err(conflict_error));
            } else {
                let mut new_deps = previous.clone();
                new_deps.insert(dep);
                states.push(Ok(new_deps));
            }
        })
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
                $crate::BranchBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}
