mod bench;
mod filter;
mod group;
mod map;
pub use bench::*;
pub use filter::*;
pub use group::*;
pub use map::*;
use py_ir::ir::TypeDefine;

use super::{mangle::Mangler, DefineScope};

pub mod kind;

#[derive(Debug, Clone)]
pub enum Type {
    /// this's index can represent to a function's overload's return type. and
    /// the index is overloads index
    FnRetty(usize),
    Owned(TypeDefine),
}

impl Type {
    pub fn as_fn_retty(&self) -> usize {
        if let Self::FnRetty(v) = self {
            *v
        } else {
            panic!("how about using if let or match stmt first?")
        }
    }

    pub fn as_owned(&self) -> &TypeDefine {
        if let Self::Owned(v) = self {
            v
        } else {
            panic!("how about using if let or match stmt first?")
        }
    }

    pub fn as_type<'a, M: Mangler>(&'a self, defs: &'a DefineScope<M>) -> &'a TypeDefine {
        match self {
            Type::FnRetty(idx) => &defs.get_fn(*idx).ty,
            Type::Owned(ty) => ty,
        }
    }
}

impl From<TypeDefine> for Type {
    fn from(v: TypeDefine) -> Self {
        Self::Owned(v)
    }
}

impl From<py_ir::ir::ComplexType> for Type {
    fn from(v: py_ir::ir::ComplexType) -> Self {
        Self::Owned(v.into())
    }
}

impl From<py_ir::ir::PrimitiveType> for Type {
    fn from(v: py_ir::ir::PrimitiveType) -> Self {
        Self::Owned(v.into())
    }
}

impl From<usize> for Type {
    fn from(v: usize) -> Self {
        Self::FnRetty(v)
    }
}

#[cfg(test)]
mod tests {
    impl DeclareMap {
        fn test_declare<I>(&mut self, iter: I) -> GroupIdx
        where
            I: IntoIterator<Item = (Type, Vec<Bench>)>,
        {
            let declare_idx = GroupIdx::new(self.groups.len());

            let mut possiables = std::collections::HashMap::default();

            for (idx, (res, deps)) in iter.into_iter().enumerate() {
                possiables.insert(idx, BenchStatus::Available(res));
                let this_node = Bench::new(declare_idx, idx);

                self.deps.insert(this_node, deps.iter().copied().collect());
                self.rdeps.insert(this_node, Default::default());

                for dep in deps {
                    self.rdeps.get_mut(&dep).unwrap().insert(this_node);
                }
            }

            self.groups
                .push(DeclareGroup::new(terl::Span::new(0, 0), possiables));

            declare_idx
        }
    }

    use super::*;

    #[test]
    fn feature() {
        let mut map = DeclareMap::new();

        macro_rules! ty {
            ($idx:literal) => {
                Type::FnRetty { 0: $idx }
            };
        }

        // ty!(1-5) is used to emulate the type A-E
        //
        // m() -> A | B | C
        // b() -> B | C | D
        // p(A, B) -> C
        // p(B, C) -> D
        // p(C, D) -> E

        let m1 = map.test_declare([(ty!(1), vec![]), (ty!(2), vec![]), (ty!(3), vec![])]);
        let n1 = map.test_declare([(ty!(2), vec![]), (ty!(3), vec![]), (ty!(4), vec![])]);

        let i = map.test_declare([
            (ty!(3), vec![Bench::new(m1, 0), Bench::new(n1, 0)]),
            (ty!(4), vec![Bench::new(m1, 1), Bench::new(n1, 1)]),
            (ty!(5), vec![Bench::new(m1, 2), Bench::new(n1, 2)]),
        ]);

        let m2 = map.test_declare([(ty!(1), vec![]), (ty!(2), vec![]), (ty!(3), vec![])]);
        let n2 = map.test_declare([(ty!(2), vec![]), (ty!(3), vec![]), (ty!(4), vec![])]);

        let j = map.test_declare([
            (ty!(3), vec![Bench::new(m2, 0), Bench::new(n2, 0)]),
            (ty!(4), vec![Bench::new(m2, 1), Bench::new(n2, 1)]),
            (ty!(5), vec![Bench::new(m2, 2), Bench::new(n2, 2)]),
        ]);

        let k = map.test_declare([(ty!(5), vec![Bench::new(i, 0), Bench::new(j, 1)])]);
        map.make_sure(Bench::new(k, 0), terl::Message::Text("".to_owned()));

        for group in [m1, n1, i, m2, n2, j, k] {
            let bench_idx = *map.groups[group.idx].res.keys().next().unwrap();
            let bench = Bench::new(group, bench_idx);

            dbg!(&map.deps[&bench]);
        }
    }
}
