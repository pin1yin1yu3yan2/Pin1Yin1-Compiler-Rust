use super::*;
use std::collections::HashMap;
use terl::{Span, WithSpan};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupIdx {
    pub(crate) idx: usize,
}

impl GroupIdx {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Debug, Clone)]
pub enum DeclareState {
    Empty,
    Declared(usize, Type),
    Declaring(HashMap<usize, Type>),
}

impl DeclareState {
    pub fn alives<'t, A, T: 't>(&'t self, gidx: GroupIdx, active: A) -> T
    where
        A: FnOnce(&mut dyn Iterator<Item = (Branch, &'t Type)>) -> T,
    {
        let alives: &mut dyn Iterator<Item = (Branch, &Type)> = match self {
            DeclareState::Empty => &mut std::iter::empty(),
            DeclareState::Declared(branch, ty) => {
                &mut std::iter::once((Branch::new(gidx, *branch), ty))
            }
            DeclareState::Declaring(alives) => &mut alives
                .iter()
                .map(|(branch, ty)| (Branch::new(gidx, *branch), ty)),
        };
        active(alives)
    }
}

impl From<(usize, Type)> for DeclareState {
    fn from((v1, v2): (usize, Type)) -> Self {
        Self::Declared(v1, v2)
    }
}

impl From<HashMap<usize, Type>> for DeclareState {
    fn from(v: HashMap<usize, Type>) -> Self {
        match v.len() {
            0 => Self::Empty,
            1 => {
                let Some((branch, unique)) = v.into_iter().next() else {
                    unreachable!()
                };
                Self::Declared(branch, unique)
            }
            _ => Self::Declaring(v),
        }
    }
}

impl FromIterator<Type> for DeclareState {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        let mut iter = iter.into_iter().peekable();
        let Some(next) = iter.next() else {
            return Self::Empty;
        };
        if iter.peek().is_some() {
            Self::Declaring(std::iter::once(next).chain(iter).enumerate().collect())
        } else {
            Self::Declared(0, next)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclareGroup {
    span: Span,
    group: GroupIdx,
    failds: HashMap<usize, DeclareError>,
    status: DeclareState,
}

impl DeclareGroup {
    pub fn new(
        span: Span,
        group: GroupIdx,
        fails: HashMap<usize, DeclareError>,
        status: DeclareState,
    ) -> Self {
        Self {
            span,
            group,
            failds: fails,
            status,
        }
    }

    pub fn new_error(&mut self, at: usize, error: DeclareError) {
        self.failds.insert(at, error);
    }

    pub fn update_state<U>(&mut self, updater: U)
    where
        U: FnOnce(&mut Self, DeclareState) -> DeclareState,
    {
        let previous = std::mem::replace(&mut self.status, DeclareState::Empty);
        self.status = updater(self, previous);
    }

    pub fn delete_branches<F, R>(&mut self, delete_if: F, reason: R) -> Vec<(Branch, DeclareError)>
    where
        F: Fn(Branch, &Type) -> bool,
        R: FnOnce() -> DeclareError,
    {
        let mut delete = vec![];
        self.update_state(|group, state| match state {
            DeclareState::Empty => DeclareState::Empty,
            DeclareState::Declared(branch, ty) => {
                let branch = Branch::new(group.group, branch);
                if delete_if(branch, &ty) {
                    let reason = DeclareError::UniqueDeleted {
                        reason: Box::new(reason()),
                    }
                    .with_previous(ty);

                    delete.push((branch, reason));
                    DeclareState::Empty
                } else {
                    DeclareState::Declared(branch.branch_idx, ty)
                }
            }
            DeclareState::Declaring(alive) => {
                let reason = reason();
                alive
                    .into_iter()
                    .filter_map(|(branch, ty)| {
                        let branch = Branch::new(group.group, branch);
                        if delete_if(branch, &ty) {
                            let reason = reason.clone().with_previous(ty);
                            delete.push((branch, reason));
                            None
                        } else {
                            Some((branch.branch_idx, ty))
                        }
                    })
                    .collect::<HashMap<_, _>>()
                    .into()
            }
        });

        delete
    }

    pub fn alives<'t, A, T: 't>(&'t self, active: A) -> T
    where
        A: FnOnce(&mut dyn Iterator<Item = (Branch, &'t Type)>) -> T,
    {
        self.status.alives(self.group, active)
    }

    pub fn is_declared(&self) -> bool {
        matches!(self.status, DeclareState::Declared(..))
    }

    /// return declare result
    ///
    /// # Panic
    ///
    /// panic if the group it not declared
    pub fn result(&self) -> &Type {
        match &self.status {
            DeclareState::Declared(_, ty) => ty,
            _ => panic!("group is not declared yet"),
        }
    }

    pub fn make_error(&self) -> terl::Error {
        let mut err = <Self as terl::WithSpan>::make_error(self, "cant infer type");
        match &self.status {
            DeclareState::Empty => err += "this cant be decalred as any type!",
            DeclareState::Declaring(alives) => {
                err += "this cant be decalred as:";
                for alives in alives.values() {
                    err += format!("\t{alives}")
                }
            }
            DeclareState::Declared(_, _) => unreachable!(),
        }

        err.extend(self.failds.values().flat_map(|faild| faild.generate()));
        err
    }

    /// # Panic
    ///
    /// panic if the branch is not exist, faild, or isnot belong to this group
    pub fn get_branch(&self, branch: Branch) -> &Type {
        debug_assert_eq!(
            branch.belong_to, self.group,
            "the branch is not belong to this group"
        );

        let branch = branch.branch_idx;
        match &self.status {
            DeclareState::Declared(idx, ty) if *idx == branch => ty,
            DeclareState::Declaring(alives) if alives.contains_key(&branch) => &alives[&branch],
            _ => panic!("the branch isnot exist, or faild"),
        }
    }
}

impl WithSpan for DeclareGroup {
    fn get_span(&self) -> Span {
        self.span
    }
}

pub struct GroupBuilder {
    pub(crate) span: Span,
    pub(crate) builders: Vec<BranchBuilder>,
}

impl GroupBuilder {
    pub fn new(span: Span, builders: Vec<BranchBuilder>) -> GroupBuilder {
        GroupBuilder { span, builders }
    }
}

impl WithSpan for GroupBuilder {
    fn get_span(&self) -> Span {
        self.span
    }
}
