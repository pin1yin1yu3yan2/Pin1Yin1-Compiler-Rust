use std::collections::HashSet;

use terl::{Span, WithSpan};

use crate::{Defs, Type, Types};

use super::{BenchFilter, DeclareMap, GroupIdx};

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
pub enum BenchBuildError {
    NonBenchSelected(GroupIdx, String),
    MultipleSelected(GroupIdx, Vec<usize>),
    ConflictSelected(Bench, Bench),
}

impl BenchBuildError {
    fn make(self, at: Span, map: &DeclareMap, defs: &Defs) -> terl::Error {
        match self {
            BenchBuildError::NonBenchSelected(gidx, expect) => map[gidx]
                .make_error("for this bench, non of depend bench matched")
                .append(at.make_message(format!("expect this to be {expect}"))),
            BenchBuildError::MultipleSelected(gidx, benches) => {
                let mut msg = String::new();

                for bench in benches {
                    use std::fmt::Write;

                    writeln!(
                        &mut msg,
                        "this can be declare to {}",
                        map[Bench::new(gidx, bench)].as_ref().unwrap().display(defs)
                    )
                    .unwrap();
                }

                at.make_error("multiple possible branches are selected to satisfy this bench\n")
                    .append(map[gidx].span.make_message(msg))
            }
            BenchBuildError::ConflictSelected(selected, conflict) => {
                let msg1 = format!(
                    "the bench requires this to be delcared as {}",
                    map[selected].as_ref().unwrap().display(defs)
                );
                let msg2 = format!(
                    "but the bench also requires this to be delcared as {}",
                    map[conflict].as_ref().unwrap().display(defs)
                );

                at.make_error("conflict requirements")
                    .append(map[selected.belong_to].span.make_message(msg1))
                    .append(map[conflict.belong_to].span.make_message(msg2))
            }
        }
    }
}

type BenchBuildAction<'b> = dyn Fn(&DeclareMap, &Defs) -> terl::Result<Bench, BenchBuildError> + 'b;

pub struct BenchBuilder<'b> {
    pub(super) res: Type,
    pub(super) actinons: Vec<Box<BenchBuildAction<'b>>>,
}

impl<'b> BenchBuilder<'b> {
    pub fn new(res: Type) -> Self {
        Self {
            res,
            actinons: vec![],
        }
    }

    pub fn new_depend<T: Types>(&mut self, gidx: GroupIdx, filter: impl BenchFilter<T> + 'b) {
        let action = move |map: &DeclareMap, defs: &Defs| {
            let mut iter = map[gidx].res.iter().filter_map(|(idx, res)| match res {
                Ok(avalable) if filter.satisfy(avalable, defs) => Some(idx),
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
    }

    /// return [BenchStatus] and
    pub fn build(
        self,
        at: Span,
        map: &mut DeclareMap,
        defs: &Defs,
    ) -> terl::Result<(Type, HashSet<Bench>)> {
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

                    let err =
                        BenchBuildError::ConflictSelected(selected, conflict).make(at, map, defs);
                    return Err(err);
                }

                Err(error) => {
                    let err = error.make(at, map, defs);
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
                $crate::BenchBuilder::new(From::from($res))
                    $(.new_filter($filter))*
            ),*]
        }
    };
}
