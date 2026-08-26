//! Tests for the operators and value handling in the evaluator: comparisons,
//! indexing, and how a `Value` compares and prints.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use lintp::dsl::evaluator::{evaluate, EvaluationContext, Value};
use lintp::dsl::parser::parse_expression;

fn context(matchers: &HashMap<String, Expr>) -> EvaluationContext<'_> {
    let mut variables = HashMap::new();
    variables.insert("NUM".to_string(), Value::Integer(5));
    variables.insert("SMALL".to_string(), Value::Integer(2));
    variables.insert("NAME".to_string(), Value::String("beta".to_string()));
    variables.insert("EARLIER".to_string(), Value::String("alpha".to_string()));
    variables.insert("FLAG".to_string(), Value::Boolean(true));

    EvaluationContext {
        variables,
        path: Path::new("/tmp/test.js"),
        custom_matchers: matchers,
        item_context: None,
        fs_cache: None,
        regex_cache: None,
    }
}

type Expr = lintp::dsl::ast::Expression;

fn eval(src: &str) -> Result<Value, lintp::Error> {
    let matchers = HashMap::new();
    let expr = parse_expression(src)?;

    evaluate(&expr, &context(&matchers))
}

/// Evaluate `src` expecting it to fail, and return the message.
fn eval_err(src: &str) -> String {
    match eval(src) {
        Ok(value) => panic!("expected an error, got {value:?}"),
        Err(e) => format!("{e:#}"),
    }
}

fn is_true(v: &Value) -> bool {
    matches!(v, Value::Boolean(true))
}

#[test]
fn compares_integers_with_every_ordering_operator() -> Result<()> {
    assert!(is_true(&eval("$SMALL < $NUM")?));
    assert!(is_true(&eval("$NUM > $SMALL")?));
    assert!(is_true(&eval("$SMALL <= $NUM")?));
    assert!(is_true(&eval("$NUM >= $SMALL")?));
    assert!(is_true(&eval("$NUM <= $NUM")?));
    assert!(is_true(&eval("$NUM >= $NUM")?));

    assert!(!is_true(&eval("$NUM < $SMALL")?));
    assert!(!is_true(&eval("$SMALL > $NUM")?));

    Ok(())
}

/// Strings order lexicographically, so a rule can compare names directly.
#[test]
fn compares_strings_with_every_ordering_operator() -> Result<()> {
    assert!(is_true(&eval("$EARLIER < $NAME")?));
    assert!(is_true(&eval("$NAME > $EARLIER")?));
    assert!(is_true(&eval("$EARLIER <= $NAME")?));
    assert!(is_true(&eval("$NAME >= $EARLIER")?));

    Ok(())
}

#[test]
fn ordering_operators_reject_mismatched_types() {
    assert!(eval_err("$NUM < $NAME").contains("Less than operator"));
    assert!(eval_err("$NUM > $NAME").contains("Greater than operator"));
    assert!(eval_err("$NUM <= $NAME").contains("Less than or equal operator"));
    assert!(eval_err("$NUM >= $NAME").contains("Greater than or equal operator"));
}

#[test]
fn equality_works_across_value_kinds() -> Result<()> {
    assert!(is_true(&eval("$NUM == 5")?));
    assert!(is_true(&eval("$NAME == \"beta\"")?));
    assert!(is_true(&eval("$FLAG == true")?));
    assert!(is_true(&eval("$NUM != 4")?));
    // Different kinds are never equal rather than being an error
    assert!(is_true(&eval("$NUM != $NAME")?));

    Ok(())
}

#[test]
fn boolean_operators_reject_non_boolean_operands() {
    assert!(eval_err("$NUM && $FLAG").contains("AND operator requires boolean"));
    assert!(eval_err("$NUM || $FLAG").contains("OR operator requires boolean"));
    assert!(eval_err("!$NUM").contains("NOT operator requires a boolean"));
}

#[test]
fn negates_integers_and_rejects_anything_else() -> Result<()> {
    assert_eq!(eval("-$NUM")?, Value::Integer(-5));
    assert!(eval_err("-$NAME").contains("Minus operator requires an integer"));

    Ok(())
}

#[test]
fn indexes_lists_and_strings() -> Result<()> {
    assert_eq!(eval("[\"a\", \"b\"][1]")?, Value::String("b".to_string()));
    assert_eq!(eval("\"abc\"[0]")?, Value::String("a".to_string()));

    Ok(())
}

/// A negative index must not wrap around into a huge unsigned value and read
/// past the end of the list.
#[test]
fn rejects_out_of_range_indexes() {
    assert!(eval_err("[\"a\"][5]").contains("Index out of bounds"));
    assert!(eval_err("[\"a\"][-1]").contains("Index out of bounds"));
    assert!(eval_err("\"abc\"[9]").contains("Index out of bounds"));
    assert!(eval_err("\"abc\"[-1]").contains("Index out of bounds"));
}

#[test]
fn rejects_indexing_something_that_is_not_indexable() {
    assert!(eval_err("$NUM[0]").contains("Cannot index into"));
}

/// Indexing counts characters, so a multi-byte name indexes the same way it
/// reads.
#[test]
fn indexes_strings_by_character_not_byte() -> Result<()> {
    assert_eq!(eval("\"café\"[3]")?, Value::String("é".to_string()));

    Ok(())
}

#[test]
fn values_print_in_their_source_form() -> Result<()> {
    assert_eq!(Value::String("x".to_string()).to_string(), "x");
    assert_eq!(Value::Integer(-3).to_string(), "-3");
    assert_eq!(Value::Boolean(false).to_string(), "false");
    assert_eq!(
        Value::List(vec![Value::Integer(1), Value::String("a".to_string())]).to_string(),
        "[1, a]"
    );

    let regex = eval("/^a+$/")?;
    assert_eq!(regex.to_string(), "/^a+$/");

    Ok(())
}

#[test]
fn regexes_and_lists_compare_by_value() -> Result<()> {
    let a = eval("/^x$/")?;
    let b = eval("/^x$/")?;
    let c = eval("/^y$/")?;
    assert_eq!(a, b);
    assert_ne!(a, c);

    assert_eq!(eval("[1, 2]")?, eval("[1, 2]")?);
    assert_ne!(eval("[1, 2]")?, eval("[1, 3]")?);
    assert_ne!(eval("[1]")?, eval("1")?);

    Ok(())
}
