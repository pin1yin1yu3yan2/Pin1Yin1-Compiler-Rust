use super::*;
use std::{collections::HashMap, rc::Rc};
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
    pub(crate) res: HashMap<usize, Result<Rc<Type>>>,
}

impl WithSpan for Group {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl Group {
    pub fn new(span: Span, res: HashMap<usize, Result<Rc<Type>>>) -> Self {
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
    pub fn unique(&mut self) -> Option<&mut Result<Rc<Type>>> {
        if !self.declared {
            return None;
        }

        // must be Some
        self.res.values_mut().find(|r| r.is_ok())
    }

    pub fn available(&self) -> impl Iterator<Item = (usize, &Type)> {
        self.res.iter().filter_map(|(idx, status)| match status {
            Ok(ty) => Some((*idx, &**ty)),
            Err(..) => None,
        })
    }

    pub fn removed(&self) -> impl Iterator<Item = (usize, &DeclareError)> {
        self.res.iter().filter_map(|(idx, status)| match status {
            Ok(..) => None,
            Err(err) => Some((*idx, err)),
        })
    }

    pub fn apply_filter<'a, T, B>(
        &'a mut self,
        defs: &'a Defs,
        filter: &'a B,
    ) -> impl Iterator<Item = usize> + 'a
    where
        T: Types,
        B: BenchFilter<T> + 'a,
    {
        let err = DeclareError::Unexpect {
            expect: filter.expect(defs),
        }
        .into_shared();
        self.res
            .iter_mut()
            .filter_map(move |(idx, status)| match status {
                Ok(ty) if !filter.satisfy(ty, defs) => {
                    *status = Err(err.clone());
                    Some(*idx)
                }
                _ => None,
            })
    }
}

pub struct GroupBuilder {
    pub(crate) span: Span,
    pub(crate) builders: Vec<BenchBuilder>,
}

impl GroupBuilder {
    pub fn new(span: Span, builders: Vec<BenchBuilder>) -> GroupBuilder {
        GroupBuilder { span, builders }
    }
}

impl WithSpan for GroupBuilder {
    fn get_span(&self) -> Span {
        self.span
    }
}
