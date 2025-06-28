use anyhow::{ Context, Result };
use glob::Pattern;
use std::path::{ Path, PathBuf };

use crate::dsl::evaluator::{ EvaluationContext, Value };
use crate::dsl::parser::parse_expression;

pub fn call_function(name: &str, args: &[Value], context: &EvaluationContext) -> Result<Value> {
  match name {
    "matches" => matches_function(args, context),
    "in" => in_function(args, context),
    "exists" => exists_function(args, context),
    "siblings" => siblings_function(args, context),
    "children" => children_function(args, context),
    "find" => find_function(args, context),
    "without" => without_function(args, context),
    "any" => any_function(args, context),
    "all" => all_function(args, context),
    "contains" => contains_function(args, context),
    "map" => map_function(args, context),
    "filter" => filter_function(args, context),
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
      let glob = Pattern::new(pattern).map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;
      Ok(Value::Boolean(glob.matches(s)))
    }
    _ => Err(anyhow::anyhow!("matches() requires string and regex/string arguments")),
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
  if args.len() < 1 || args.len() > 3 {
    return Err(anyhow::anyhow!("exists() requires 1-3 arguments"));
  }

  let pattern = match &args[0] {
    Value::String(s) => s,
    _ => {
      return Err(anyhow::anyhow!("exists() first argument must be a string pattern"));
    }
  };

  let min = if args.len() > 1 {
    match &args[1] {
      Value::Integer(i) => *i as usize,
      _ => {
        return Err(anyhow::anyhow!("exists() second argument must be an integer"));
      }
    }
  } else {
    1 // At least one match required by default
  };

  let max = if args.len() > 2 {
    match &args[2] {
      Value::Integer(i) => *i as usize,
      _ => {
        return Err(anyhow::anyhow!("exists() third argument must be an integer"));
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
  let glob_pattern = format!("{}/{}", parent.display(), pattern);
  let entries = glob
    ::glob(&glob_pattern)
    .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;

  let mut count = 0;
  for entry in entries {
    if entry.is_ok() {
      count += 1;
    }
  }

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
      return Err(anyhow::anyhow!("siblings() argument must be a string pattern"));
    }
  };

  // Get parent directory
  let parent = context.path.parent().unwrap_or(Path::new("."));

  // Find matching siblings
  let glob_pattern = format!("{}/{}", parent.display(), pattern);
  let entries = glob
    ::glob(&glob_pattern)
    .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;

  let mut result = Vec::new();

  for entry in entries {
    if let Ok(path) = entry {
      if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
          result.push(Value::String(name_str.to_string()));
        }
      }
    }
  }

  Ok(Value::List(result))
}

fn children_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
  if args.len() != 1 {
    return Err(anyhow::anyhow!("children() requires 1 argument"));
  }

  let pattern = match &args[0] {
    Value::String(s) => s,
    _ => {
      return Err(anyhow::anyhow!("children() argument must be a string pattern"));
    }
  };

  // Check if current path is a directory
  if !context.path.is_dir() {
    return Ok(Value::List(Vec::new()));
  }

  // Find matching children
  let glob_pattern = format!("{}/{}", context.path.display(), pattern);
  let entries = glob
    ::glob(&glob_pattern)
    .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;

  let mut result = Vec::new();

  for entry in entries {
    if let Ok(path) = entry {
      if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
          result.push(Value::String(name_str.to_string()));
        }
      }
    }
  }

  Ok(Value::List(result))
}

fn find_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("find() requires 2 arguments"));
  }

  let dir = match &args[0] {
    Value::String(s) => PathBuf::from(s),
    Value::Path(p) => p.clone(),
    _ => {
      return Err(anyhow::anyhow!("find() first argument must be a directory path"));
    }
  };

  let pattern = match &args[1] {
    Value::String(s) => s,
    _ => {
      return Err(anyhow::anyhow!("find() second argument must be a string pattern"));
    }
  };

  // Find matching files
  let glob_pattern = format!("{}/{}", dir.display(), pattern);
  let entries = glob
    ::glob(&glob_pattern)
    .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;

  let mut result = Vec::new();

  for entry in entries {
    if let Ok(path) = entry {
      if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
          result.push(Value::String(name_str.to_string()));
        }
      }
    }
  }

  Ok(Value::List(result))
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

fn any_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("any() requires 2 arguments"));
  }

  let list = match &args[0] {
    Value::List(items) => items,
    _ => {
      return Err(anyhow::anyhow!("any() first argument must be a list"));
    }
  };

  let expr = match &args[1] {
    Value::String(s) => {
      parse_expression(s).context(format!("Failed to parse expression: {}", s))?
    }
    _ => {
      return Err(anyhow::anyhow!("any() second argument must be an expression"));
    }
  };

  for item in list {
    let new_context = EvaluationContext {
      variables: context.variables.clone(),
      path: context.path,
      custom_matchers: context.custom_matchers,
      item_context: Some(item.clone()),
    };

    match crate::dsl::evaluator::evaluate(&expr, &new_context)? {
      Value::Boolean(true) => {
        return Ok(Value::Boolean(true));
      }
      _ => {}
    }
  }

  Ok(Value::Boolean(false))
}

fn all_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("all() requires 2 arguments"));
  }

  let list = match &args[0] {
    Value::List(items) => items,
    _ => {
      return Err(anyhow::anyhow!("all() first argument must be a list"));
    }
  };

  let expr = match &args[1] {
    Value::String(s) => {
      parse_expression(s).context(format!("Failed to parse expression: {}", s))?
    }
    _ => {
      return Err(anyhow::anyhow!("all() second argument must be an expression"));
    }
  };

  for item in list {
    let new_context = EvaluationContext {
      variables: context.variables.clone(),
      path: context.path,
      custom_matchers: context.custom_matchers,
      item_context: Some(item.clone()),
    };

    match crate::dsl::evaluator::evaluate(&expr, &new_context)? {
      Value::Boolean(false) => {
        return Ok(Value::Boolean(false));
      }
      _ => {}
    }
  }

  Ok(Value::Boolean(true))
}

fn contains_function(args: &[Value], _context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("contains() requires 2 arguments"));
  }

  match (&args[0], &args[1]) {
    (Value::List(list), item) => {
      let mut found = false;

      for list_item in list {
        if list_item == item {
          found = true;
          break;
        }
      }

      Ok(Value::Boolean(found))
    }
    (Value::String(haystack), Value::String(needle)) => {
      Ok(Value::Boolean(haystack.contains(needle)))
    }
    _ => Err(anyhow::anyhow!("contains() requires list and item or string and substring")),
  }
}

fn map_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("map() requires 2 arguments"));
  }

  let list = match &args[0] {
    Value::List(items) => items,
    _ => {
      return Err(anyhow::anyhow!("map() first argument must be a list"));
    }
  };

  let expr = match &args[1] {
    Value::String(s) => {
      parse_expression(s).context(format!("Failed to parse expression: {}", s))?
    }
    _ => {
      return Err(anyhow::anyhow!("map() second argument must be an expression"));
    }
  };

  let mut result = Vec::new();

  for item in list {
    let new_context = EvaluationContext {
      variables: context.variables.clone(),
      path: context.path,
      custom_matchers: context.custom_matchers,
      item_context: Some(item.clone()),
    };

    let mapped_value = crate::dsl::evaluator::evaluate(&expr, &new_context)?;
    result.push(mapped_value);
  }

  Ok(Value::List(result))
}

fn filter_function(args: &[Value], context: &EvaluationContext) -> Result<Value> {
  if args.len() != 2 {
    return Err(anyhow::anyhow!("filter() requires 2 arguments"));
  }

  let list = match &args[0] {
    Value::List(items) => items,
    _ => {
      return Err(anyhow::anyhow!("filter() first argument must be a list"));
    }
  };

  let expr = match &args[1] {
    Value::String(s) => {
      parse_expression(s).context(format!("Failed to parse expression: {}", s))?
    }
    _ => {
      return Err(anyhow::anyhow!("filter() second argument must be an expression"));
    }
  };

  let mut result = Vec::new();

  for item in list {
    let new_context = EvaluationContext {
      variables: context.variables.clone(),
      path: context.path,
      custom_matchers: context.custom_matchers,
      item_context: Some(item.clone()),
    };

    match crate::dsl::evaluator::evaluate(&expr, &new_context)? {
      Value::Boolean(true) => result.push(item.clone()),
      _ => {}
    }
  }

  Ok(Value::List(result))
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
    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
    _ => Err(anyhow::anyhow!("count() requires a list or string argument")),
  }
}
