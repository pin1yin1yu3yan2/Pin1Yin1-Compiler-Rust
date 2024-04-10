use super::{kind::DeclareKind, BenchBuilder, BenchFilter, BenchStatus, Type};
use crate::semantic::{mangle::Mangler, DefineScope};
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
pub struct DeclareGroup {
    pub(super) span: Span,
    pub(super) res: HashMap<usize, BenchStatus>,
}

impl WithSpan for DeclareGroup {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl DeclareGroup {
    pub fn new(span: Span, res: HashMap<usize, BenchStatus>) -> Self {
        Self { span, res }
    }
}

pub struct GroupBuilder<'b, M: Mangler> {
    pub(super) span: Span,
    pub(super) builders: Vec<BenchBuilder<'b, M>>,
    pub(super) filtered: Vec<(Type, terl::Error)>,
}

impl<M: Mangler> GroupBuilder<'_, M> {
    pub fn new(span: Span, builders: Vec<BenchBuilder<M>>) -> GroupBuilder<'_, M> {
        GroupBuilder {
            span,
            builders,
            filtered: vec![],
        }
    }

    pub fn pre_filter<K, B>(self, defs: &DefineScope<M>, filter: B) -> Self
    where
        K: DeclareKind,
        B: BenchFilter<K, M>,
    {
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

impl<M: Mangler> WithSpan for GroupBuilder<'_, M> {
    fn get_span(&self) -> Span {
        self.span
    }
}
