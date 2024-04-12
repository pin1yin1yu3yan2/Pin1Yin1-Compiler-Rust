use crate::*;
use std::collections::HashSet;
use terl::{Span, WithSpan};

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

type BenchBuildAction<'b> = dyn Fn(&DeclareMap, &Defs) -> Result<Bench> + 'b;

pub struct BenchBuilder<'b> {
    pub(crate) res: Type,
    pub(crate) actinons: Vec<Box<BenchBuildAction<'b>>>,
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
                return Err(DeclareError::NonBenchSelected {
                    expect: filter.expect(defs),
                });
            };

            if let Some(idx) = iter.next() {
                let benches = std::iter::once(*idx).chain(iter.copied());
                return Err(DeclareError::MultSelected {
                    expect: filter.expect(defs),
                    selected: benches.iter(),
                });
            }

            Ok(Bench::new(gidx, idx))
        };

        self.actinons.push(Box::new(action) as _);
    }

    pub fn build(self, map: &mut DeclareMap, defs: &Defs) -> Result<(Type, HashSet<Bench>)> {
        let mut deps: HashSet<Bench> = HashSet::new();
        let mut used_group = HashSet::new();
        for action in self.actinons {
            let bench = (action)(map, defs)?;
            if used_group.contains(&bench.belong_to) && !deps.contains(&bench) {
                let previous = *deps
                    .iter()
                    .find(|bench| bench.belong_to == bench.belong_to)
                    .unwrap();

                let conflict_with = map[previous]
                    .as_ref()
                    .unwrap_or_else(|_| unreachable!())
                    .clone();
                let err = DeclareError::ConflictSelected { conflict_with };
                return Err(err);
            } else {
                deps.insert(bench);
                used_group.insert(bench.belong_to);
            }
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
