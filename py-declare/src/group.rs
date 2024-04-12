use super::*;
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
    pub(crate) declared: bool,
    pub(crate) span: Span,
    // this can only be init once, or keep empty to init later(UNKNOWN type)
    pub(crate) res: HashMap<usize, Result<Type>>,
}

impl WithSpan for Group {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl Group {
    pub fn new(span: Span, res: HashMap<usize, Result<Type>>) -> Self {
        Self {
            span,
            res,
            declared: false,
        }
    }

    /// return a [`Ok`] to use [`std::mem::swap`]
    /// to replace previous_result
    ///
    /// this method return [`None`] if provious declare result is removed,
    /// or the Group is even not declared
    pub fn unique(&mut self) -> Option<&mut Result<Type>> {
        if !self.declared {
            return None;
        }

        // must be Some
        self.res.values_mut().filter(|r| r.is_ok()).next()
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
    pub(crate) span: Span,
    pub(crate) builders: Vec<BenchBuilder<'b>>,
    pub(crate) filtered: Vec<(Type, terl::Error)>,
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
