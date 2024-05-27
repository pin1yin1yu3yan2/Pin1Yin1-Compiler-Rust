mod branch;
mod error;
mod filter;
mod graph;
mod group;
mod res;
pub use branch::*;
pub use error::*;
pub use filter::*;
pub use graph::*;
pub use group::*;
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
        let ordered_span = |idx: usize| terl::Span::new(idx, idx + 1);

        use py_ir::types::{ComplexType, TypeDefine};

        let raw_types = (0..6)
            .map(|idx| TypeDefine::from(ComplexType::no_decorators(format!("t{idx}"))))
            .collect::<Vec<_>>();
        let types = raw_types
            .iter()
            .map(|raw| Type::from(raw.clone()))
            .collect::<Vec<_>>();
        let filters = raw_types
            .iter()
            .map(|raw| filters::TypeEqual::new(raw, no_span))
            .collect::<Vec<_>>();

        let mut map = DeclareGraph::new();
        let defs = Defs::new();

        let m1 = map.build_group(GroupBuilder::new(
            ordered_span(1),
            vec![
                types[1].clone().into(),
                types[2].clone().into(),
                types[3].clone().into(),
            ],
        ));
        let n1 = map.build_group(GroupBuilder::new(
            ordered_span(2),
            vec![
                types[2].clone().into(),
                types[3].clone().into(),
                types[4].clone().into(),
            ],
        ));

        let i = {
            let gb = GroupBuilder::new(
                ordered_span(11),
                vec![
                    BranchesBuilder::new(types[3].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m1, &filters[1])
                        .new_depend::<Directly, _>(&mut map, &defs, n1, &filters[2]),
                    BranchesBuilder::new(types[4].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m1, &filters[2])
                        .new_depend::<Directly, _>(&mut map, &defs, n1, &filters[3]),
                    BranchesBuilder::new(types[5].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m1, &filters[3])
                        .new_depend::<Directly, _>(&mut map, &defs, n1, &filters[4]),
                ],
            );

            map.build_group(gb)
        };

        let m2 = map.build_group(GroupBuilder::new(
            ordered_span(3),
            vec![
                types[1].clone().into(),
                types[2].clone().into(),
                types[3].clone().into(),
            ],
        ));
        let n2 = map.build_group(GroupBuilder::new(
            ordered_span(4),
            vec![
                types[2].clone().into(),
                types[3].clone().into(),
                types[4].clone().into(),
            ],
        ));
        let j = {
            let gb = GroupBuilder::new(
                ordered_span(12),
                vec![
                    BranchesBuilder::new(types[3].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m2, &filters[1])
                        .new_depend::<Directly, _>(&mut map, &defs, n2, &filters[2]),
                    BranchesBuilder::new(types[4].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m2, &filters[2])
                        .new_depend::<Directly, _>(&mut map, &defs, n2, &filters[3]),
                    BranchesBuilder::new(types[5].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, m2, &filters[3])
                        .new_depend::<Directly, _>(&mut map, &defs, n2, &filters[4]),
                ],
            );
            map.build_group(gb)
        };

        let _k = {
            let gb = GroupBuilder::new(
                ordered_span(21),
                vec![
                    BranchesBuilder::new(types[3].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, i, &filters[1])
                        .new_depend::<Directly, _>(&mut map, &defs, j, &filters[2]),
                    BranchesBuilder::new(types[4].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, i, &filters[2])
                        .new_depend::<Directly, _>(&mut map, &defs, j, &filters[3]),
                    BranchesBuilder::new(types[5].clone())
                        .new_depend::<Directly, _>(&mut map, &defs, i, &filters[3])
                        .new_depend::<Directly, _>(&mut map, &defs, j, &filters[4]),
                ],
            );
            map.build_group(gb)
        };

        // map.make_sure(Branch::new(k, 0), DeclareError::Empty);

        assert!(map.declare_all().is_ok());
    }
}
