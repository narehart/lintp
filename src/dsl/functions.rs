use anyhow::{Context, Result};
use glob::Pattern;
use std::path::{Path, PathBuf};

use crate::dsl::ast::Expression;
use crate::dsl::evaluator::{EvaluationContext, Value};
use crate::dsl::parser::parse_expression_impl;
use crate::util::forward_slashes;

/// Entry point for the collection functions (any/all/map/filter), whose
/// lambda argument arrives unevaluated so `$item` can be bound per element.
///
/// # Errors
///
/// Returns [`crate::Error::Dsl`] if `collection` is not a list, `name` is
/// not a recognized lambda function, or the lambda fails to parse or
/// evaluate for one of the collection's items.
pub fn call_lambda_function(
    name: &str,
    collection: &Value,
    lambda: &Expression,
    context: &EvaluationContext,
) -> std::result::Result<Value, crate::Error> {
    call_lambda_function_impl(name, collection, lambda, context)
        .map_err(|e| crate::Error::Dsl(format!("{:#}", e)))
}

/// Implementation behind [`call_lambda_function`]; kept separate (and
/// anyhow-based) because it's mutually recursive with `dsl::evaluator`,
/// where the surrounding `anyhow::Context` chaining is more convenient than
/// converting back and forth through [`crate::Error`] on every call.
pub(crate) fn call_lambda_function_impl(
    name: &str,
    collection: &Value,
    lambda: &Expression,
    context: &EvaluationContext,
) -> Result<Value> {
    let list = match collection {
        Value::List(items) => items,
        _ => {
            return Err(anyhow::anyhow!("{}() first argument must be a list", name));
        }
    };

    // Legacy form: the lambda written as a quoted string ('endsWith($item, ..)')
    // is parsed as an expression rather than treated as a literal
    let parsed;
    let lambda = if let Expression::StringLiteral(s) = lambda {
        parsed = parse_expression_impl(s).context(format!("Failed to parse expression: {}", s))?;
        &parsed
    } else {
        lambda
    };

    match name {
        "any" => {
            for item in list {
                if let Value::Boolean(true) = eval_with_item(lambda, item, context)? {
                    return Ok(Value::Boolean(true));
                }
            }
            Ok(Value::Boolean(false))
        }
        "all" => {
            for item in list {
                if let Value::Boolean(false) = eval_with_item(lambda, item, context)? {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        }
        "map" => {
            let mut result = Vec::new();
            for item in list {
                result.push(eval_with_item(lambda, item, context)?);
            }
            Ok(Value::List(result))
        }
        "filter" => {
            let mut result = Vec::new();
            for item in list {
                if let Value::Boolean(true) = eval_with_item(lambda, item, context)? {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(anyhow::anyhow!("Unknown lambda function: {}", name)),
    }
}

fn eval_with_item(lambda: &Expression, item: &Value, context: &EvaluationContext) -> Result<Value> {
    let item_context = EvaluationContext {
        variables: context.variables.clone(),
        path: context.path,
        custom_matchers: context.custom_matchers,
        item_context: Some(item.clone()),
        fs_cache: context.fs_cache,
        regex_cache: context.regex_cache,
    };

    crate::dsl::evaluator::evaluate_impl(lambda, &item_context)
}

/// Run a glob, using the run-wide cache when one is available so repeated
/// lookups of the same pattern don't re-read the filesystem.
fn glob_paths(pattern: &str, context: &EvaluationContext) -> Result<Vec<PathBuf>> {
    if let Some(cache) = context.fs_cache {
        if let Some(paths) = cache.borrow().get(pattern) {
            return Ok(paths.clone());
        }
    }

    let paths: Vec<PathBuf> = glob::glob(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?
        .flatten()
        .collect();

    if let Some(cache) = context.fs_cache {
        cache
            .borrow_mut()
            .insert(pattern.to_string(), paths.clone());
    }

    Ok(paths)
}

fn glob_names(pattern: &str, context: &EvaluationContext) -> Result<Vec<Value>> {
    Ok(glob_paths(pattern, context)?
        .iter()
        .filter_map(|path| path.file_name())
        .filter_map(|name| name.to_str())
        .map(|name| Value::String(name.to_string()))
        .collect())
}

/// Dispatches a built-in function call (`matches`, `exists`, `count`, ...)
/// by name.
///
/// # Errors
///
/// Returns [`crate::Error::Dsl`] if `name` is not a recognized function, or
/// if `args` has the wrong count or type for it.
pub fn call_function(
    name: &str,
    args: &[Value],
    context: &EvaluationContext,
) -> std::result::Result<Value, crate::Error> {
    call_function_impl(name, args, context).map_err(|e| crate::Error::Dsl(format!("{:#}", e)))
}

/// Implementation behind [`call_function`]; kept separate (and
/// anyhow-based) because it's mutually recursive with `dsl::evaluator`,
/// where the surrounding `anyhow::Context` chaining is more convenient than
/// converting back and forth through [`crate::Error`] on every call.
pub(crate) fn call_function_impl(
    name: &str,
    args: &[Value],
    context: &EvaluationContext,
) -> Result<Value> {
    match name {
        "matches" => matches_function(args, context),
        "in" => in_function(args, context),
        "exists" => exists_function(args, context),
        "siblings" => siblings_function(args, context),
        "children" => children_function(args, context),
        "find" => find_function(args, context),
        "without" => without_function(args, context),
        "any" | "all" | "map" | "filter" => string_lambda_function(name, args, context),
        "contains" => contains_function(args, context),
        "startsWith" => starts_with_function(args, context),
        "endsWith" => ends_with_function(args, context),
        "count" => count_function(args, context),
        _ => Err(anyhow::anyhow!("Unknown function: {}", name)),
    }
}

fn matches_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("matches() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::Regex(re)) => Ok(Value::Boolean(re.is_match(s))),
        (Value::String(s), Value::String(pattern)) => {
            // Treat string pattern as glob pattern
            let glob = Pattern::new(pattern)
                .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;
            Ok(Value::Boolean(glob.matches(s)))
        }
        _ => Err(anyhow::anyhow!(
            "matches() requires string and regex/string arguments"
        )),
    }
}

fn in_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("in() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::List(items)) => {
            let mut found = false;

            for item in items {
                if let Value::String(item_str) = item {
                    if s == item_str {
                        found = true;
                        break;
                    }
                }
            }

            Ok(Value::Boolean(found))
        }
        _ => Err(anyhow::anyhow!("in() requires string and list arguments")),
    }
}

fn exists_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(anyhow::anyhow!("exists() requires 1-3 arguments"));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(anyhow::anyhow!(
                "exists() first argument must be a string pattern"
            ));
        }
    };

    let min = if args.len() > 1 {
        match &args[1] {
            Value::Integer(i) if *i >= 0 => *i as usize,
            Value::Integer(_) => {
                return Err(anyhow::anyhow!("exists() min/max must be non-negative"));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "exists() second argument must be an integer"
                ));
            }
        }
    } else {
        1 // At least one match required by default
    };

    let max = if args.len() > 2 {
        match &args[2] {
            Value::Integer(i) if *i >= 0 => *i as usize,
            Value::Integer(_) => {
                return Err(anyhow::anyhow!("exists() min/max must be non-negative"));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "exists() third argument must be an integer"
                ));
            }
        }
    } else {
        usize::MAX // No upper limit by default
    };

    // Get parent directory
    let parent = if context.path.is_dir() {
        context.path
    } else {
        context.path.parent().unwrap_or(Path::new("."))
    };

    // Count matching files
    let glob_pattern = forward_slashes(&format!("{}/{}", parent.display(), pattern));
    let count = glob_paths(&glob_pattern, context)?.len();

    // Check if count is within range
    Ok(Value::Boolean(count >= min && count <= max))
}

fn siblings_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow::anyhow!("siblings() requires 1 argument"));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(anyhow::anyhow!(
                "siblings() argument must be a string pattern"
            ));
        }
    };

    // Get parent directory
    let parent = context.path.parent().unwrap_or(Path::new("."));

    // Find matching siblings
    let glob_pattern = forward_slashes(&format!("{}/{}", parent.display(), pattern));
    Ok(Value::List(glob_names(&glob_pattern, context)?))
}

fn children_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow::anyhow!("children() requires 1 argument"));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(anyhow::anyhow!(
                "children() argument must be a string pattern"
            ));
        }
    };

    // Check if current path is a directory
    if !context.path.is_dir() {
        return Ok(Value::List(Vec::new()));
    }

    // Find matching children
    let glob_pattern = forward_slashes(&format!("{}/{}", context.path.display(), pattern));
    Ok(Value::List(glob_names(&glob_pattern, context)?))
}

fn find_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("find() requires 2 arguments"));
    }

    let dir = match &args[0] {
        Value::String(s) => PathBuf::from(s),
        _ => {
            return Err(anyhow::anyhow!(
                "find() first argument must be a directory path"
            ));
        }
    };

    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(anyhow::anyhow!(
                "find() second argument must be a string pattern"
            ));
        }
    };

    // Find matching files
    let glob_pattern = forward_slashes(&format!("{}/{}", dir.display(), pattern));
    Ok(Value::List(glob_names(&glob_pattern, context)?))
}

fn without_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("without() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => {
            if let Some(stripped) = s.strip_suffix(suffix) {
                Ok(Value::String(stripped.to_string()))
            } else {
                Ok(Value::String(s.clone()))
            }
        }
        _ => Err(anyhow::anyhow!("without() requires string arguments")),
    }
}

/// Back-compat shim: a lambda that arrives as an already-evaluated string
/// value (e.g. from a string template) is parsed and delegated.
fn string_lambda_function(
    name: &str,
    args: &[Value],
    context: &EvaluationContext,
) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("{}() requires 2 arguments", name));
    }

    let expr = match &args[1] {
        Value::String(s) => {
            parse_expression_impl(s).context(format!("Failed to parse expression: {}", s))?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "{}() second argument must be an expression",
                name
            ));
        }
    };

    call_lambda_function_impl(name, &args[0], &expr, context)
}

fn contains_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("contains() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(haystack), Value::String(needle)) => {
            Ok(Value::Boolean(haystack.contains(needle)))
        }
        (Value::List(_), _) => Err(anyhow::anyhow!(
            "contains() checks substrings; for list membership use in(item, list)"
        )),
        _ => Err(anyhow::anyhow!(
            "contains() requires string haystack and substring arguments"
        )),
    }
}

fn starts_with_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("startsWith() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Boolean(s.starts_with(prefix))),
        _ => Err(anyhow::anyhow!("startsWith() requires string arguments")),
    }
}

fn ends_with_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow::anyhow!("endsWith() requires 2 arguments"));
    }

    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Boolean(s.ends_with(suffix))),
        _ => Err(anyhow::anyhow!("endsWith() requires string arguments")),
    }
}

fn count_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow::anyhow!("count() requires 1 argument"));
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Integer(items.len() as i64)),
        // Character count, not byte count: names like "café.js" should
        // measure the same regardless of encoding width
        Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
        _ => Err(anyhow::anyhow!(
            "count() requires a list or string argument"
        )),
    }
}
