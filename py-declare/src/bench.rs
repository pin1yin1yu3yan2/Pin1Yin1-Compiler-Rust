use crate::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bench {
    /// index of [ReflectDeclare] in [DeclareMap]
    pub(crate) belong_to: UndeclaredTy,
    /// index of possiable of [Declare]
    pub(crate) bench_idx: usize,
}

impl Bench {
    pub fn new(belong_to: UndeclaredTy, bench_idx: usize) -> Self {
        Self {
            belong_to,
            bench_idx,
        }
    }
}

pub struct BenchBuilder {
    pub(crate) main_state: Result<Type>,
    pub(crate) used_groups: HashSet<UndeclaredTy>,
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
        if matches!(self.main_state,Ok(ref ty) if !filter.satisfy(ty) ) {
            let privious =
                std::mem::replace(&mut self.main_state, Err(DeclareError::Empty)).unwrap();
            let err = DeclareError::Unexpect {
                expect: filter.expect(defs),
            }
            .with_previous(privious);
            self.main_state = Err(err)
        }
    }

    pub fn new_depend<T, B>(mut self, map: &mut DeclareMap, gidx: UndeclaredTy, filter: &B) -> Self
    where
        T: Types,
        B: BenchFilter<T>,
    {
        if self.main_state.is_err() {
            return self;
        }

        let new_deps = map[gidx]
            .alive
            .iter()
            .filter_map(|(idx, ty)| if filter.satisfy(ty) { Some(idx) } else { None })
            .map(|bench_idx| Bench::new(gidx, *bench_idx))
            .collect::<Vec<_>>();

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

        self.used_groups.insert(gidx);
        self
    }
}

fn update_deps(
    at: terl::Span,
    used_groups: &HashSet<UndeclaredTy>,
    new_deps: &[Bench],
    mut previous: HashSet<Bench>,
    map: &mut DeclareMap,
    mut reciver: impl FnMut(Result<HashSet<Bench>>),
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
            reciver(Err(conflict_error))
        } else {
            previous.insert(dep);
            reciver(Ok(previous))
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
                reciver(Err(conflict_error))
            } else {
                let mut deps = previous.clone();
                deps.insert(dep);
                reciver(Ok(deps))
            }
        })
    }
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
