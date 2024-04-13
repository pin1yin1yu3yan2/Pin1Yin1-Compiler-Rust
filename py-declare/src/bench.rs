use crate::*;
use std::{collections::HashSet, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bench {
    /// index of [ReflectDeclare] in [DeclareMap]
    pub(crate) belong_to: GroupIdx,
    /// index of possiable of [Declare]
    pub(crate) bench_idx: usize,
}

impl Bench {
    pub fn new(belong_to: GroupIdx, bench_idx: usize) -> Self {
        Self {
            belong_to,
            bench_idx,
        }
    }
}

pub struct BenchBuilder {
    pub(crate) main_state: Result<Type>,
    pub(crate) used_groups: HashSet<GroupIdx>,
    pub(crate) states: Vec<Result<HashSet<Bench>>>,
}

impl BenchBuilder {
    pub fn new(ty: Type) -> Self {
        Self {
            main_state: Ok(ty),
            used_groups: Default::default(),
            states: vec![Ok(HashSet::new())],
        }
    }

    pub const fn is_ok(&self) -> bool {
        self.main_state.is_ok()
    }

    pub fn filter_self<T, B>(&mut self, defs: &Defs, filter: &B)
    where
        T: Types,
        B: BenchFilter<T>,
    {
        if matches!(self.main_state,Ok(ref ty) if !filter.satisfy(ty, defs) ) {
            let privious =
                std::mem::replace(&mut self.main_state, Err(DeclareError::Empty)).unwrap();
            let err = DeclareError::Unexpect {
                expect: filter.expect(defs),
            }
            .with_previous(Rc::new(privious));
            self.main_state = Err(err)
        }
    }

    pub fn new_depend<T, B>(
        mut self,
        map: &mut DeclareMap,
        defs: &Defs,
        gidx: GroupIdx,
        filter: &B,
    ) -> Self
    where
        T: Types,
        B: BenchFilter<T>,
    {
        if self.main_state.is_err() {
            return self;
        }

        let new_deps = map[gidx]
            .res
            .iter()
            .filter_map(|(idx, res)| match res {
                Ok(avalable) if filter.satisfy(avalable, defs) => Some(idx),
                _ => None,
            })
            .map(|bench_idx| Bench::new(gidx, *bench_idx))
            .collect::<Vec<_>>();

        if new_deps.is_empty() {
            let err = DeclareError::NonBenchSelected {
                expect: filter.expect(defs),
            }
            .with_location(filter.get_span())
            .into_shared();

            for (.., not_selected) in &mut map[gidx].res {
                if not_selected.is_ok() {
                    *not_selected = Err(err.clone())
                }
            }
        } else {
            self.states = self.states.into_iter().fold(vec![], |mut states, state| {
                match state {
                    Ok(deps) => {
                        let reciver = |item| states.push(item);
                        let at = filter.get_span();
                        let used_groups = &self.used_groups;
                        update_deps(at, used_groups, &new_deps, deps, map, reciver);
                    }
                    Err(e) => states.push(Err(e)),
                }
                states
            });
        }

        self.used_groups.insert(gidx);
        self
    }
}

fn update_deps(
    at: terl::Span,
    used_groups: &HashSet<GroupIdx>,
    new_deps: &[Bench],
    mut previous: HashSet<Bench>,
    map: &mut DeclareMap,
    mut reciver: impl FnMut(Result<HashSet<Bench>>),
) {
    if new_deps.len() == 1 {
        let dep = new_deps[0];
        if used_groups.contains(&dep.belong_to) && !previous.contains(&dep) {
            reciver(Err(conflict_error(map, dep, at)))
        } else {
            previous.insert(dep);
            reciver(Ok(previous))
        }
    } else {
        new_deps.iter().for_each(|&dep| {
            if used_groups.contains(&dep.belong_to) && !previous.contains(&dep) {
                reciver(Err(conflict_error(map, dep, at)))
            } else {
                let mut deps = previous.clone();
                deps.insert(dep);
                reciver(Ok(deps))
            }
        })
    }
}

fn conflict_error(map: &mut DeclareMap, dep: Bench, location: terl::Span) -> DeclareError {
    DeclareError::ConflictSelected {
        conflict_with: map[dep].clone().unwrap(),
    }
    .with_location(location)
}

#[macro_export]
macro_rules! benches {
    {
        $(
            ($($filter:expr),*) => $res:expr
        ),*
    } => {
        {
            vec![$(
                $crate::BenchBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}
