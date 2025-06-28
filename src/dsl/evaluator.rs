use anyhow::{ Context as ErrorContext, Result };
use regex::Regex;
use std::collections::HashMap;
use std::path::{ Path, PathBuf };

use crate::dsl::ast::{ BinaryOperator, Expression, StringTemplatePart, UnaryOperator };
use crate::dsl::functions;

#[derive(Debug, Clone)]
pub enum Value {
  String(String),
  Integer(i64),
  Boolean(bool),
  Regex(Regex),
  List(Vec<Value>),
  Path(PathBuf),
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
      Value::Path(p) => write!(f, "{}", p.display()),
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
      (Value::Path(a), Value::Path(b)) => a == b,
      _ => false,
    }
  }
}

pub struct EvaluationContext<'a> {
  pub variables: HashMap<String, Value>,
  pub path: &'a Path,
  pub custom_matchers: &'a HashMap<String, Expression>,
  pub item_context: Option<Value>, // For map/filter operations
}

pub fn evaluate(expr: &Expression, context: &EvaluationContext) -> Result<Value> {
  match expr {
    Expression::Variable(name) => {
      if let Some(value) = context.variables.get(name) {
        Ok(value.clone())
      } else if name == "item" && context.item_context.is_some() {
        Ok(context.item_context.as_ref().unwrap().clone())
      } else {
        Err(anyhow::anyhow!("Unknown variable: {}", name))
      }
    }

    Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
    Expression::IntegerLiteral(i) => Ok(Value::Integer(*i)),
    Expression::BooleanLiteral(b) => Ok(Value::Boolean(*b)),

    Expression::RegexLiteral(pattern) => {
      let regex = Regex::new(pattern).with_context(||
        format!("Invalid regex pattern: {}", pattern)
      )?;
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
        BinaryOperator::And =>
          match (left_value, right_value) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l && r)),
            _ => Err(anyhow::anyhow!("AND operator requires boolean operands")),
          }
        BinaryOperator::Or =>
          match (left_value, right_value) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l || r)),
            _ => Err(anyhow::anyhow!("OR operator requires boolean operands")),
          }
        BinaryOperator::Equal => Ok(Value::Boolean(left_value == right_value)),
        BinaryOperator::NotEqual => Ok(Value::Boolean(left_value != right_value)),
        BinaryOperator::LessThan =>
          match (left_value, right_value) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l < r)),
            _ => Err(anyhow::anyhow!("Less than operator requires integer or string operands")),
          }
        BinaryOperator::GreaterThan =>
          match (left_value, right_value) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l > r)),
            _ => Err(anyhow::anyhow!("Greater than operator requires integer or string operands")),
          }
        BinaryOperator::LessThanOrEqual =>
          match (left_value, right_value) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l <= r)),
            _ =>
              Err(
                anyhow::anyhow!("Less than or equal operator requires integer or string operands")
              ),
          }
        BinaryOperator::GreaterThanOrEqual =>
          match (left_value, right_value) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l >= r)),
            _ =>
              Err(
                anyhow::anyhow!(
                  "Greater than or equal operator requires integer or string operands"
                )
              ),
          }
      }
    }

    Expression::UnaryOp { op, expr } => {
      let value = evaluate(expr, context)?;

      match op {
        UnaryOperator::Not =>
          match value {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(anyhow::anyhow!("NOT operator requires a boolean operand")),
          }
        UnaryOperator::Minus =>
          match value {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            _ => Err(anyhow::anyhow!("Minus operator requires an integer operand")),
          }
      }
    }

    Expression::FunctionCall { name, args } => {
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
            Err(anyhow::anyhow!("Index out of bounds: {} for list of length {}", i, items.len()))
          } else {
            Ok(items[i as usize].clone())
          }
        }
        (Value::String(s), Value::Integer(i)) => {
          let chars: Vec<char> = s.chars().collect();
          if i < 0 || (i as usize) >= chars.len() {
            Err(anyhow::anyhow!("Index out of bounds: {} for string of length {}", i, chars.len()))
          } else {
            Ok(Value::String(chars[i as usize].to_string()))
          }
        }
        _ => Err(anyhow::anyhow!("Cannot index into {:?} with {:?}", expr_clone, index_clone)),
      }
    }
  }
}
