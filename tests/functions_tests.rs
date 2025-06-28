use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use lintp::dsl::ast::Expression;
use lintp::dsl::evaluator::{ EvaluationContext, Value };
use lintp::dsl::functions::call_function;

/// Helper function to create a basic evaluation context for testing
fn create_test_context<'a>(
  path: &'a Path,
  custom_matchers: &'a HashMap<String, Expression>
) -> EvaluationContext<'a> {
  let mut variables = HashMap::new();
  variables.insert("NAME".to_string(), Value::String("test.js".to_string()));
  variables.insert("PATH".to_string(), Value::Path(path.to_path_buf()));
  variables.insert("EXT".to_string(), Value::String("js".to_string()));
  variables.insert("BASENAME".to_string(), Value::String("test".to_string()));

  if let Some(parent) = path.parent() {
    variables.insert("PARENT".to_string(), Value::Path(parent.to_path_buf()));
  }

  EvaluationContext {
    variables,
    path,
    custom_matchers,
    item_context: None,
  }
}

/// Tests for the matches function
#[test]
fn test_matches_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test with regex pattern - matching
  let args = vec![Value::String("test.js".to_string()), Value::Regex(Regex::new("^test\\.js$")?)];
  let result = call_function("matches", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with regex pattern - non-matching
  let args = vec![Value::String("test.js".to_string()), Value::Regex(Regex::new("^other\\.js$")?)];
  let result = call_function("matches", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with string pattern as glob
  let args = vec![Value::String("test.js".to_string()), Value::String("*.js".to_string())];
  let result = call_function("matches", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with invalid number of arguments
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("matches", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument types
  let args = vec![Value::Integer(42), Value::Regex(Regex::new("test")?)];
  let result = call_function("matches", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the in function
#[test]
fn test_in_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test with string in list - found
  let args = vec![
    Value::String("js".to_string()),
    Value::List(vec![Value::String("js".to_string()), Value::String("ts".to_string())])
  ];
  let result = call_function("in", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with string in list - not found
  let args = vec![
    Value::String("php".to_string()),
    Value::List(vec![Value::String("js".to_string()), Value::String("ts".to_string())])
  ];
  let result = call_function("in", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with invalid number of arguments
  let args = vec![Value::String("js".to_string())];
  let result = call_function("in", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument types
  let args = vec![Value::String("js".to_string()), Value::String("not a list".to_string())];
  let result = call_function("in", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the without function
#[test]
fn test_without_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test removing suffix - matching
  let args = vec![Value::String("test.js".to_string()), Value::String(".js".to_string())];
  let result = call_function("without", &args, &context)?;
  assert!(matches!(result, Value::String(s) if s == "test"));

  // Test removing suffix - non-matching
  let args = vec![Value::String("test.js".to_string()), Value::String(".ts".to_string())];
  let result = call_function("without", &args, &context)?;
  assert!(matches!(result, Value::String(s) if s == "test.js"));

  // Test with invalid number of arguments
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("without", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument types
  let args = vec![Value::Integer(42), Value::String(".js".to_string())];
  let result = call_function("without", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the contains function
#[test]
fn test_contains_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test with string contains substring
  let args = vec![Value::String("test.js".to_string()), Value::String("test".to_string())];
  let result = call_function("contains", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with string does not contain substring
  let args = vec![Value::String("test.js".to_string()), Value::String("xyz".to_string())];
  let result = call_function("contains", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with list contains item
  let args = vec![
    Value::List(vec![Value::String("js".to_string()), Value::String("ts".to_string())]),
    Value::String("js".to_string())
  ];
  let result = call_function("contains", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with list does not contain item
  let args = vec![
    Value::List(vec![Value::String("js".to_string()), Value::String("ts".to_string())]),
    Value::String("php".to_string())
  ];
  let result = call_function("contains", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with invalid number of arguments
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("contains", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument types
  let args = vec![Value::Integer(42), Value::String("test".to_string())];
  let result = call_function("contains", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the startsWith function
#[test]
fn test_starts_with_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test with string that starts with prefix
  let args = vec![Value::String("test.js".to_string()), Value::String("test".to_string())];
  let result = call_function("startsWith", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with string that doesn't start with prefix
  let args = vec![Value::String("test.js".to_string()), Value::String("js".to_string())];
  let result = call_function("startsWith", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with invalid number of arguments
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("startsWith", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument types
  let args = vec![Value::Integer(42), Value::String("test".to_string())];
  let result = call_function("startsWith", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the endsWith function
#[test]
fn test_ends_with_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test with string that ends with suffix
  let args = vec![Value::String("test.js".to_string()), Value::String(".js".to_string())];
  let result = call_function("endsWith", &args, &context)?;
  assert!(matches!(result, Value::Boolean(true)));

  // Test with string that doesn't end with suffix
  let args = vec![Value::String("test.js".to_string()), Value::String("test".to_string())];
  let result = call_function("endsWith", &args, &context)?;
  assert!(matches!(result, Value::Boolean(false)));

  // Test with invalid number of arguments
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("endsWith", &args, &context);
  assert!(result.is_err());

  Ok(())
}

/// Tests for the count function
#[test]
fn test_count_function() -> Result<()> {
  let path = Path::new("/tmp/test.js");
  let custom_matchers = HashMap::new();
  let context = create_test_context(path, &custom_matchers);

  // Test counting string length
  let args = vec![Value::String("test.js".to_string())];
  let result = call_function("count", &args, &context)?;
  assert!(matches!(result, Value::Integer(7)));

  // Test counting list items
  let args = vec![
    Value::List(
      vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string())
      ]
    )
  ];
  let result = call_function("count", &args, &context)?;
  assert!(matches!(result, Value::Integer(3)));

  // Test with empty list
  let args = vec![Value::List(vec![])];
  let result = call_function("count", &args, &context)?;
  assert!(matches!(result, Value::Integer(0)));

  // Test with invalid number of arguments
  let args = vec![];
  let result = call_function("count", &args, &context);
  assert!(result.is_err());

  // Test with invalid argument type
  let args = vec![Value::Boolean(true)];
  let result = call_function("count", &args, &context);
  assert!(result.is_err());

  Ok(())
}
