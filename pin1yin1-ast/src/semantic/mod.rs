pub mod definition;
pub mod definition_pool;

pub fn check<'s>(stmts: impl IntoIterator<Item = crate::ast::Statement<'s>>) {
    let stmts = stmts
        .into_iter()
        .map(definition::Statement::from)
        .collect::<Vec<_>>();
    dbg!(stmts);
}
