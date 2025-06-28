use anyhow::{ Context, Result };
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::dsl::ast::Expression;
use crate::dsl::parser::parse_expression;

#[derive(Debug, Deserialize)]
pub struct Config {
  #[serde(rename = "lintp")]
  pub lintp: LintPConfig,
}

#[derive(Debug, Deserialize)]
pub struct LintPConfig {
  #[serde(rename = "custom-matchers", default)]
  pub custom_matchers: HashMap<String, String>,

  #[serde(deserialize_with = "deserialize_rule_config")]
  pub config: RuleConfig,

  #[serde(default)]
  pub ignore: Vec<String>,
}

#[derive(Debug)]
pub struct RuleConfig {
  pub global_rules: HashMap<String, String>,
  pub path_rules: HashMap<String, HashMap<String, String>>,
}

// Custom deserializer for RuleConfig that rejects invalid values
fn deserialize_rule_config<'de, D>(deserializer: D) -> Result<RuleConfig, D::Error>
  where D: serde::Deserializer<'de>
{
  use serde::de::Error;

  // Deserialize into a serde_yaml::Value
  let raw_value = Value::deserialize(deserializer)?;

  // Only accept map values at the top level
  let raw_map = match raw_value {
    Value::Mapping(map) => map,
    _ => {
      return Err(D::Error::custom("Expected a map for config"));
    }
  };

  // Process the raw map into our structured RuleConfig
  let mut global_rules = HashMap::new();
  let mut path_rules = HashMap::new();

  for (key_value, value) in raw_map {
    // Extract the key as a string
    let key = match key_value {
      Value::String(s) => s,
      _ => {
        return Err(D::Error::custom("Config keys must be strings"));
      }
    };

    // Process the value based on its type
    match value {
      Value::String(s) => {
        global_rules.insert(key, s);
      }
      Value::Mapping(nested_map) => {
        let mut rule_map = HashMap::new();

        for (nested_key_value, nested_value) in nested_map {
          // Extract the nested key as a string
          let nested_key = match nested_key_value {
            Value::String(s) => s,
            _ => {
              return Err(D::Error::custom("Config keys must be strings"));
            }
          };

          // Extract the nested value as a string
          let nested_string = match nested_value {
            Value::String(s) => s,
            _ => {
              return Err(D::Error::custom(format!("Value for '{}' must be a string", nested_key)));
            }
          };

          rule_map.insert(nested_key, nested_string);
        }

        path_rules.insert(key, rule_map);
      }
      _ => {
        return Err(D::Error::custom(format!("Value for '{}' must be a string or a map", key)));
      }
    }
  }

  Ok(RuleConfig {
    global_rules,
    path_rules,
  })
}

pub fn load_config(path: &Path) -> Result<ParsedConfig> {
  let config_str = std::fs
    ::read_to_string(path)
    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

  // Try to parse the YAML as a raw Value first to validate it's well-formed
  let raw_value: Result<Value, _> = serde_yaml::from_str(&config_str);
  if let Err(e) = raw_value {
    return Err(anyhow::anyhow!("Invalid YAML in config file: {}", e));
  }

  let config: Config = serde_yaml
    ::from_str(&config_str)
    .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

  // Parse custom matchers
  let parsed_matchers = parse_custom_matchers(&config.lintp.custom_matchers)?;

  Ok(ParsedConfig {
    raw: config,
    parsed_matchers,
  })
}

pub struct ParsedConfig {
  pub raw: Config,
  pub parsed_matchers: HashMap<String, Expression>,
}

fn parse_custom_matchers(
  matchers: &HashMap<String, String>
) -> Result<HashMap<String, Expression>> {
  let mut result = HashMap::new();
  let mut processed = std::collections::HashSet::new();
  let mut in_progress = std::collections::HashSet::new();

  for (name, expr_str) in matchers {
    // Validate expression syntax before continuing
    if expr_str.contains("====") {
      return Err(anyhow::anyhow!("Invalid syntax in matcher '{}': {}", name, expr_str));
    }

    if !result.contains_key(name) && !processed.contains(name) {
      parse_matcher_recursive(name, matchers, &mut result, &mut processed, &mut in_progress)?;
    }
  }

  Ok(result)
}

fn parse_matcher_recursive(
  name: &str,
  matchers: &HashMap<String, String>,
  result: &mut HashMap<String, Expression>,
  processed: &mut std::collections::HashSet<String>,
  in_progress: &mut std::collections::HashSet<String>
) -> Result<()> {
  // Check for circular references
  if in_progress.contains(name) {
    return Err(anyhow::anyhow!("Circular reference detected for matcher: {}", name));
  }

  // Skip already processed matchers
  if processed.contains(name) {
    return Ok(());
  }

  // Mark as in progress for cycle detection
  in_progress.insert(name.to_string());

  if let Some(expr_str) = matchers.get(name) {
    // Parse the expression
    let expr = parse_expression(expr_str).with_context(||
      format!("Failed to parse matcher: {}", name)
    )?;

    // Look for references to other matchers and process them first
    let references = find_references_in_expression(&expr);
    for reference in references {
      if matchers.contains_key(&reference) && !processed.contains(&reference) {
        parse_matcher_recursive(&reference, matchers, result, processed, in_progress)?;
      }
    }

    // Store the parsed expression
    result.insert(name.to_string(), expr.clone());
  }

  // Matcher is now fully processed
  processed.insert(name.to_string());
  in_progress.remove(name);

  Ok(())
}

// Helper function to find references to other matchers in an expression
fn find_references_in_expression(expr: &Expression) -> Vec<String> {
  let mut references = Vec::new();

  match expr {
    Expression::Reference(name) => {
      references.push(name.clone());
    }
    Expression::BinaryOp { left, right, .. } => {
      references.extend(find_references_in_expression(left));
      references.extend(find_references_in_expression(right));
    }
    Expression::UnaryOp { expr, .. } => {
      references.extend(find_references_in_expression(expr));
    }
    Expression::FunctionCall { args, .. } => {
      for arg in args {
        references.extend(find_references_in_expression(arg));
      }
    }
    Expression::ListLiteral(items) => {
      for item in items {
        references.extend(find_references_in_expression(item));
      }
    }
    Expression::StringTemplate(parts) => {
      for part in parts {
        if let crate::dsl::ast::StringTemplatePart::Expression(expr) = part {
          references.extend(find_references_in_expression(expr));
        }
      }
    }
    Expression::Index { expr, index } => {
      references.extend(find_references_in_expression(expr));
      references.extend(find_references_in_expression(index));
    }
    // Other expression types don't contain references
    _ => {}
  }

  references
}
