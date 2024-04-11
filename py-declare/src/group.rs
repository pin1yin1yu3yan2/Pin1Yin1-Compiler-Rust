use crate::{Defs, Type, Types};

use super::{BenchBuilder, BenchFilter};
use std::collections::HashMap;
use terl::{Span, WithSpan};

/// used to represent a group of type, types in group may be declared
/// or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupIdx {
    pub(crate) idx: usize,
}

impl GroupIdx {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub(super) span: Span,
    pub(super) res: HashMap<usize, terl::Result<Type>>,
}

impl WithSpan for Group {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl Group {
    pub fn new(span: Span, res: HashMap<usize, terl::Result<Type>>) -> Self {
        Self { span, res }
    }

    pub fn is_unique(&self) -> bool {
        self.available().count() == 1
    }

    pub fn available(&self) -> impl Iterator<Item = (usize, &Type)> {
        self.res.iter().filter_map(|(idx, status)| match status {
            Ok(ty) => Some((*idx, ty)),
            Err(..) => None,
        })
    }

    pub fn removed(&self) -> impl Iterator<Item = (usize, &terl::Error)> {
        self.res.iter().filter_map(|(idx, status)| match status {
            Ok(..) => None,
            Err(err) => Some((*idx, err)),
        })
    }

    pub fn apply_filter<'a, T, B>(
        &'a mut self,
        defs: &'a Defs,
        filter: &'a B,
        make_error: impl Fn(String) -> terl::Error + 'a,
    ) -> impl Iterator<Item = usize> + 'a
    where
        T: Types,
        B: BenchFilter<T> + 'a,
    {
        self.res
            .iter_mut()
            .filter_map(move |(idx, status)| match status {
                Ok(ty) if !filter.satisfy(ty, defs) => {
                    *status = Err(make_error(filter.expect(defs)));
                    Some(*idx)
                }
                _ => None,
            })
    }
}

pub struct GroupBuilder<'b> {
    pub(super) span: Span,
    pub(super) builders: Vec<BenchBuilder<'b>>,
    pub(super) filtered: Vec<(Type, terl::Error)>,
}

impl GroupBuilder<'_> {
    pub fn new(span: Span, builders: Vec<BenchBuilder>) -> GroupBuilder<'_> {
        GroupBuilder {
            span,
            builders,
            filtered: vec![],
        }
    }

    pub fn pre_filter<T: Types>(self, defs: &Defs, filter: impl BenchFilter<T>) -> Self {
        let mut builders = vec![];
        let mut filtered = self.filtered;

        for builder in self.builders {
            if !filter.satisfy(&builder.res, defs) {
                filtered.push((
                    builder.res,
                    self.span
                        .make_error(format!("expect this to be {}", filter.expect(defs))),
                ))
            } else {
                builders.push(builder)
            }
        }

        Self {
            span: self.span,
            builders,
            filtered,
        }
    }
}

impl WithSpan for GroupBuilder<'_> {
    fn get_span(&self) -> Span {
        self.span
    }
}
