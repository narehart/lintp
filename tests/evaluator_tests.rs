use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use lintp::dsl::ast::{BinaryOperator, Expression, UnaryOperator};
use lintp::dsl::evaluator::{evaluate, EvaluationContext, Value};
use lintp::dsl::parser::parse_expression;

/// Helper function to create a basic evaluation context for testing
fn create_test_context<'a>(
    path: &'a Path,
    custom_matchers: &'a HashMap<String, Expression>,
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
        fs_cache: None,
    }
}

/// Tests for evaluating literals
#[test]
fn test_evaluate_literals() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // String literal
    let expr = Expression::StringLiteral("hello".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::String(s) if s == "hello"));

    // Integer literal
    let expr = Expression::IntegerLiteral(42);
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Integer(n) if n == 42));

    // Boolean literal
    let expr = Expression::BooleanLiteral(true);
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Regex literal
    let expr = Expression::RegexLiteral("^test$".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Regex(_)));

    // List literal
    let expr = Expression::ListLiteral(vec![
        Expression::StringLiteral("a".to_string()),
        Expression::StringLiteral("b".to_string()),
    ]);
    let result = evaluate(&expr, &context)?;
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], Value::String(s) if s == "a"));
        assert!(matches!(&items[1], Value::String(s) if s == "b"));
    } else {
        panic!("Expected List value");
    }

    Ok(())
}

/// Tests for evaluating variables
#[test]
fn test_evaluate_variables() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // Existing variable
    let expr = Expression::Variable("NAME".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::String(s) if s == "test.js"));

    // Existing path variable
    let expr = Expression::Variable("PATH".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Path(p) if p == Path::new("/tmp/test.js")));

    // Variable in item context
    // Clone the context to avoid the borrow issue
    let mut item_context = create_test_context(path, &custom_matchers);
    item_context.item_context = Some(Value::String("item-value".to_string()));

    let expr = Expression::Variable("item".to_string());
    let result = evaluate(&expr, &item_context)?;
    assert!(matches!(result, Value::String(s) if s == "item-value"));

    // Non-existent variable
    let expr = Expression::Variable("NONEXISTENT".to_string());
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    Ok(())
}

/// Tests for evaluating binary operations
#[test]
fn test_evaluate_binary_ops() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // AND operation - both true
    let expr = Expression::BinaryOp {
        op: BinaryOperator::And,
        left: Box::new(Expression::BooleanLiteral(true)),
        right: Box::new(Expression::BooleanLiteral(true)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // AND operation - one false
    let expr = Expression::BinaryOp {
        op: BinaryOperator::And,
        left: Box::new(Expression::BooleanLiteral(true)),
        right: Box::new(Expression::BooleanLiteral(false)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(false)));

    // OR operation - one true
    let expr = Expression::BinaryOp {
        op: BinaryOperator::Or,
        left: Box::new(Expression::BooleanLiteral(false)),
        right: Box::new(Expression::BooleanLiteral(true)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // OR operation - both false
    let expr = Expression::BinaryOp {
        op: BinaryOperator::Or,
        left: Box::new(Expression::BooleanLiteral(false)),
        right: Box::new(Expression::BooleanLiteral(false)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(false)));

    // Equal operation - equal
    let expr = Expression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(Expression::StringLiteral("test".to_string())),
        right: Box::new(Expression::StringLiteral("test".to_string())),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Equal operation - not equal
    let expr = Expression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(Expression::StringLiteral("test".to_string())),
        right: Box::new(Expression::StringLiteral("other".to_string())),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(false)));

    // Not equal operation
    let expr = Expression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(Expression::StringLiteral("test".to_string())),
        right: Box::new(Expression::StringLiteral("other".to_string())),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Short-circuit evaluation of AND
    let expr = Expression::BinaryOp {
        op: BinaryOperator::And,
        left: Box::new(Expression::BooleanLiteral(false)),
        right: Box::new(Expression::Variable("NONEXISTENT".to_string())), // Would cause error if evaluated
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(false)));

    // Short-circuit evaluation of OR
    let expr = Expression::BinaryOp {
        op: BinaryOperator::Or,
        left: Box::new(Expression::BooleanLiteral(true)),
        right: Box::new(Expression::Variable("NONEXISTENT".to_string())), // Would cause error if evaluated
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    Ok(())
}

/// Tests for evaluating unary operations
#[test]
fn test_evaluate_unary_ops() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // NOT operation - true
    let expr = Expression::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(Expression::BooleanLiteral(true)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(false)));

    // NOT operation - false
    let expr = Expression::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(Expression::BooleanLiteral(false)),
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // NOT operation - error with non-boolean
    let expr = Expression::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(Expression::StringLiteral("test".to_string())),
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    Ok(())
}

/// Tests for evaluating function calls
#[test]
fn test_evaluate_function_calls() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // matches function with regex
    let expr = Expression::FunctionCall {
        name: "matches".to_string(),
        args: vec![
            Expression::Variable("NAME".to_string()),
            Expression::RegexLiteral("test\\.js".to_string()),
        ],
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // in function
    let expr = Expression::FunctionCall {
        name: "in".to_string(),
        args: vec![
            Expression::Variable("EXT".to_string()),
            Expression::ListLiteral(vec![
                Expression::StringLiteral("js".to_string()),
                Expression::StringLiteral("ts".to_string()),
            ]),
        ],
    };
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Unknown function
    let expr = Expression::FunctionCall {
        name: "unknown".to_string(),
        args: vec![],
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    Ok(())
}

/// Tests for evaluating references to custom matchers
#[test]
fn test_evaluate_references() -> Result<()> {
    let path = Path::new("/tmp/test.js");

    // Set up custom matchers
    let mut custom_matchers = HashMap::new();

    // Add a simple matcher
    custom_matchers.insert(
        "js-file".to_string(),
        Expression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Variable("EXT".to_string())),
            right: Box::new(Expression::StringLiteral("js".to_string())),
        },
    );

    // Add a matcher that references another matcher
    custom_matchers.insert(
        "test-js-file".to_string(),
        Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Reference("js-file".to_string())),
            right: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Variable("BASENAME".to_string())),
                right: Box::new(Expression::StringLiteral("test".to_string())),
            }),
        },
    );

    let context = create_test_context(path, &custom_matchers);

    // Reference to simple matcher
    let expr = Expression::Reference("js-file".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Reference to matcher with nested reference
    let expr = Expression::Reference("test-js-file".to_string());
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Reference to non-existent matcher
    let expr = Expression::Reference("nonexistent".to_string());
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    Ok(())
}

/// Tests for evaluating complex expressions
#[test]
fn test_evaluate_complex_expressions() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let mut custom_matchers = HashMap::new();

    // Add some custom matchers
    custom_matchers.insert("js-file".to_string(), parse_expression("$EXT == \"js\"")?);

    custom_matchers.insert(
        "test-file".to_string(),
        parse_expression("matches($NAME, /^test\\..+$/)")?, // Added ? operator here
    );

    let context = create_test_context(path, &custom_matchers);

    // Complex expression with functions, references, and operators
    let expr = parse_expression("js-file && test-file && !matches($NAME, /temp/)")?;
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Test some deeply nested expressions
    let expr =
        parse_expression("(js-file || $EXT == \"ts\") && (test-file || matches($NAME, /demo/))")?;
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    Ok(())
}

/// Tests for evaluating with item context (for map, filter, any, all)
#[test]
fn test_evaluate_with_item_context() -> Result<()> {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let mut context = create_test_context(path, &custom_matchers);

    // Set up item context
    context.item_context = Some(Value::String("item-value.js".to_string()));

    // Reference $item in expression
    let expr = parse_expression("matches($item, /.*\\.js$/)")?;
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Use complex expression with $item
    let expr = parse_expression("$item == \"item-value.js\"")?;
    let result = evaluate(&expr, &context)?;
    assert!(matches!(result, Value::Boolean(true)));

    // Test error when item context is missing
    let mut context2 = create_test_context(path, &custom_matchers); // Create a new context
    context2.item_context = None;
    let expr = parse_expression("$item == \"item-value.js\"")?;
    let result = evaluate(&expr, &context2);
    assert!(result.is_err());

    Ok(())
}

/// Tests for error handling in evaluation
#[test]
fn test_evaluate_errors() {
    let path = Path::new("/tmp/test.js");
    let custom_matchers = HashMap::new();
    let context = create_test_context(path, &custom_matchers);

    // Type error in binary operation
    let expr = Expression::BinaryOp {
        op: BinaryOperator::And,
        left: Box::new(Expression::StringLiteral("not a boolean".to_string())),
        right: Box::new(Expression::BooleanLiteral(true)),
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    // Type error in unary operation
    let expr = Expression::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(Expression::StringLiteral("not a boolean".to_string())),
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    // Error in function call - wrong number of arguments
    let expr = Expression::FunctionCall {
        name: "matches".to_string(),
        args: vec![
            Expression::Variable("NAME".to_string()), // Missing second argument
        ],
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());

    // Error in function call - wrong argument types
    let expr = Expression::FunctionCall {
        name: "matches".to_string(),
        args: vec![
            Expression::BooleanLiteral(true), // Not a string
            Expression::RegexLiteral("test".to_string()),
        ],
    };
    let result = evaluate(&expr, &context);
    assert!(result.is_err());
}
