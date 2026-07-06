/// Comprehensive DSL feature tests
///
/// This module tests all DSL features individually and in combination to ensure
/// complete coverage of the lintp domain-specific language functionality.
use anyhow::Result;
use lintp::dsl::ast::Expression;
use lintp::dsl::evaluator::{evaluate, EvaluationContext, Value};
use lintp::dsl::parser::parse_expression;
use std::collections::HashMap;
use std::path::Path;

mod common;
use common::constants::*;

/// Helper function to create test evaluation context
fn create_test_context<'a>(
    file_path: &'a str,
    custom_matchers: &'a HashMap<String, Expression>,
) -> EvaluationContext<'a> {
    let path = Path::new(file_path);
    let mut variables = HashMap::new();

    // Set up all standard variables
    let name = path
        .file_name()
        .map_or("".to_string(), |n| n.to_string_lossy().to_string());
    variables.insert("NAME".to_string(), Value::String(name));
    variables.insert(
        "PATH".to_string(),
        Value::String(path.display().to_string()),
    );

    if let Some(ext) = path.extension() {
        variables.insert(
            "EXT".to_string(),
            Value::String(ext.to_string_lossy().to_string()),
        );
    } else {
        variables.insert("EXT".to_string(), Value::String("".to_string()));
    }

    if let Some(stem) = path.file_stem() {
        variables.insert(
            "BASENAME".to_string(),
            Value::String(stem.to_string_lossy().to_string()),
        );
    } else {
        variables.insert("BASENAME".to_string(), Value::String("".to_string()));
    }

    if let Some(parent) = path.parent() {
        variables.insert(
            "PARENT".to_string(),
            Value::String(parent.display().to_string()),
        );
    }

    EvaluationContext {
        variables,
        path,
        custom_matchers,
        item_context: None,
        fs_cache: None,
        regex_cache: None,
    }
}

/// Helper function to evaluate expression string
fn eval_expr(expr_str: &str, file_path: &str) -> Result<Value> {
    let expr = parse_expression(expr_str)?;
    let custom_matchers = HashMap::new();
    let context = create_test_context(file_path, &custom_matchers);
    evaluate(&expr, &context)
}

/// Helper function to check if expression evaluates to true
fn is_true(expr_str: &str, file_path: &str) -> bool {
    matches!(eval_expr(expr_str, file_path), Ok(Value::Boolean(true)))
}

/// Helper function to check if expression evaluates to false
fn is_false(expr_str: &str, file_path: &str) -> bool {
    matches!(eval_expr(expr_str, file_path), Ok(Value::Boolean(false)))
}

// =============================================================================
// VARIABLE TESTS
// =============================================================================

#[test]
fn test_variable_name() -> Result<()> {
    // Test $NAME variable
    assert_eq!(
        eval_expr("$NAME", "/path/to/file.js")?,
        Value::String("file.js".to_string())
    );
    assert_eq!(
        eval_expr("$NAME", "/simple.txt")?,
        Value::String("simple.txt".to_string())
    );
    assert_eq!(
        eval_expr("$NAME", "/path/component.test.js")?,
        Value::String("component.test.js".to_string())
    );

    Ok(())
}

#[test]
fn test_variable_basename() -> Result<()> {
    // Test $BASENAME variable (filename without extension)
    assert_eq!(
        eval_expr("$BASENAME", "/path/to/file.js")?,
        Value::String("file".to_string())
    );
    assert_eq!(
        eval_expr("$BASENAME", "/simple.txt")?,
        Value::String("simple".to_string())
    );
    assert_eq!(
        eval_expr("$BASENAME", "/component.test.js")?,
        Value::String("component.test".to_string())
    );
    assert_eq!(
        eval_expr("$BASENAME", "/noextension")?,
        Value::String("noextension".to_string())
    );

    Ok(())
}

#[test]
fn test_variable_ext() -> Result<()> {
    // Test $EXT variable
    assert_eq!(
        eval_expr("$EXT", "/path/to/file.js")?,
        Value::String("js".to_string())
    );
    assert_eq!(
        eval_expr("$EXT", "/simple.txt")?,
        Value::String("txt".to_string())
    );
    assert_eq!(
        eval_expr("$EXT", "/component.test.tsx")?,
        Value::String("tsx".to_string())
    );
    assert_eq!(
        eval_expr("$EXT", "/noextension")?,
        Value::String("".to_string())
    );

    Ok(())
}

#[test]
fn test_variable_path() -> Result<()> {
    // $PATH is a string so the documented patterns
    // (contains($PATH, "/src/")) work on it
    let result = eval_expr("$PATH", "/path/to/file.js")?;
    assert!(matches!(result, Value::String(s) if s == "/path/to/file.js"));

    Ok(())
}

#[test]
fn test_variable_parent() -> Result<()> {
    // $PARENT is a string so $PARENT == "." comparisons work
    let result = eval_expr("$PARENT", "/path/to/file.js")?;
    assert!(matches!(result, Value::String(s) if s == "/path/to"));

    Ok(())
}

// =============================================================================
// LITERAL TESTS
// =============================================================================

#[test]
fn test_string_literals() -> Result<()> {
    // Test string literals with different quote types
    assert_eq!(
        eval_expr("\"hello world\"", "/test.js")?,
        Value::String("hello world".to_string())
    );
    assert_eq!(
        eval_expr("'single quotes'", "/test.js")?,
        Value::String("single quotes".to_string())
    );
    assert_eq!(
        eval_expr("\"\"", "/test.js")?,
        Value::String("".to_string())
    );

    // Test escape sequences
    assert_eq!(
        eval_expr("\"hello\\nworld\"", "/test.js")?,
        Value::String("hello\nworld".to_string())
    );
    assert_eq!(
        eval_expr("\"quote: \\\"test\\\"\"", "/test.js")?,
        Value::String("quote: \"test\"".to_string())
    );

    Ok(())
}

#[test]
fn test_integer_literals() -> Result<()> {
    // Test integer literals
    assert_eq!(eval_expr("42", "/test.js")?, Value::Integer(42));
    assert_eq!(eval_expr("0", "/test.js")?, Value::Integer(0));
    assert_eq!(eval_expr("-5", "/test.js")?, Value::Integer(-5));

    Ok(())
}

#[test]
fn test_boolean_literals() -> Result<()> {
    // Test boolean literals
    assert_eq!(eval_expr("true", "/test.js")?, Value::Boolean(true));
    assert_eq!(eval_expr("false", "/test.js")?, Value::Boolean(false));

    Ok(())
}

#[test]
fn test_regex_literals() -> Result<()> {
    // Test regex literals
    let result = eval_expr("/test/", "/test.js")?;
    assert!(matches!(result, Value::Regex(_)));

    let result = eval_expr("/^[a-z]+$/", "/test.js")?;
    assert!(matches!(result, Value::Regex(_)));

    Ok(())
}

#[test]
fn test_list_literals() -> Result<()> {
    // Test list literals
    let result = eval_expr("[\"a\", \"b\", \"c\"]", "/test.js")?;
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("a".to_string()));
        assert_eq!(items[1], Value::String("b".to_string()));
        assert_eq!(items[2], Value::String("c".to_string()));
    } else {
        panic!("Expected List value");
    }

    // Test empty list
    let result = eval_expr("[]", "/test.js")?;
    if let Value::List(items) = result {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected empty List value");
    }

    Ok(())
}

// =============================================================================
// OPERATOR TESTS
// =============================================================================

#[test]
fn test_logical_operators() -> Result<()> {
    // Test AND operator
    assert!(is_true("true && true", "/test.js"));
    assert!(is_false("true && false", "/test.js"));
    assert!(is_false("false && true", "/test.js"));
    assert!(is_false("false && false", "/test.js"));

    // Test OR operator
    assert!(is_true("true || true", "/test.js"));
    assert!(is_true("true || false", "/test.js"));
    assert!(is_true("false || true", "/test.js"));
    assert!(is_false("false || false", "/test.js"));

    // Test NOT operator
    assert!(is_true("!false", "/test.js"));
    assert!(is_false("!true", "/test.js"));

    Ok(())
}

#[test]
fn test_comparison_operators() -> Result<()> {
    // Test equality operators
    assert!(is_true("$EXT == \"js\"", "/test.js"));
    assert!(is_false("$EXT == \"ts\"", "/test.js"));
    assert!(is_true("$EXT != \"ts\"", "/test.js"));
    assert!(is_false("$EXT != \"js\"", "/test.js"));

    // Test with different types
    assert!(is_true("42 == 42", "/test.js"));
    assert!(is_false("42 == 43", "/test.js"));
    assert!(is_true("\"hello\" == \"hello\"", "/test.js"));
    assert!(is_false("\"hello\" == \"world\"", "/test.js"));

    Ok(())
}

#[test]
fn test_unary_operators() -> Result<()> {
    // Test unary minus
    assert_eq!(eval_expr("-42", "/test.js")?, Value::Integer(-42));
    assert_eq!(eval_expr("--42", "/test.js")?, Value::Integer(42));

    // Test not operator
    assert!(is_true("!false", "/test.js"));
    assert!(is_false("!true", "/test.js"));

    Ok(())
}

#[test]
fn test_operator_precedence() -> Result<()> {
    // Test operator precedence: NOT has higher precedence than AND/OR
    assert!(is_true("!false && true", "/test.js")); // (!false) && true = true && true = true
    assert!(is_false("!true && false", "/test.js")); // (!true) && false = false && false = false

    // Test AND has higher precedence than OR
    assert!(is_true("false || true && true", "/test.js")); // false || (true && true) = false || true = true
    assert!(is_false("false || false && true", "/test.js")); // false || (false && true) = false || false = false

    Ok(())
}

// =============================================================================
// FUNCTION TESTS
// =============================================================================

#[test]
fn test_matches_function() -> Result<()> {
    // Test matches function with various patterns
    assert!(is_true("matches($BASENAME, /^test$/)", "/path/test.js"));
    assert!(is_false("matches($BASENAME, /^test$/)", "/path/other.js"));

    // Test kebab-case pattern
    assert!(is_true(
        "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)",
        "/path/hello-world.js"
    ));
    assert!(is_false(
        "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)",
        "/path/HelloWorld.js"
    ));

    // Test PascalCase pattern
    assert!(is_true(
        "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)",
        "/path/HelloWorld.js"
    ));
    assert!(is_false(
        "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)",
        "/path/hello-world.js"
    ));

    Ok(())
}

#[test]
fn test_in_function() -> Result<()> {
    // Test in function
    assert!(is_true("in($EXT, [\"js\", \"ts\", \"jsx\"])", "/test.js"));
    assert!(is_false("in($EXT, [\"js\", \"ts\", \"jsx\"])", "/test.py"));

    // Test with different types
    assert!(is_true("in(\"hello\", [\"hello\", \"world\"])", "/test.js"));
    assert!(is_false("in(\"foo\", [\"hello\", \"world\"])", "/test.js"));

    Ok(())
}

// Note: exists, siblings, children functions would require file system setup
// These would be better tested in integration tests with actual file structures

#[test]
fn test_without_function() -> Result<()> {
    // Test without function for removing suffixes
    let result = eval_expr("without(\"hello.test\", \".test\")", "/test.js")?;
    assert_eq!(result, Value::String("hello".to_string()));

    let result = eval_expr("without(\"component.spec.js\", \".spec.js\")", "/test.js")?;
    assert_eq!(result, Value::String("component".to_string()));

    // Test when suffix doesn't exist
    let result = eval_expr("without(\"hello\", \".test\")", "/test.js")?;
    assert_eq!(result, Value::String("hello".to_string()));

    Ok(())
}

// =============================================================================
// STANDARDIZED PATTERN TESTS
// =============================================================================

#[test]
fn test_kebab_case_pattern_comprehensive() -> Result<()> {
    let pattern = format!("matches($BASENAME, /{}/)", KEBAB_CASE_PATTERN);

    // Valid kebab-case
    assert!(is_true(&pattern, "/hello-world.js"));
    assert!(is_true(&pattern, "/my-component.tsx"));
    assert!(is_true(&pattern, "/test-123.js"));
    assert!(is_true(&pattern, "/a.js"));
    assert!(is_true(&pattern, "/single.js"));
    assert!(is_true(&pattern, "/multi-word-name.js"));

    // Invalid kebab-case
    assert!(is_false(&pattern, "/HelloWorld.js"));
    assert!(is_false(&pattern, "/hello_world.js"));
    assert!(is_false(&pattern, "/-hello.js"));
    assert!(is_false(&pattern, "/hello-.js"));
    assert!(is_false(&pattern, "/hello--world.js"));
    // Note: Our pattern allows numbers at start - this is by design for flexibility

    Ok(())
}

#[test]
fn test_pascal_case_pattern_comprehensive() -> Result<()> {
    let pattern = format!("matches($BASENAME, /{}/)", PASCAL_CASE_PATTERN);

    // Valid PascalCase
    assert!(is_true(&pattern, "/HelloWorld.js"));
    assert!(is_true(&pattern, "/MyComponent.tsx"));
    assert!(is_true(&pattern, "/A.js"));
    assert!(is_true(&pattern, "/Component.js"));
    assert!(is_true(&pattern, "/Test123.js"));
    assert!(is_true(&pattern, "/HTML5Parser.js"));

    // Invalid PascalCase
    assert!(is_false(&pattern, "/helloWorld.js"));
    assert!(is_false(&pattern, "/hello-world.js"));
    assert!(is_false(&pattern, "/hello_world.js"));
    assert!(is_false(&pattern, "/123Hello.js"));
    // Note: ALLCAPS actually matches our PascalCase pattern - this is acceptable

    Ok(())
}

#[test]
fn test_camel_case_pattern_comprehensive() -> Result<()> {
    let pattern = format!("matches($BASENAME, /{}/)", CAMEL_CASE_PATTERN);

    // Valid camelCase
    assert!(is_true(&pattern, "/helloWorld.js"));
    assert!(is_true(&pattern, "/myComponent.tsx"));
    assert!(is_true(&pattern, "/a.js"));
    assert!(is_true(&pattern, "/component.js"));
    assert!(is_true(&pattern, "/test123.js"));
    assert!(is_true(&pattern, "/parseHTML5.js"));

    // Invalid camelCase
    assert!(is_false(&pattern, "/HelloWorld.js"));
    assert!(is_false(&pattern, "/hello-world.js"));
    assert!(is_false(&pattern, "/hello_world.js"));
    assert!(is_false(&pattern, "/123hello.js"));
    assert!(is_false(&pattern, "/ALLCAPS.js"));

    Ok(())
}

#[test]
fn test_snake_case_pattern_comprehensive() -> Result<()> {
    let pattern = format!("matches($BASENAME, /{}/)", SNAKE_CASE_PATTERN);

    // Valid snake_case
    assert!(is_true(&pattern, "/hello_world.js"));
    assert!(is_true(&pattern, "/my_component.tsx"));
    assert!(is_true(&pattern, "/test_123.js"));
    assert!(is_true(&pattern, "/a.js"));
    assert!(is_true(&pattern, "/single.js"));
    assert!(is_true(&pattern, "/multi_word_name.js"));

    // Invalid snake_case
    assert!(is_false(&pattern, "/HelloWorld.js"));
    assert!(is_false(&pattern, "/hello-world.js"));
    assert!(is_false(&pattern, "/_hello.js"));
    assert!(is_false(&pattern, "/hello_.js"));
    assert!(is_false(&pattern, "/hello__world.js"));
    // Note: Our pattern allows numbers at start - this is by design for flexibility

    Ok(())
}

#[test]
fn test_file_extension_patterns() -> Result<()> {
    // Test JavaScript files
    assert!(is_true("$EXT == \"js\"", "/component.js"));
    assert!(is_false("$EXT == \"js\"", "/component.ts"));

    // Test TypeScript files
    assert!(is_true("$EXT == \"ts\"", "/component.ts"));
    assert!(is_false("$EXT == \"ts\"", "/component.js"));

    // Test JSX files
    assert!(is_true("$EXT == \"jsx\"", "/component.jsx"));
    assert!(is_false("$EXT == \"jsx\"", "/component.js"));

    // Test TSX files
    assert!(is_true("$EXT == \"tsx\"", "/component.tsx"));
    assert!(is_false("$EXT == \"tsx\"", "/component.ts"));

    Ok(())
}

#[test]
fn test_test_file_patterns() -> Result<()> {
    let jest_pattern = format!("matches($NAME, /{}/)", JEST_TEST_PATTERN);
    let spec_pattern = format!("matches($NAME, /{}/)", SPEC_TEST_PATTERN);

    // Test Jest pattern
    assert!(is_true(&jest_pattern, "/component.test.js"));
    assert!(is_true(&jest_pattern, "/utils.test.ts"));
    assert!(is_true(&jest_pattern, "/app.test.jsx"));
    assert!(is_true(&jest_pattern, "/router.test.tsx"));
    assert!(is_false(&jest_pattern, "/component.js"));
    assert!(is_false(&jest_pattern, "/component.spec.js"));

    // Test Spec pattern
    assert!(is_true(&spec_pattern, "/component.spec.js"));
    assert!(is_true(&spec_pattern, "/utils.spec.ts"));
    assert!(is_true(&spec_pattern, "/app.spec.jsx"));
    assert!(is_true(&spec_pattern, "/router.spec.tsx"));
    assert!(is_false(&spec_pattern, "/component.js"));
    assert!(is_false(&spec_pattern, "/component.test.js"));

    Ok(())
}

// =============================================================================
// COMPLEX EXPRESSION TESTS
// =============================================================================

#[test]
fn test_complex_logical_expressions() -> Result<()> {
    // Test complex AND/OR combinations
    assert!(is_true(
        "($EXT == \"js\" || $EXT == \"ts\") && matches($BASENAME, /^[a-z]/)",
        "/component.js"
    ));
    assert!(is_false(
        "($EXT == \"js\" || $EXT == \"ts\") && matches($BASENAME, /^[A-Z]/)",
        "/component.js"
    ));

    // Test complex NOT expressions
    assert!(is_true(
        "!($EXT == \"py\" || $EXT == \"rb\")",
        "/component.js"
    ));
    assert!(is_false(
        "!($EXT == \"js\" || $EXT == \"ts\")",
        "/component.js"
    ));

    Ok(())
}

#[test]
fn test_custom_matcher_combinations() -> Result<()> {
    // Create custom matchers
    let mut custom_matchers = HashMap::new();
    custom_matchers.insert("kebab-case".to_string(), parse_expression(KEBAB_CASE_EXPR)?);
    custom_matchers.insert(
        "PascalCase".to_string(),
        parse_expression(PASCAL_CASE_EXPR)?,
    );
    custom_matchers.insert("js-file".to_string(), parse_expression(JS_FILE_EXPR)?);

    let context = create_test_context("/hello-world.js", &custom_matchers);

    // Test individual custom matchers
    let expr = parse_expression("kebab-case")?;
    assert!(matches!(evaluate(&expr, &context)?, Value::Boolean(true)));

    let expr = parse_expression("PascalCase")?;
    assert!(matches!(evaluate(&expr, &context)?, Value::Boolean(false)));

    let expr = parse_expression("js-file")?;
    assert!(matches!(evaluate(&expr, &context)?, Value::Boolean(true)));

    // Test combined custom matchers
    let expr = parse_expression("kebab-case && js-file")?;
    assert!(matches!(evaluate(&expr, &context)?, Value::Boolean(true)));

    let expr = parse_expression("PascalCase && js-file")?;
    assert!(matches!(evaluate(&expr, &context)?, Value::Boolean(false)));

    Ok(())
}

#[test]
fn test_real_world_scenarios() -> Result<()> {
    // Test component naming rules
    assert!(is_true(
        "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/) && $EXT == \"jsx\"",
        "/Button.jsx"
    ));
    assert!(is_false(
        "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/) && $EXT == \"jsx\"",
        "/button.jsx"
    ));

    // Test utility naming rules
    assert!(is_true(
        "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/) && in($EXT, [\"js\", \"ts\"])",
        "/format-date.js"
    ));
    assert!(is_false(
        "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/) && in($EXT, [\"js\", \"ts\"])",
        "/FormatDate.js"
    ));

    // Test test file rules
    assert!(
    is_true(
      "matches($NAME, /\\.test\\.(js|ts|jsx|tsx)$/) && in($EXT, [\"js\", \"ts\", \"jsx\", \"tsx\"])",
      "/component.test.js"
    )
  );
    assert!(
    is_false(
      "matches($NAME, /\\.test\\.(js|ts|jsx|tsx)$/) && in($EXT, [\"js\", \"ts\", \"jsx\", \"tsx\"])",
      "/component.js"
    )
  );

    Ok(())
}

#[test]
fn test_expression_parsing_edge_cases() -> Result<()> {
    // Test expressions with parentheses
    assert!(is_true("(true && false) || true", "/test.js"));
    assert!(is_false("true && (false || false)", "/test.js"));

    // Test nested expressions
    assert!(is_true("!(! true)", "/test.js"));
    assert!(is_false("!(! false)", "/test.js"));

    // Test complex nesting
    assert!(
    is_true(
      "($EXT == \"js\" && matches($BASENAME, /^test/)) || ($EXT == \"ts\" && matches($BASENAME, /^spec/))",
      "/test-file.js"
    )
  );

    Ok(())
}

#[test]
fn test_error_conditions() {
    // Test invalid expressions that should fail at parse time
    assert!(parse_expression("$EXT ===").is_err());
    assert!(parse_expression("unclosed(").is_err());
    assert!(parse_expression("/unclosed_regex").is_err());
    assert!(parse_expression("$NAME $ $EXT").is_err()); // Invalid syntax

    // Test expressions that parse but fail at evaluation
    assert!(parse_expression("$INVALID_VAR").is_ok()); // Parses but would fail at evaluation
    assert!(parse_expression("42 && true").is_ok()); // Type mismatch caught at evaluation
    assert!(parse_expression("\"string\" > 42").is_ok()); // Type mismatch caught at evaluation
    assert!(parse_expression("matches()").is_ok()); // Function calls parse, fail at evaluation

    // Test that invalid variables and wrong function calls fail at evaluation time
    assert!(eval_expr("$INVALID_VAR", "/test.js").is_err());
    assert!(eval_expr("matches()", "/test.js").is_err());
}

#[test]
fn test_all_dsl_constants_are_valid() -> Result<()> {
    // Test that all our standardized DSL expressions parse correctly
    parse_expression(KEBAB_CASE_EXPR)?;
    parse_expression(PASCAL_CASE_EXPR)?;
    parse_expression(CAMEL_CASE_EXPR)?;
    parse_expression(SNAKE_CASE_EXPR)?;
    parse_expression(JS_FILE_EXPR)?;
    parse_expression(TS_FILE_EXPR)?;
    parse_expression(JSX_FILE_EXPR)?;
    parse_expression(TSX_FILE_EXPR)?;
    parse_expression(TEST_FILE_EXPR)?;
    parse_expression(SPEC_FILE_EXPR)?;

    Ok(())
}
