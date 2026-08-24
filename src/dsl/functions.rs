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
    into_dsl_error(call_lambda_function_impl(name, collection, lambda, context))
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
    let Value::List(list) = collection else {
        return Err(anyhow::anyhow!("{name}() first argument must be a list"));
    };

    // Legacy form: the lambda written as a quoted string ('endsWith($item, ..)')
    // is parsed as an expression rather than treated as a literal
    let parsed;
    let lambda = if let Expression::StringLiteral(s) = lambda {
        parsed = parse_expression_impl(s).context(format!("Failed to parse expression: {s}"))?;
        &parsed
    } else {
        lambda
    };

    match name {
        // Both stop at the first item that settles the answer, so a lambda
        // that would error on a later item never runs. A non-boolean result
        // decides nothing and the search continues.
        "any" | "all" => {
            let decisive = name == "any";
            for item in list {
                if matches!(eval_with_item(lambda, item, context)?, Value::Boolean(b) if b == decisive)
                {
                    return Ok(Value::Boolean(decisive));
                }
            }
            Ok(Value::Boolean(!decisive))
        }
        // map keeps what the lambda returned; filter keeps the item the
        // lambda approved of.
        "map" | "filter" => {
            let keep_result = name == "map";
            let mut result = Vec::new();
            for item in list {
                let value = eval_with_item(lambda, item, context)?;
                if keep_result {
                    result.push(value);
                } else if matches!(value, Value::Boolean(true)) {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(anyhow::anyhow!("Unknown lambda function: {name}")),
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
        .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {e}"))?
        .flatten()
        .collect();

    if let Some(cache) = context.fs_cache {
        cache
            .borrow_mut()
            .insert(pattern.to_string(), paths.clone());
    }

    Ok(paths)
}

/// Flatten an internal anyhow error into the crate's public error type, which
/// is all the two public entry points do beyond delegating.
fn into_dsl_error(result: Result<Value>) -> std::result::Result<Value, crate::Error> {
    result.map_err(|e| crate::Error::Dsl(format!("{e:#}")))
}

/// Every built-in checks its own arity first, with the same message shape.
fn expect_args(name: &str, args: &[Value], count: usize) -> Result<()> {
    if args.len() == count {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "{name}() requires {count} argument{}",
        if count == 1 { "" } else { "s" }
    ))
}

/// Read a string argument, or fail with the caller's own wording — the
/// messages name the specific argument ("`find()` first argument must be a
/// directory path"), so they can't be generated from the function name.
fn string_arg<'a>(args: &'a [Value], index: usize, message: &str) -> Result<&'a str> {
    match &args[index] {
        Value::String(s) => Ok(s),
        _ => Err(anyhow::anyhow!("{message}")),
    }
}

/// Glob `pattern` under `base` and return the matching file names.
fn glob_in(base: &Path, pattern: &str, context: &EvaluationContext) -> Result<Value> {
    let glob_pattern = forward_slashes(&format!("{}/{}", base.display(), pattern));

    Ok(Value::List(
        glob_paths(&glob_pattern, context)?
            .iter()
            .filter_map(|path| path.file_name())
            .filter_map(|name| name.to_str())
            .map(|name| Value::String(name.to_string()))
            .collect(),
    ))
}

/// Every two-argument built-in has the same skeleton — check the arity, match
/// the argument types, apply the operation — and differs only in the middle
/// step. `apply` returns `None` when the arguments are the wrong types (the
/// caller's `type_error` is reported), or `Some(Err(..))` when the operation
/// itself fails and has something more specific to say.
fn binary_function(
    name: &str,
    args: &[Value],
    type_error: &str,
    apply: impl Fn(&Value, &Value) -> Option<Result<Value>>,
) -> Result<Value> {
    expect_args(name, args, 2)?;

    apply(&args[0], &args[1]).unwrap_or_else(|| Err(anyhow::anyhow!("{type_error}")))
}

/// The subset of [`binary_function`] whose operation is a `str` predicate that
/// cannot itself fail.
fn string_predicate(
    name: &str,
    args: &[Value],
    type_error: &str,
    predicate: impl Fn(&str, &str) -> bool,
) -> Result<Value> {
    binary_function(name, args, type_error, |a, b| match (a, b) {
        (Value::String(a), Value::String(b)) => Some(Ok(Value::Boolean(predicate(a, b)))),
        _ => None,
    })
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
    into_dsl_error(call_function_impl(name, args, context))
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
        "matches" => binary_function(
            name,
            args,
            "matches() requires string and regex/string arguments",
            |value, pattern| match (value, pattern) {
                (Value::String(s), Value::Regex(re)) => Some(Ok(Value::Boolean(re.is_match(s)))),
                // A string pattern is treated as a glob
                (Value::String(s), Value::String(pattern)) => Some(
                    Pattern::new(pattern)
                        .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {e}"))
                        .map(|glob| Value::Boolean(glob.matches(s))),
                ),
                _ => None,
            },
        ),
        "in" => binary_function(
            name,
            args,
            "in() requires string and list arguments",
            |needle, haystack| match (needle, haystack) {
                (Value::String(s), Value::List(items)) => {
                    Some(Ok(Value::Boolean(items.iter().any(
                        |item| matches!(item, Value::String(candidate) if candidate == s),
                    ))))
                }
                _ => None,
            },
        ),
        "without" => binary_function(
            name,
            args,
            "without() requires string arguments",
            |value, suffix| match (value, suffix) {
                (Value::String(s), Value::String(suffix)) => Some(Ok(Value::String(
                    s.strip_suffix(suffix).unwrap_or(s).to_string(),
                ))),
                _ => None,
            },
        ),
        "contains" => contains_function(args),
        "startsWith" => string_predicate(
            name,
            args,
            "startsWith() requires string arguments",
            |s, prefix| s.starts_with(prefix),
        ),
        "endsWith" => string_predicate(
            name,
            args,
            "endsWith() requires string arguments",
            |s, suffix| s.ends_with(suffix),
        ),
        "exists" => exists_function(args, context),
        "siblings" | "children" | "find" => glob_function(name, args, context),
        "any" | "all" | "map" | "filter" => string_lambda_function(name, args, context),
        "count" => count_function(args),
        _ => Err(anyhow::anyhow!("Unknown function: {name}")),
    }
}

/// Read one of `exists()`'s optional min/max bounds, falling back to
/// `default` when the argument was omitted. `position` names the argument in
/// the error ("second", "third").
fn bound_arg(args: &[Value], index: usize, position: &str, default: usize) -> Result<usize> {
    let Some(value) = args.get(index) else {
        return Ok(default);
    };

    match value {
        Value::Integer(i) => usize::try_from(*i)
            .map_err(|_| anyhow::anyhow!("exists() min/max must be non-negative")),
        _ => Err(anyhow::anyhow!(
            "exists() {position} argument must be an integer"
        )),
    }
}

fn exists_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(anyhow::anyhow!("exists() requires 1-3 arguments"));
    }

    let pattern = string_arg(args, 0, "exists() first argument must be a string pattern")?;

    // At least one match required by default, with no upper limit
    let min = bound_arg(args, 1, "second", 1)?;
    let max = bound_arg(args, 2, "third", usize::MAX)?;

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

/// `siblings()`, `children()` and `find()` all glob a pattern under some
/// directory and differ only in which directory that is, so they share one
/// implementation rather than three near-identical ones.
fn glob_function(name: &str, args: &[Value], context: &EvaluationContext) -> Result<Value> {
    match name {
        "siblings" => {
            expect_args(name, args, 1)?;
            let pattern = string_arg(args, 0, "siblings() argument must be a string pattern")?;
            glob_in(
                context.path.parent().unwrap_or(Path::new(".")),
                pattern,
                context,
            )
        }
        "children" => {
            expect_args(name, args, 1)?;
            let pattern = string_arg(args, 0, "children() argument must be a string pattern")?;

            // A file has no children; only directories can match
            if context.path.is_dir() {
                glob_in(context.path, pattern, context)
            } else {
                Ok(Value::List(Vec::new()))
            }
        }
        _ => {
            expect_args(name, args, 2)?;
            let dir = PathBuf::from(string_arg(
                args,
                0,
                "find() first argument must be a directory path",
            )?);
            let pattern = string_arg(args, 1, "find() second argument must be a string pattern")?;
            glob_in(&dir, pattern, context)
        }
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
        return Err(anyhow::anyhow!("{name}() requires 2 arguments"));
    }

    let expr = match &args[1] {
        Value::String(s) => {
            parse_expression_impl(s).context(format!("Failed to parse expression: {s}"))?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "{name}() second argument must be an expression"
            ));
        }
    };

    call_lambda_function_impl(name, &args[0], &expr, context)
}

/// `contains()` is [`string_predicate`] plus one extra steer: reaching for it
/// on a list is the natural mistake, and it deserves a better error than the
/// generic type message.
fn contains_function(args: &[Value]) -> Result<Value> {
    if let Some(Value::List(_)) = args.first() {
        return Err(anyhow::anyhow!(
            "contains() checks substrings; for list membership use in(item, list)"
        ));
    }

    string_predicate(
        "contains",
        args,
        "contains() requires string haystack and substring arguments",
        |haystack, needle| haystack.contains(needle),
    )
}

fn count_function(args: &[Value]) -> Result<Value> {
    expect_args("count", args, 1)?;

    let count = match &args[0] {
        Value::List(items) => items.len(),
        // Character count, not byte count: names like "café.js" should
        // measure the same regardless of encoding width
        Value::String(s) => s.chars().count(),
        _ => {
            return Err(anyhow::anyhow!(
                "count() requires a list or string argument"
            ));
        }
    };

    Ok(Value::Integer(i64::try_from(count).unwrap_or(i64::MAX)))
}
