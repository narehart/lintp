use anyhow::{Context as ErrorContext, Result};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::dsl::ast::{BinaryOperator, Expression, StringTemplatePart, UnaryOperator};
use crate::dsl::functions;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
    Regex(Regex),
    List(Vec<Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Regex(r) => write!(f, "/{}/", r.as_str()),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
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

pub struct EvaluationContext<'a> {
    pub variables: HashMap<String, Value>,
    pub path: &'a Path,
    pub custom_matchers: &'a HashMap<String, Expression>,
    pub item_context: Option<Value>, // For map/filter operations
    pub fs_cache: Option<&'a FsCache>,
    pub regex_cache: Option<&'a RegexCache>,
}

pub fn evaluate(expr: &Expression, context: &EvaluationContext) -> Result<Value> {
    match expr {
        Expression::Variable(name) => {
            if let Some(value) = context.variables.get(name) {
                Ok(value.clone())
            } else if name == "item" {
                if let Some(item) = context.item_context.as_ref() {
                    Ok(item.clone())
                } else {
                    Err(anyhow::anyhow!("Unknown variable: {}", name))
                }
            } else {
                Err(anyhow::anyhow!("Unknown variable: {}", name))
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

            let regex = Regex::new(pattern)
                .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

            if let Some(cache) = context.regex_cache {
                cache
                    .borrow_mut()
                    .insert(pattern.clone(), regex.clone());
            }

            Ok(Value::Regex(regex))
        }

        Expression::ListLiteral(items) => {
            let mut values = Vec::new();

            for item in items {
                let value = evaluate(item, context)?;
                values.push(value);
            }

            Ok(Value::List(values))
        }

        Expression::BinaryOp { op, left, right } => {
            let left_value = evaluate(left, context)?;

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

            let right_value = evaluate(right, context)?;

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
            let value = evaluate(expr, context)?;

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
                let collection = evaluate(&args[0], context)?;
                return functions::call_lambda_function(name, &collection, &args[1], context);
            }

            let mut arg_values = Vec::new();

            for arg in args {
                let value = evaluate(arg, context)?;
                arg_values.push(value);
            }

            functions::call_function(name, &arg_values, context)
        }

        Expression::Reference(name) => {
            if let Some(expr) = context.custom_matchers.get(name) {
                evaluate(expr, context)
            } else {
                Err(anyhow::anyhow!("Unknown reference: {}", name))
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
                        let value = evaluate(expr, context)?;
                        result.push_str(&value.to_string());
                    }
                }
            }

            Ok(Value::String(result))
        }

        Expression::Index { expr, index } => {
            let expr_value = evaluate(expr, context)?;
            let index_value = evaluate(index, context)?;
            let expr_clone = expr_value.clone();
            let index_clone = index_value.clone();

            match (expr_value, index_value) {
                (Value::List(items), Value::Integer(i)) => {
                    if i < 0 || (i as usize) >= items.len() {
                        Err(anyhow::anyhow!(
                            "Index out of bounds: {} for list of length {}",
                            i,
                            items.len()
                        ))
                    } else {
                        Ok(items[i as usize].clone())
                    }
                }
                (Value::String(s), Value::Integer(i)) => {
                    let chars: Vec<char> = s.chars().collect();
                    if i < 0 || (i as usize) >= chars.len() {
                        Err(anyhow::anyhow!(
                            "Index out of bounds: {} for string of length {}",
                            i,
                            chars.len()
                        ))
                    } else {
                        Ok(Value::String(chars[i as usize].to_string()))
                    }
                }
                _ => Err(anyhow::anyhow!(
                    "Cannot index into {:?} with {:?}",
                    expr_clone,
                    index_clone
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
