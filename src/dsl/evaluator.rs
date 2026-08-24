use anyhow::{Context as ErrorContext, Result};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::dsl::ast::{BinaryOperator, Expression, StringTemplatePart, UnaryOperator};
use crate::dsl::functions;

/// A value produced while evaluating an expression.
///
/// The DSL is dynamically typed: functions check the variants they were
/// handed and report a type error rather than coercing.
#[derive(Debug, Clone)]
pub enum Value {
    /// Text — every built-in variable evaluates to one of these.
    String(String),
    /// A whole number, from a literal or from `count()`.
    Integer(i64),
    /// The result of a comparison, a matcher, or a whole rule.
    Boolean(bool),
    /// A compiled `/pattern/`, only ever an argument to `matches()`.
    Regex(Regex),
    /// The result of `siblings()`, `children()`, `find()`, `map()`, `filter()`.
    List(Vec<Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Regex(r) => write!(f, "/{}/", r.as_str()),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::List(a), Value::List(b)) => a == b,
            _ => false,
        }
    }
}

/// Cache of glob results shared across a lint run so the collection
/// functions (siblings/children/exists/find) don't re-read the same
/// directory for every file they are evaluated against.
pub type FsCache = RefCell<HashMap<String, Vec<PathBuf>>>;

/// Cache of compiled regexes shared across a lint run so a `/pattern/`
/// literal used in a rule is compiled once, not once per file evaluated
/// against that rule.
pub type RegexCache = RefCell<HashMap<String, Regex>>;

/// Everything an expression can see while it is being evaluated against one
/// file or directory.
pub struct EvaluationContext<'a> {
    /// The built-in variables (`NAME`, `EXT`, `PATH`, ...) keyed without the
    /// leading `$`.
    pub variables: HashMap<String, Value>,
    /// The path currently being checked; the anchor for `siblings()`,
    /// `children()` and `exists()`.
    pub path: &'a Path,
    /// Named expressions from `custom-matchers`, resolved when an
    /// [`Expression::Reference`] is evaluated.
    pub custom_matchers: &'a HashMap<String, Expression>,
    /// The value bound to `$item` inside `any()`, `all()`, `map()` and
    /// `filter()`; `None` outside a lambda.
    pub item_context: Option<Value>,
    /// Directory listings shared across the run, so a `siblings()` rule reads
    /// each directory once rather than once per file in it.
    pub fs_cache: Option<&'a FsCache>,
    /// Compiled regexes shared across the run, so a pattern is compiled once
    /// rather than once per file it is tested against.
    pub regex_cache: Option<&'a RegexCache>,
}

/// Evaluates a parsed rule or custom-matcher expression against `context`.
///
/// # Errors
///
/// Returns [`crate::Error::Dsl`] if the expression references an unknown
/// variable, matcher, or custom-matcher reference; calls a built-in
/// function with the wrong argument count or type; or otherwise fails to
/// evaluate to the expected type (e.g. a non-boolean operand to `&&`).
pub fn evaluate(
    expr: &Expression,
    context: &EvaluationContext,
) -> std::result::Result<Value, crate::Error> {
    evaluate_impl(expr, context).map_err(|e| crate::Error::Dsl(format!("{e:#}")))
}

/// Implementation behind [`evaluate`]; kept separate (and anyhow-based)
/// because it recurses into itself and into `dsl::functions`, where the
/// surrounding `anyhow::Context` chaining is more convenient than
/// converting back and forth through [`crate::Error`] on every call.
pub(crate) fn evaluate_impl(expr: &Expression, context: &EvaluationContext) -> Result<Value> {
    match expr {
        Expression::Variable(name) => {
            if let Some(value) = context.variables.get(name) {
                Ok(value.clone())
            } else if name == "item" {
                if let Some(item) = context.item_context.as_ref() {
                    Ok(item.clone())
                } else {
                    Err(anyhow::anyhow!("Unknown variable: {name}"))
                }
            } else {
                Err(anyhow::anyhow!("Unknown variable: {name}"))
            }
        }

        Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
        Expression::IntegerLiteral(i) => Ok(Value::Integer(*i)),
        Expression::BooleanLiteral(b) => Ok(Value::Boolean(*b)),

        Expression::RegexLiteral(pattern) => {
            // Regexes are compiled once per distinct pattern and reused for
            // every file evaluated against a rule, instead of recompiling
            // the same pattern on every single evaluation.
            if let Some(cache) = context.regex_cache {
                if let Some(regex) = cache.borrow().get(pattern) {
                    return Ok(Value::Regex(regex.clone()));
                }
            }

            let regex =
                Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {pattern}"))?;

            if let Some(cache) = context.regex_cache {
                cache.borrow_mut().insert(pattern.clone(), regex.clone());
            }

            Ok(Value::Regex(regex))
        }

        Expression::ListLiteral(items) => {
            let mut values = Vec::new();

            for item in items {
                let value = evaluate_impl(item, context)?;
                values.push(value);
            }

            Ok(Value::List(values))
        }

        Expression::BinaryOp { op, left, right } => {
            let left_value = evaluate_impl(left, context)?;

            // Short-circuit evaluation for logical operators
            match op {
                BinaryOperator::And => {
                    if let Value::Boolean(false) = left_value {
                        return Ok(Value::Boolean(false));
                    }
                }
                BinaryOperator::Or => {
                    if let Value::Boolean(true) = left_value {
                        return Ok(Value::Boolean(true));
                    }
                }
                _ => {}
            }

            let right_value = evaluate_impl(right, context)?;

            match op {
                BinaryOperator::And => match (left_value, right_value) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l && r)),
                    _ => Err(anyhow::anyhow!("AND operator requires boolean operands")),
                },
                BinaryOperator::Or => match (left_value, right_value) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l || r)),
                    _ => Err(anyhow::anyhow!("OR operator requires boolean operands")),
                },
                BinaryOperator::Equal => Ok(Value::Boolean(left_value == right_value)),
                BinaryOperator::NotEqual => Ok(Value::Boolean(left_value != right_value)),
                BinaryOperator::LessThan => match (left_value, right_value) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l < r)),
                    _ => Err(anyhow::anyhow!(
                        "Less than operator requires integer or string operands"
                    )),
                },
                BinaryOperator::GreaterThan => match (left_value, right_value) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l > r)),
                    _ => Err(anyhow::anyhow!(
                        "Greater than operator requires integer or string operands"
                    )),
                },
                BinaryOperator::LessThanOrEqual => match (left_value, right_value) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l <= r)),
                    _ => Err(anyhow::anyhow!(
                        "Less than or equal operator requires integer or string operands"
                    )),
                },
                BinaryOperator::GreaterThanOrEqual => match (left_value, right_value) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l >= r)),
                    _ => Err(anyhow::anyhow!(
                        "Greater than or equal operator requires integer or string operands"
                    )),
                },
            }
        }

        Expression::UnaryOp { op, expr } => {
            let value = evaluate_impl(expr, context)?;

            match op {
                UnaryOperator::Not => match value {
                    Value::Boolean(b) => Ok(Value::Boolean(!b)),
                    _ => Err(anyhow::anyhow!("NOT operator requires a boolean operand")),
                },
                UnaryOperator::Minus => match value {
                    Value::Integer(i) => Ok(Value::Integer(-i)),
                    _ => Err(anyhow::anyhow!(
                        "Minus operator requires an integer operand"
                    )),
                },
            }
        }

        Expression::FunctionCall { name, args } => {
            // The collection functions take a lambda as their second argument.
            // It must NOT be evaluated eagerly: `$item` is only bound while
            // iterating, so the lambda is passed through as an expression.
            if matches!(name.as_str(), "any" | "all" | "map" | "filter") && args.len() == 2 {
                let collection = evaluate_impl(&args[0], context)?;
                return functions::call_lambda_function_impl(name, &collection, &args[1], context);
            }

            let mut arg_values = Vec::new();

            for arg in args {
                let value = evaluate_impl(arg, context)?;
                arg_values.push(value);
            }

            functions::call_function_impl(name, &arg_values, context)
        }

        Expression::Reference(name) => {
            if let Some(expr) = context.custom_matchers.get(name) {
                evaluate_impl(expr, context)
            } else {
                Err(anyhow::anyhow!("Unknown reference: {name}"))
            }
        }

        Expression::StringTemplate(parts) => {
            let mut result = String::new();

            for part in parts {
                match part {
                    StringTemplatePart::Literal(s) => {
                        result.push_str(s);
                    }
                    StringTemplatePart::Expression(expr) => {
                        let value = evaluate_impl(expr, context)?;
                        result.push_str(&value.to_string());
                    }
                }
            }

            Ok(Value::String(result))
        }

        Expression::Index { expr, index } => {
            let expr_value = evaluate_impl(expr, context)?;
            let index_value = evaluate_impl(index, context)?;
            let expr_clone = expr_value.clone();
            let index_clone = index_value.clone();

            match (expr_value, index_value) {
                (Value::List(items), Value::Integer(i)) => {
                    // try_from rejects negatives outright, so a negative
                    // index can never wrap into a huge usize and index past
                    // the end of the list
                    match usize::try_from(i).ok().and_then(|i| items.get(i)) {
                        Some(item) => Ok(item.clone()),
                        None => Err(anyhow::anyhow!(
                            "Index out of bounds: {} for list of length {}",
                            i,
                            items.len()
                        )),
                    }
                }
                (Value::String(s), Value::Integer(i)) => {
                    let chars: Vec<char> = s.chars().collect();
                    match usize::try_from(i).ok().and_then(|i| chars.get(i)) {
                        Some(c) => Ok(Value::String(c.to_string())),
                        None => Err(anyhow::anyhow!(
                            "Index out of bounds: {} for string of length {}",
                            i,
                            chars.len()
                        )),
                    }
                }
                _ => Err(anyhow::anyhow!(
                    "Cannot index into {expr_clone:?} with {index_clone:?}"
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::ast::Expression;
    use std::path::Path;

    /// Regression test: a `/pattern/` regex literal evaluated repeatedly
    /// (as happens once per file linted with the same rule) must compile
    /// the pattern once and reuse it from `regex_cache`, not recompile it
    /// on every call.
    #[test]
    fn test_regex_literal_uses_and_populates_cache() {
        let path = Path::new("/tmp/test.js");
        let custom_matchers = HashMap::new();
        let regex_cache: RegexCache = RegexCache::default();

        let context = EvaluationContext {
            variables: HashMap::new(),
            path,
            custom_matchers: &custom_matchers,
            item_context: None,
            fs_cache: None,
            regex_cache: Some(&regex_cache),
        };

        let expr = Expression::RegexLiteral("^test-[0-9]+$".to_string());

        // Cache starts empty
        assert_eq!(regex_cache.borrow().len(), 0);

        let first = evaluate(&expr, &context).unwrap();
        assert!(matches!(first, Value::Regex(_)));
        assert_eq!(regex_cache.borrow().len(), 1);

        // Evaluating the same pattern again must reuse the cached entry
        // rather than inserting a second one.
        let second = evaluate(&expr, &context).unwrap();
        assert!(matches!(second, Value::Regex(_)));
        assert_eq!(regex_cache.borrow().len(), 1);
    }
}
