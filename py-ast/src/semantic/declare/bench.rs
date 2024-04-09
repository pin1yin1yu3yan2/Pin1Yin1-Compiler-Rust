use std::collections::HashSet;

use terl::{Span, WithSpan};

use crate::semantic::{mangle::Mangler, DefineScope};

use super::{kind::DeclareKind, BenchFilter, DeclareMap, GroupIdx, TypeIdx};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bench {
    /// index of [ReflectDeclare] in [DeclareMap]
    pub(super) belong_to: GroupIdx,
    /// index of possiable of [Declare]
    pub(super) bench_idx: usize,
}

impl Bench {
    pub fn new(belong_to: GroupIdx, bench_idx: usize) -> Self {
        Self {
            belong_to,
            bench_idx,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BenchStatus {
    Available(TypeIdx),
    Faild(terl::Error),
}

#[derive(Debug, Clone)]
pub enum BenchBuildError {
    NonBenchSelected(GroupIdx, String),
    MultipleSelected(GroupIdx, Vec<usize>),
    ConflictSelected(Bench, Bench),
}

impl BenchBuildError {
    fn make<K, M>(self, at: Span, map: &DeclareMap, defs: &DefineScope<M>) -> terl::Error
    where
        M: Mangler,
        K: DeclareKind,
    {
        match self {
            BenchBuildError::NonBenchSelected(gidx, expect) => map[gidx]
                .make_error(
                    "for this bench, non of depend bench matched",
                    terl::ErrorKind::Semantic,
                )
                .append(at, format!("expect this to be {expect}")),
            BenchBuildError::MultipleSelected(gidx, benches) => {
                let mut msg = String::new();

                for bench in benches {
                    use std::fmt::Write;
                    writeln!(
                        &mut msg,
                        "this can be declare to {}",
                        map.display_bench::<K, M>(Bench::new(gidx, bench), defs)
                    );
                }

                let err = at
                    .make_error(
                        "multiple possible branches are selected to satisfy this bench\n",
                        terl::ErrorKind::Semantic,
                    )
                    .append(map[gidx].span, msg);

                err
            }
            BenchBuildError::ConflictSelected(selected, conflict) => {
                let msg1 = format!(
                    "the bench requires this to be delcared as {}",
                    map.display_bench::<K, M>(selected, defs)
                );
                let msg2 = format!(
                    "but the bench also requires this to be delcared as {}",
                    map.display_bench::<K, M>(conflict, defs)
                );

                at.make_error(format!("conflict requirements"), terl::ErrorKind::Semantic)
                    .append(map[selected.belong_to].span, msg1)
                    .append(map[conflict.belong_to].span, msg2)
            }
        }
    }
}

type BenchBuildAction<M: Mangler> =
    dyn Fn(&DeclareMap, &DefineScope<M>) -> terl::Result<Bench, BenchBuildError>;

pub struct BenchBuilder<M: Mangler> {
    pub(super) res: TypeIdx,
    pub(super) actinons: Vec<Box<BenchBuildAction<M>>>,
}

impl<M: Mangler> BenchBuilder<M> {
    pub fn new(res: TypeIdx) -> Self {
        Self {
            res,
            actinons: vec![],
        }
    }

    pub fn new_filter<K, F>(
        mut self,
        gidx: GroupIdx,
        filter: impl BenchFilter<K, M> + 'static,
    ) -> Self
    where
        K: DeclareKind,
    {
        let action = move |map: &DeclareMap, defs: &DefineScope<M>| {
            let mut iter = map[gidx].res.iter().filter_map(|(idx, res)| match res {
                BenchStatus::Available(avalable) if filter.satisfy(avalable, defs) => Some(idx),
                _ => None,
            });
            let Some(&idx) = iter.next() else {
                return Err(BenchBuildError::NonBenchSelected(gidx, filter.expect(defs)));
            };

            if let Some(idx) = iter.next() {
                let benches = std::iter::once(*idx).chain(iter.copied()).collect();
                return Err(BenchBuildError::MultipleSelected(gidx, benches));
            }

            Ok(Bench::new(gidx, idx))
        };

        self.actinons.push(Box::new(action) as _);
        self
    }

    /// return [BenchStatus] and
    pub fn build<K: DeclareKind>(
        self,
        at: Span,
        map: &mut DeclareMap,
        defs: &DefineScope<M>,
    ) -> terl::Result<(TypeIdx, HashSet<Bench>)> {
        let mut deps: HashSet<Bench> = HashSet::new();
        let mut used_group = HashSet::new();
        for action in self.actinons {
            match (action)(map, defs) {
                // use different bench in a group together
                Ok(conflict)
                    if used_group.contains(&conflict.belong_to) && !deps.contains(&conflict) =>
                {
                    let selected = *deps
                        .iter()
                        .find(|bench| bench.belong_to == conflict.belong_to)
                        .unwrap();

                    let err = BenchBuildError::ConflictSelected(selected, conflict)
                        .make::<K, M>(at, map, defs);
                    return Err(err);
                }

                Err(error) => {
                    let err = error.make::<K, M>(at, map, defs);
                    return Err(err);
                }

                // normal case
                Ok(dep_node) => {
                    deps.insert(dep_node);
                    used_group.insert(dep_node.belong_to);
                }
            };
        }

        Ok((self.res, deps))
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
                $crate::semantic::declare::BenchBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}
