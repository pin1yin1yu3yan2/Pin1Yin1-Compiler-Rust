mod branch;
mod error;
mod filter;
mod group;
mod map;
mod res;
pub use branch::*;
pub use error::*;
pub use filter::*;
pub use group::*;
pub use map::*;
pub use res::*;

pub mod defs;
pub mod mir;
pub use defs::Defs;

type Result<T, E = DeclareError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn feature() {
        // ty!(1-5) is used to emulate the type A-E
        //
        // m() -> A | B | C
        // b() -> B | C | D
        // p(A, B) -> C
        // p(B, C) -> D
        // p(C, D) -> E

        let no_span = terl::Span::new(0, 0);

        use py_ir::{ComplexType, TypeDefine};
        use py_lex::SharedString;

        let raw_types = (0..6)
            .map(|idx| {
                TypeDefine::from(ComplexType::no_decorators(SharedString::from(format!(
                    "t{idx}"
                ))))
            })
            .collect::<Vec<_>>();
        let types = raw_types
            .iter()
            .map(|raw| Type::from(raw.clone()))
            .collect::<Vec<_>>();
        let filters = raw_types
            .iter()
            .map(|raw| filters::TypeEqual::new(raw, no_span))
            .collect::<Vec<_>>();

        let mut map = DeclareMap::new();

        let m1 = map.build_group(GroupBuilder::new(
            no_span,
            vec![
                types[1].clone().into(),
                types[2].clone().into(),
                types[3].clone().into(),
            ],
        ));
        let n1 = map.build_group(GroupBuilder::new(
            no_span,
            vec![
                types[2].clone().into(),
                types[3].clone().into(),
                types[4].clone().into(),
            ],
        ));

        let i = {
            let gb = GroupBuilder::new(
                no_span,
                vec![
                    BranchBuilder::new(types[3].clone())
                        .new_depend::<Directly, _>(&mut map, m1, &filters[1])
                        .new_depend::<Directly, _>(&mut map, n1, &filters[2]),
                    BranchBuilder::new(types[4].clone())
                        .new_depend::<Directly, _>(&mut map, m1, &filters[2])
                        .new_depend::<Directly, _>(&mut map, n1, &filters[3]),
                    BranchBuilder::new(types[5].clone())
                        .new_depend::<Directly, _>(&mut map, m1, &filters[3])
                        .new_depend::<Directly, _>(&mut map, n1, &filters[4]),
                ],
            );
            map.build_group(gb)
        };

        let m2 = map.build_group(GroupBuilder::new(
            no_span,
            vec![
                types[1].clone().into(),
                types[2].clone().into(),
                types[3].clone().into(),
            ],
        ));
        let n2 = map.build_group(GroupBuilder::new(
            no_span,
            vec![
                types[2].clone().into(),
                types[3].clone().into(),
                types[4].clone().into(),
            ],
        ));
        let j = {
            let gb = GroupBuilder::new(
                no_span,
                vec![
                    BranchBuilder::new(types[3].clone())
                        .new_depend::<Directly, _>(&mut map, m2, &filters[1])
                        .new_depend::<Directly, _>(&mut map, n2, &filters[2]),
                    BranchBuilder::new(types[4].clone())
                        .new_depend::<Directly, _>(&mut map, m2, &filters[2])
                        .new_depend::<Directly, _>(&mut map, n2, &filters[3]),
                    BranchBuilder::new(types[5].clone())
                        .new_depend::<Directly, _>(&mut map, m2, &filters[3])
                        .new_depend::<Directly, _>(&mut map, n2, &filters[4]),
                ],
            );
            map.build_group(gb)
        };

        let _k = {
            let gb = GroupBuilder::new(
                no_span,
                vec![BranchBuilder::new(types[5].clone())
                    .new_depend::<Directly, _>(&mut map, i, &filters[4])
                    .new_depend::<Directly, _>(&mut map, j, &filters[5])],
            );
            map.build_group(gb)
        };

        // map.make_sure(Branch::new(k, 0), DeclareError::Empty);

        assert!(map.declare_all().is_ok());
    }
}
