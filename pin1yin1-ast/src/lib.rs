/// the ast should be easy enough, even be a kind of IR
///
/// most of abstract will be transformed into basic operations
///
/// like method calls will be transformed into normal funcion calls
///
///
/// now, temp variables will be defined as ids(a number) in ast,
/// and variable with name will keep its name in ast
///
/// TODO: and a mangle rule will be applied
///
/// function overdrive depend on mangle rules, because its a kind of symbol
/// i mean that, we should not use `foo1` as `foo`'s overdrive's name(
pub mod ast;
pub mod keywords;
pub mod macros;
