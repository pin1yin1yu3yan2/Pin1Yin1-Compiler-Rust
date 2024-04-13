use crate::Type;
use std::rc::Rc;
use terl::Span;

#[derive(Debug, Clone)]
pub enum DeclareError {
    UniqueDeleted {
        reason: Box<DeclareError>,
    },
    NonBenchSelected {
        expect: String,
    },
    ConflictSelected {
        conflict_with: Rc<Type>,
    },
    GroupSolved {
        decalre_as: Type,
    },

    WithLocation {
        location: Span,
        err: Box<DeclareError>,
    },
    WithPrevious {
        previous: Rc<Type>,
        error: Box<DeclareError>,
    },
    Shared {
        err: Rc<DeclareError>,
    },

    RemovedDuoDeclared {
        reason: Box<DeclareError>,
    },

    //
    Declared {
        declare_as: Rc<Type>,
    },
    Unexpect {
        expect: String,
    },

    Filtered,

    Empty,
}

impl DeclareError {
    pub fn with_location(self, location: Span) -> Self {
        Self::WithLocation {
            location,
            err: Box::new(self),
        }
    }

    pub fn into_shared(self) -> Self {
        Self::Shared { err: Rc::new(self) }
    }

    pub fn with_previous(self, previous: Rc<Type>) -> Self {
        Self::WithPrevious {
            previous,
            error: Box::new(self),
        }
    }
}
