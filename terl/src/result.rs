use crate::*;

/// A type alias for a generic parse result.
///
/// This type alias represents the result of a parsing operation, where the
/// `P` type is the parse unit and `S` is the source input. The result is
/// either the parsed target or a parse error.
///
/// # See Also
///
/// - [`ParseError`]: The type of error that can be returned by a parse operation.
/// - [`ParseUnit`]: The trait that defines the parsing behavior for a parse unit.
pub type ParseResult<P, S> = Result<<P as ParseUnit<S>>::Target, ParseError>;

/// A type alias for a generic result type.
///
/// This type alias represents the result of a generic operation, where the
/// `T` type is the result value and `E` is the error type. If no error type is
/// specified, it defaults to `Error`.
///
/// # See Also
///
/// - [`Error`]: The default error type used when no error type is specified.
pub type Result<T, E = Error> = std::result::Result<T, E>;
