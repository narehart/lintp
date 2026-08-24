//! The rule expression language: parsing, evaluation, and built-ins.

/// The parsed shape of an expression.
pub mod ast;
/// Walking a parsed expression against one file.
pub mod evaluator;
/// The built-in functions rules can call.
pub mod functions;
/// Turning rule source text into an [`Expression`](crate::dsl::ast::Expression).
pub mod parser;
