use super::*;
use std::{cell::LazyCell, collections::HashMap};
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
        Self::from_iter(iter.into_iter().enumerate())
    }
}

impl FromIterator<(usize, Type)> for DeclareState {
    fn from_iter<T: IntoIterator<Item = (usize, Type)>>(iter: T) -> Self {
        let mut iter = iter.into_iter().peekable();
        let Some(next) = iter.next() else {
            return Self::Empty;
        };
        if iter.peek().is_some() {
            Self::Declaring(std::iter::once(next).chain(iter).collect())
        } else {
            Self::Declared(next.0, next.1)
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

    pub fn push_error(&mut self, at: usize, error: DeclareError) {
        self.failds.insert(at, error);
    }

    fn update_state<U>(&mut self, updater: U)
    where
        U: FnOnce(DeclareState) -> DeclareState,
    {
        let previous = std::mem::replace(&mut self.status, DeclareState::Empty);
        self.status = updater(previous);
    }

    pub fn filter_alive<F, C>(&mut self, mut filter: F) -> C
    where
        F: FnMut(Branch, Type) -> Result<(Branch, Type), DeclareError>,
        C: Default + Extend<(Branch, DeclareError)>,
    {
        let mut remove = C::default();
        let belong_to = self.group;
        let filter_map = |(branch, ty)| {
            let branch = Branch::new(belong_to, branch);
            filter(branch, ty)
                .map_err(|e| remove.extend(Some((branch, e))))
                .map(|(branch, ty)| (branch.branch_idx, ty))
                .ok()
        };

        self.update_state(|state| match state {
            DeclareState::Empty => DeclareState::Empty,
            DeclareState::Declared(branch, ty) => {
                let iter = std::iter::once((branch, ty)).filter_map(filter_map);
                DeclareState::from_iter(iter)
            }
            DeclareState::Declaring(branches) => {
                let iter = branches.into_iter().filter_map(filter_map);
                DeclareState::from_iter(iter)
            }
        });
        remove
    }

    /// remove only one branch
    ///
    /// note: this method will do nothing if the branch is not exist(including have been remvoed)
    pub fn remove_branch(&mut self, branch: usize, reason: DeclareError) -> DeclareError {
        let mut new_reason = DeclareError::Empty;
        self.update_state(|state| match state {
            DeclareState::Declared(unique, previous) => {
                if unique == branch {
                    new_reason = reason.with_previous(previous);
                    DeclareState::Empty
                } else {
                    DeclareState::Declared(unique, previous)
                }
            }
            DeclareState::Declaring(mut items) => {
                let previous = items.remove(&branch);
                new_reason = reason.with_previous(previous.unwrap_or_else(|| unreachable!()));
                items.into()
            }
            _ => unreachable!(),
        });
        new_reason
    }

    /// # Note
    ///
    /// the [`DeclareError`] generated by `reason` will be clone many times, so, you should
    /// call [`DeclareError::into_shared`] to wrapped it in [`Rc`]
    ///
    /// and, [`DeclareError::with_previous`] will be called on [`DeclareError`]
    ///
    /// [`Rc`]: std::rc
    pub fn remove_branches<F, R>(&mut self, remove_if: F, reason: R) -> Vec<(Branch, DeclareError)>
    where
        F: Fn(Branch, &Type) -> bool,
        R: FnOnce() -> DeclareError,
    {
        let reason = LazyCell::new(reason);

        self.filter_alive(move |branch, ty| {
            if remove_if(branch, &ty) {
                Err((*reason).clone().with_previous(ty))
            } else {
                Ok((branch, ty))
            }
        })
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
            DeclareState::Empty => err += "this cant be declared as any type!",
            DeclareState::Declaring(alives) => {
                err += "this cant be declared as:";
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
    pub(crate) branches: Vec<BranchesBuilder>,
}

impl GroupBuilder {
    pub fn new(span: Span, branches: Vec<BranchesBuilder>) -> GroupBuilder {
        GroupBuilder { span, branches }
    }
}

impl WithSpan for GroupBuilder {
    fn get_span(&self) -> Span {
        self.span
    }
}
