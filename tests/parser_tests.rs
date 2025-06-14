use anyhow::Result;
use lintp::dsl::ast::{ BinaryOperator, Expression, UnaryOperator };
use lintp::dsl::parser::parse_expression;

/// Tests for parsing basic literals
#[test]
fn test_parse_literals() -> Result<()> {
  // String literals
  let expr = parse_expression("\"hello world\"")?;
  assert!(matches!(expr, Expression::StringLiteral(s) if s == "hello world"));

  let expr = parse_expression("'hello world'")?;
  assert!(matches!(expr, Expression::StringLiteral(s) if s == "hello world"));

  // Integer literals
  let expr = parse_expression("42")?;
  assert!(matches!(expr, Expression::IntegerLiteral(n) if n == 42));

  let expr = parse_expression("-42")?;
  // -42 is parsed as a unary minus operation, not as a negative literal
  if let Expression::UnaryOp { op: UnaryOperator::Minus, expr: inner } = expr {
    assert!(matches!(inner.as_ref(), Expression::IntegerLiteral(n) if *n == 42));
  } else {
    panic!("Expected UnaryOp with Minus");
  }

  // Boolean literals
  let expr = parse_expression("true")?;
  assert!(matches!(expr, Expression::BooleanLiteral(true)));

  let expr = parse_expression("false")?;
  assert!(matches!(expr, Expression::BooleanLiteral(false)));

  // Regex literals
  let expr = parse_expression("/^test-[0-9]+$/")?;
  assert!(matches!(expr, Expression::RegexLiteral(pattern) if pattern == "^test-[0-9]+$"));

  // List literals
  let expr = parse_expression("[]")?;
  assert!(matches!(expr, Expression::ListLiteral(items) if items.is_empty()));

  // List with items
  let expr = parse_expression("[\"a\", \"b\", 42]")?;
  if let Expression::ListLiteral(items) = expr {
    assert_eq!(items.len(), 3);
    assert!(matches!(&items[0], Expression::StringLiteral(s) if s == "a"));
    assert!(matches!(&items[1], Expression::StringLiteral(s) if s == "b"));
    assert!(matches!(&items[2], Expression::IntegerLiteral(n) if *n == 42));
  } else {
    panic!("Expected ListLiteral");
  }

  Ok(())
}

/// Tests for parsing variables
#[test]
fn test_parse_variables() -> Result<()> {
  let expr = parse_expression("$NAME")?;
  assert!(matches!(expr, Expression::Variable(name) if name == "NAME"));

  let expr = parse_expression("$PATH")?;
  assert!(matches!(expr, Expression::Variable(name) if name == "PATH"));

  let expr = parse_expression("$EXT")?;
  assert!(matches!(expr, Expression::Variable(name) if name == "EXT"));

  let expr = parse_expression("$PARENT")?;
  assert!(matches!(expr, Expression::Variable(name) if name == "PARENT"));

  Ok(())
}

/// Tests for parsing binary operations
#[test]
fn test_parse_binary_ops() -> Result<()> {
  // AND operation
  let expr = parse_expression("$NAME == \"test\" && $EXT == \"js\"")?;
  if let Expression::BinaryOp { op, left, right } = expr {
    assert_eq!(op, BinaryOperator::And);

    if let Expression::BinaryOp { op: left_op, .. } = *left {
      assert_eq!(left_op, BinaryOperator::Equal);
    } else {
      panic!("Expected BinaryOp for left operand");
    }

    if let Expression::BinaryOp { op: right_op, .. } = *right {
      assert_eq!(right_op, BinaryOperator::Equal);
    } else {
      panic!("Expected BinaryOp for right operand");
    }
  } else {
    panic!("Expected BinaryOp");
  }

  // OR operation
  let expr = parse_expression("true || false")?;
  if let Expression::BinaryOp { op, left, right } = expr {
    assert_eq!(op, BinaryOperator::Or);
    assert!(matches!(*left, Expression::BooleanLiteral(true)));
    assert!(matches!(*right, Expression::BooleanLiteral(false)));
  } else {
    panic!("Expected BinaryOp");
  }

  // Equal operation
  let expr = parse_expression("$NAME == \"test\"")?;
  if let Expression::BinaryOp { op, left, right } = expr {
    assert_eq!(op, BinaryOperator::Equal);
    assert!(matches!(*left, Expression::Variable(name) if name == "NAME"));
    assert!(matches!(*right, Expression::StringLiteral(s) if s == "test"));
  } else {
    panic!("Expected BinaryOp");
  }

  // Not equal operation
  let expr = parse_expression("$NAME != \"test\"")?;
  if let Expression::BinaryOp { op, .. } = expr {
    assert_eq!(op, BinaryOperator::NotEqual);
  } else {
    panic!("Expected BinaryOp");
  }

  // Multiple operators with precedence
  let expr = parse_expression("$NAME == \"test\" || $NAME == \"temp\" && $EXT == \"js\"")?;
  if let Expression::BinaryOp { op, .. } = expr {
    assert_eq!(op, BinaryOperator::Or);
  } else {
    panic!("Expected BinaryOp");
  }

  Ok(())
}

/// Tests for parsing unary operations
#[test]
fn test_parse_unary_ops() -> Result<()> {
  // NOT operation
  let expr = parse_expression("!$NAME == \"test\"")?;
  if let Expression::UnaryOp { op, expr: inner } = expr {
    assert_eq!(op, UnaryOperator::Not);
    assert!(matches!(*inner, Expression::BinaryOp { .. }));
  } else {
    panic!("Expected UnaryOp");
  }

  // NOT with parentheses
  let expr = parse_expression("!(true || false)")?;
  if let Expression::UnaryOp { op, expr: inner } = expr {
    assert_eq!(op, UnaryOperator::Not);
    assert!(matches!(*inner, Expression::BinaryOp { .. }));
  } else {
    panic!("Expected UnaryOp");
  }

  Ok(())
}

/// Tests for parsing function calls
#[test]
fn test_parse_function_calls() -> Result<()> {
  // Simple function call
  let expr = parse_expression("matches($NAME, /test/)")?;
  if let Expression::FunctionCall { name, args } = expr {
    assert_eq!(name, "matches");
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0], Expression::Variable(name) if name == "NAME"));
    assert!(matches!(&args[1], Expression::RegexLiteral(pattern) if pattern == "test"));
  } else {
    panic!("Expected FunctionCall");
  }

  // Function with no arguments
  let expr = parse_expression("isEmpty()")?;
  if let Expression::FunctionCall { name, args } = expr {
    assert_eq!(name, "isEmpty");
    assert_eq!(args.len(), 0);
  } else {
    panic!("Expected FunctionCall");
  }

  // Function with multiple arguments
  let expr = parse_expression("in($NAME, [\"a\", \"b\", \"c\"])")?;
  if let Expression::FunctionCall { name, args } = expr {
    assert_eq!(name, "in");
    assert_eq!(args.len(), 2);
  } else {
    panic!("Expected FunctionCall");
  }

  // Nested function calls
  let expr = parse_expression(
    "any(map(siblings(\"*.js\"), without($item, \".js\")), $NAME == $item)"
  )?;
  if let Expression::FunctionCall { name, args } = expr {
    assert_eq!(name, "any");
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0], Expression::FunctionCall { .. }));
  } else {
    panic!("Expected FunctionCall");
  }

  Ok(())
}

/// Tests for parsing references to custom matchers
#[test]
fn test_parse_references() -> Result<()> {
  let expr = parse_expression("kebab-case")?;
  assert!(matches!(expr, Expression::Reference(name) if name == "kebab-case"));

  let expr = parse_expression("PascalCase")?;
  assert!(matches!(expr, Expression::Reference(name) if name == "PascalCase"));

  Ok(())
}

/// Tests for parsing complex expressions with parentheses
#[test]
fn test_parse_complex_expressions() -> Result<()> {
  let expr = parse_expression(
    "(matches($NAME, /test/) || in($NAME, [\"a\", \"b\"])) && !matches($EXT, /tmp/)"
  )?;

  if let Expression::BinaryOp { op, left, right } = expr {
    assert_eq!(op, BinaryOperator::And);

    if let Expression::BinaryOp { op: left_op, .. } = *left {
      assert_eq!(left_op, BinaryOperator::Or);
    } else {
      panic!("Expected BinaryOp for left operand");
    }

    assert!(matches!(*right, Expression::UnaryOp { .. }));
  } else {
    panic!("Expected BinaryOp");
  }

  // Test with deeply nested expressions
  let expr = parse_expression(
    "(!(matches($NAME, /test/) || in($NAME, [\"a\", \"b\"]))) && ($EXT == \"js\" || $EXT == \"ts\")"
  )?;

  if let Expression::BinaryOp { op, .. } = expr {
    assert_eq!(op, BinaryOperator::And);
  } else {
    panic!("Expected BinaryOp");
  }

  Ok(())
}

/// Tests for parsing string templates
#[test]
fn test_parse_string_templates() -> Result<()> {
  let expr = parse_expression("${$NAME}")?;

  if let Expression::StringTemplate(parts) = expr {
    assert_eq!(parts.len(), 1);
  } else {
    panic!("Expected StringTemplate");
  }

  // String template in function call
  let expr = parse_expression("matches($NAME, ${children(\"*.pattern\")[0]})")?;

  if let Expression::FunctionCall { name, args } = expr {
    assert_eq!(name, "matches");
    assert_eq!(args.len(), 2);
  } else {
    panic!("Expected FunctionCall");
  }

  Ok(())
}

/// Tests for error handling of malformed expressions
#[test]
fn test_parse_errors() {
  // Unclosed string
  let result = parse_expression("\"unclosed string");
  assert!(result.is_err());

  // Invalid regex
  let result = parse_expression("/unclosed regex");
  assert!(result.is_err());

  // Unbalanced parentheses
  let result = parse_expression("(matches($NAME, /test/)");
  assert!(result.is_err());

  // Invalid binary operation
  let result = parse_expression("$NAME ++ \"test\"");
  assert!(result.is_err());

  // Invalid function call
  let result = parse_expression("matches($NAME, ");
  assert!(result.is_err());
}
