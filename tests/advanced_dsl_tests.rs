/// Comprehensive tests for advanced DSL features in lintp
///
/// This module tests the sophisticated file system query and collection processing
/// capabilities that differentiate lintp as a powerful "SQL for file systems" tool.
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

use lintp::dsl::evaluator::{ evaluate, EvaluationContext, Value };
use lintp::dsl::functions::call_function;
use lintp::dsl::parser::parse_expression;

mod test_helpers;

use test_helpers::*;

// =============================================================================
// TEST UTILITIES FOR ADVANCED DSL TESTING
// =============================================================================

/// Create a comprehensive test project structure for advanced testing
fn create_advanced_test_project() -> Result<TempDir> {
  let temp_dir = tempfile::tempdir()?;
  let root = temp_dir.path();

  // Create comprehensive directory structure
  std::fs::create_dir_all(root.join("src/components"))?;
  std::fs::create_dir_all(root.join("src/utils"))?;

  // Create component files
  create_test_file(root, "src/components/Button.js", "// Button component")?;
  create_test_file(root, "src/components/Button.spec.js", "// Button tests")?;
  create_test_file(root, "src/components/Card.jsx", "// Card component")?;
  create_test_file(root, "src/components/Modal.tsx", "// Modal component")?;

  // Create utility files
  create_test_file(root, "src/utils/format-date.js", "// Date formatter")?;
  create_test_file(root, "src/utils/api-client.ts", "// API client")?;

  Ok(temp_dir)
}

/// Helper to evaluate an expression in a context
fn eval_in_context(expr_str: &str, context: &EvaluationContext) -> Result<Value> {
  let expr = parse_expression(expr_str)?;
  evaluate(&expr, context)
}

/// Helper to evaluate expression and expect boolean result
fn eval_bool(expr_str: &str, context: &EvaluationContext) -> Result<bool> {
  match eval_in_context(expr_str, context)? {
    Value::Boolean(b) => Ok(b),
    other => Err(anyhow::anyhow!("Expected boolean, got: {:?}", other)),
  }
}

/// Helper to evaluate expression and expect list result
fn eval_list(expr_str: &str, context: &EvaluationContext) -> Result<Vec<String>> {
  match eval_in_context(expr_str, context)? {
    Value::List(items) => {
      let mut result = Vec::new();
      for item in items {
        match item {
          Value::String(s) => result.push(s),
          other => {
            return Err(anyhow::anyhow!("Expected string in list, got: {:?}", other));
          }
        }
      }
      Ok(result)
    }
    other => Err(anyhow::anyhow!("Expected list, got: {:?}", other)),
  }
}

/// Helper to evaluate expression and expect string result
fn eval_string(expr_str: &str, context: &EvaluationContext) -> Result<String> {
  match eval_in_context(expr_str, context)? {
    Value::String(s) => Ok(s),
    other => Err(anyhow::anyhow!("Expected string, got: {:?}", other)),
  }
}

// =============================================================================
// FILE SYSTEM QUERY FUNCTION TESTS
// =============================================================================

#[test]
fn test_exists_function_basic() -> Result<()> {
  let project = create_advanced_test_project()?;
  let custom_matchers = HashMap::new();

  // Create a test context with the project path
  let test_path = project.path().join("src/components/Button.js");
  let context = create_test_evaluation_context(&test_path, &custom_matchers);

  // Should find existing files in same directory
  assert!(eval_bool("exists('*.jsx')", &context)?);
  assert!(eval_bool("exists('*.tsx')", &context)?);
  assert!(eval_bool("exists('*.spec.js')", &context)?);

  // Should not find non-existent files
  assert!(!eval_bool("exists('*.py')", &context)?);
  assert!(!eval_bool("exists('*.php')", &context)?);

  Ok(())
}

#[test]
fn test_siblings_function() -> Result<()> {
  let project = create_advanced_test_project()?;
  let custom_matchers = HashMap::new();

  // Test from Button.js in components directory
  let test_path = project.path().join("src/components/Button.js");
  let context = create_test_evaluation_context(&test_path, &custom_matchers);

  // Get all JavaScript files in same directory
  let js_files = eval_list("siblings('*.js')", &context)?;
  assert!(js_files.contains(&"Button.js".to_string()));
  assert!(js_files.contains(&"Button.spec.js".to_string()));

  // Get all JSX files in same directory
  let jsx_files = eval_list("siblings('*.jsx')", &context)?;
  assert!(jsx_files.contains(&"Card.jsx".to_string()));

  Ok(())
}

#[test]
fn test_find_function() -> Result<()> {
  let project = create_advanced_test_project()?;
  let custom_matchers = HashMap::new();
  let test_path = project.path().join("src/components/Button.js");
  let context = create_test_evaluation_context(&test_path, &custom_matchers);

  // Find all JS files in utils directory
  let utils_path = project.path().join("src/utils");
  let utils_js = call_function(
    "find",
    &[Value::String(utils_path.to_string_lossy().to_string()), Value::String("*.js".to_string())],
    &context
  )?;

  if let Value::List(files) = utils_js {
    let file_names: Vec<String> = files
      .into_iter()
      .filter_map(|v| {
        if let Value::String(s) = v { Some(s) } else { None }
      })
      .collect();
    assert!(file_names.contains(&"format-date.js".to_string()));
  } else {
    panic!("Expected list result from find function");
  }

  Ok(())
}

// =============================================================================
// COLLECTION PROCESSING FUNCTION TESTS
// =============================================================================

#[test]
fn test_map_function_basic() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test mapping over simple list
  let result = eval_list("map(['hello.js', 'world.ts'], 'without($item, \".js\")')", &context)?;
  assert_eq!(result, vec!["hello".to_string(), "world.ts".to_string()]);

  // Test mapping with string templates - just return the item for now
  let result = eval_list("map(['Button', 'Card'], '${$item}')", &context)?;
  assert_eq!(result, vec!["Button".to_string(), "Card".to_string()]);

  Ok(())
}

#[test]
fn test_any_function() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test any() with simple list - should find match
  assert!(eval_bool("any(['a.js', 'b.ts', 'c.jsx'], 'matches($item, /\\.js$/)')", &context)?);

  // Test any() with simple list - should not find match
  assert!(!eval_bool("any(['a.ts', 'b.tsx', 'c.jsx'], 'matches($item, /\\.py$/)')", &context)?);

  // Test any() with empty list
  assert!(!eval_bool("any([], 'matches($item, /\\.js$/)')", &context)?);

  Ok(())
}

#[test]
fn test_all_function() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test all() where all items match
  assert!(eval_bool("all(['a.js', 'b.js', 'c.js'], 'matches($item, /\\.js$/)')", &context)?);

  // Test all() where not all items match
  assert!(!eval_bool("all(['a.js', 'b.ts', 'c.js'], 'matches($item, /\\.js$/)')", &context)?);

  // Test all() with empty list (vacuously true)
  assert!(eval_bool("all([], 'matches($item, /\\.js$/)')", &context)?);

  Ok(())
}

#[test]
fn test_filter_function() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Filter list to only JS files
  let js_files = eval_list(
    "filter(['a.js', 'b.ts', 'c.jsx', 'd.js'], 'matches($item, /\\.js$/)')",
    &context
  )?;
  assert_eq!(js_files, vec!["a.js".to_string(), "d.js".to_string()]);

  // Filter with more complex condition
  let long_names = eval_list("filter(['ab.js', 'xyz.js', 'a.js'], 'count($item) > 4')", &context)?;
  assert_eq!(long_names, vec!["ab.js".to_string(), "xyz.js".to_string()]);

  Ok(())
}

// =============================================================================
// STRING TEMPLATE AND CONTEXT TESTS
// =============================================================================

#[test]
fn test_string_templates_basic() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/Button.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test basic variable interpolation - using simple string concatenation for now
  let result = eval_string("${$BASENAME}", &context)?;
  assert_eq!(result, "Button");

  // Test multiple variables - test them separately
  let result = eval_string("${$EXT}", &context)?;
  assert_eq!(result, "js");

  Ok(())
}

#[test]
fn test_item_context_variable() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test $item in map context - use simple string concatenation
  let result = eval_list("map(['a', 'b', 'c'], '${$item}')", &context)?;
  assert_eq!(result, vec!["a".to_string(), "b".to_string(), "c".to_string()]);

  // Test $item in any context
  assert!(eval_bool("any(['test.js', 'other.ts'], '$item == \"test.js\"')", &context)?);

  // Test $item in all context
  assert!(eval_bool("all(['a.js', 'b.js'], 'endsWith($item, \".js\")')", &context)?);

  // Test $item in filter context
  let filtered = eval_list("filter(['short', 'verylongname'], 'count($item) > 5')", &context)?;
  assert_eq!(filtered, vec!["verylongname".to_string()]);

  Ok(())
}

#[test]
fn test_item_context_error_handling() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test $item outside collection context should fail
  let result = eval_in_context("$item", &context);
  assert!(result.is_err());

  Ok(())
}

// =============================================================================
// REAL-WORLD INTEGRATION TESTS
// =============================================================================

#[test]
fn test_component_test_validation() -> Result<()> {
  let project = create_advanced_test_project()?;
  let custom_matchers = HashMap::new();

  // Test the sophisticated query from the project synopsis:
  // any(map(siblings("*.js"), without($item, ".js")), exists("${item}.spec.js"))
  //
  // This checks if any component in the directory has a corresponding test file

  let test_path = project.path().join("src/components/Button.js");
  let context = create_test_evaluation_context(&test_path, &custom_matchers);

  // This expression should be true because Button.js has Button.spec.js
  let has_tests = eval_bool(
    "any(map(siblings('*.js'), 'without($item, \".js\")'), 'exists(\"${$item}.spec.js\")')",
    &context
  )?;
  assert!(has_tests);

  Ok(())
}

#[test]
fn test_performance_with_large_collections() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Create a large list for performance testing
  let large_list: Vec<String> = (0..1000).map(|i| format!("file{}.js", i)).collect();
  let large_list_value = Value::List(
    large_list
      .iter()
      .map(|s| Value::String(s.clone()))
      .collect()
  );

  // Test any() short-circuits (should be fast)
  let result = call_function(
    "any",
    &[large_list_value.clone(), Value::String("$item == 'file0.js'".to_string())],
    &context
  )?;
  assert_eq!(result, Value::Boolean(true));

  // Test all() short-circuits on false (should be fast)
  let result = call_function(
    "all",
    &[large_list_value, Value::String("$item == 'file0.js'".to_string())],
    &context
  )?;
  assert_eq!(result, Value::Boolean(false));

  Ok(())
}

// =============================================================================
// ERROR HANDLING AND EDGE CASES
// =============================================================================

#[test]
fn test_error_handling_invalid_arguments() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test exists with invalid arguments
  assert!(eval_in_context("exists()", &context).is_err());
  assert!(eval_in_context("exists(123)", &context).is_err());

  // Test map with invalid arguments
  assert!(eval_in_context("map('not a list', '${$item}')", &context).is_err());
  assert!(eval_in_context("map(['a', 'b'], 123)", &context).is_err());

  // Test any/all with invalid arguments
  assert!(eval_in_context("any('not a list', '$item == \"a\"')", &context).is_err());
  assert!(eval_in_context("all(['a'], 'invalid expression syntax !!!')", &context).is_err());

  Ok(())
}

#[test]
fn test_edge_cases_empty_results() -> Result<()> {
  let custom_matchers = HashMap::new();
  let test_path = Path::new("/tmp/test.js");
  let context = create_test_evaluation_context(test_path, &custom_matchers);

  // Test functions with empty results
  assert!(!eval_bool("exists('*.nonexistent')", &context)?);

  let empty_map = eval_list("map([], '${$item}')", &context)?;
  assert!(empty_map.is_empty());

  let empty_filter = eval_list("filter(['a', 'b'], '$item == \"c\"')", &context)?;
  assert!(empty_filter.is_empty());

  Ok(())
}
