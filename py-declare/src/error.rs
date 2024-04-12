use std::{collections::HashMap, rc::Rc};

use crate::{Bench, Type};

#[derive(Debug, Clone)]
pub enum DeclareError {
    UniqueDeleted {
        previous: Type,
        reason: Box<DeclareError>,
    },
    NonBenchSelected {
        expect: String,
    },
    MultSelected {
        expect: String,
        selected: HashMap<Bench, Type>,
    },
    ConflictSelected {
        conflict_with: Type,
    },
    Unexpect {
        expect: String,
    },
    GroupSolved {
        decalre_as: Type,
    },
    TypeUnmatch {
        previous: Type,
    },
    RemovedDuoDeclared {
        reason: Box<DeclareError>,
    },
    ReasonDeclared {
        declare_as: Rc<Type>,
    },
    Empty,
}
