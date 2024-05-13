use crate::Type;
use std::rc::Rc;
use terl::{Message, Span};

#[derive(Debug, Clone)]
pub enum DeclareError {
    UniqueDeleted {
        reason: Box<DeclareError>,
    },
    NonBranchSelected {
        expect: String,
    },
    ConflictSelected {
        conflict_with: Type,
        this: Type,
    },
    Align {
        left: Span,
        right: Span,
    },
    WithLocation {
        location: Span,
        error: Box<DeclareError>,
    },
    WithPrevious {
        previous: Type,
        error: Box<DeclareError>,
    },
    Shared {
        err: Rc<DeclareError>,
    },

    //
    Declared {
        declare_as: Type,
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
            error: Box::new(self),
        }
    }

    pub fn into_shared(self) -> Self {
        Self::Shared { err: Rc::new(self) }
    }

    pub fn with_previous(self, previous: Type) -> Self {
        Self::WithPrevious {
            previous,
            error: Box::new(self),
        }
    }

    fn generate_inner(&self, msgs: &mut Vec<terl::Message>) {
        match self {
            DeclareError::UniqueDeleted { reason } => {
                msgs.push(Message::Text(
                    "this has been declared, but filtered latter".to_owned(),
                ));
                reason.generate_inner(msgs)
            }
            DeclareError::NonBranchSelected { expect } => {
                msgs.push(Message::Text(format!("this should be `{expect}`")))
            }
            DeclareError::ConflictSelected {
                conflict_with,
                this,
            } => msgs.push(Message::Text(format!(
                "this is required to been decalred as {} and {} together, but its impossiable",
                this, conflict_with
            ))),
            Self::Align { left, right } => {
                msgs.push("those two has been decalred to have same type".into());
                msgs.push((*left).into());
                msgs.push((*right).into());
            }
            DeclareError::Declared { declare_as } => msgs.push(Message::Text(format!(
                "this has been declared as {declare_as}"
            ))),
            DeclareError::Unexpect { expect } => msgs.push(Message::Text(format!(
                "expect this to be decalred as `{}`",
                expect.to_owned()
            ))),
            DeclareError::Filtered => msgs.push(Message::Text("this has been filtered".to_owned())),
            DeclareError::Shared { err } => err.generate_inner(msgs),

            DeclareError::WithLocation { location, error } => {
                let len = msgs.len();
                error.generate_inner(msgs);
                if let Message::Text(ref mut msg) = msgs[len] {
                    msgs[len] = Message::Rich(std::mem::take(msg), *location)
                }
            }
            DeclareError::WithPrevious { previous, error } => {
                error.generate_inner(msgs);
                msgs.push(Message::Text(format!(
                    "note: this used to be guessed as {previous}"
                )))
            }
            DeclareError::Empty => {}
        }
    }

    pub fn generate(&self) -> Vec<terl::Message> {
        let mut msgs = vec![];
        self.generate_inner(&mut msgs);
        msgs
    }
}
