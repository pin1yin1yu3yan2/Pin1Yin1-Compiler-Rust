use super::*;
use std::collections::HashMap;
use terl::{Span, WithSpan};

/// used to represent a group of type, types in group may be declared
/// or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UndeclaredTy {
    pub(crate) idx: usize,
}

impl UndeclaredTy {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub(crate) span: Span,
    // this can only be init once, or keep empty to init later(UNKNOWN type)
    pub(crate) alive: HashMap<usize, Type>,
    pub(crate) faild: HashMap<usize, DeclareError>,
}

impl WithSpan for Group {
    fn get_span(&self) -> Span {
        self.span
    }
}

impl Group {
    pub fn new(
        span: Span,
        alive: HashMap<usize, Type>,
        removed: HashMap<usize, DeclareError>,
    ) -> Self {
        Self {
            span,
            alive,
            faild: removed,
        }
    }

    pub fn unique(&self) -> Option<&Type> {
        if self.alive.len() != 1 {
            return None;
        }
        self.alive.values().next()
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
