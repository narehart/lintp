use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::dsl::ast::Expression;
use crate::dsl::parser::parse_expression;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "lintp")]
    pub lintp: LintPConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintPConfig {
    #[serde(rename = "custom-matchers", default)]
    pub custom_matchers: HashMap<String, String>,

    #[serde(deserialize_with = "deserialize_rule_config")]
    pub config: RuleConfig,

    #[serde(default)]
    pub ignore: Vec<String>,
}

/// A rule expression plus an optional human-readable message shown instead
/// of the raw expression when the rule fails.
#[derive(Debug, Clone)]
pub struct RuleEntry {
    pub rule: String,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct RuleConfig {
    pub global_rules: HashMap<String, RuleEntry>,
    pub path_rules: HashMap<String, HashMap<String, RuleEntry>>,
}

/// Rule keys are extension patterns and must start with '.'; anything else
/// would never match a file (the rule-key lookup filters on the leading dot)
/// and so would be silently inert — the classic case being a path scope
/// mis-indented so its 'rule:' key turns the whole scope into a rule entry.
fn validate_rule_key<E: serde::de::Error>(key: &str, scope: Option<&str>) -> Result<(), E> {
    if key.starts_with('.') {
        return Ok(());
    }
    let location = match scope {
        Some(path) => format!(" under path scope '{}'", path),
        None => String::new(),
    };
    let hint = if key.contains('/') || key.contains('*') {
        format!(
            " — if '{}' is meant to scope rules to a path, nest extension rules under it (e.g. \"{}\": {{ .js: ... }})",
            key, key
        )
    } else {
        String::new()
    };
    Err(E::custom(format!(
        "Invalid rule key '{}'{}: rule keys are extension patterns starting with '.' (.js, .test.ts, .dir, .*){}",
        key, location, hint
    )))
}

/// Expand glob-style brace alternation: ".{png,jpg}" → [".png", ".jpg"],
/// "src/{a,b}/*" → ["src/a/*", "src/b/*"]. Multiple groups expand as a
/// cartesian product; nesting is not supported. Returns the input
/// unchanged when it contains no braces.
fn expand_braces<E: serde::de::Error>(key: &str) -> Result<Vec<String>, E> {
    let (Some(open), Some(close)) = (key.find('{'), key.find('}')) else {
        if key.contains('{') || key.contains('}') {
            return Err(E::custom(format!(
                "Invalid key '{}': unbalanced braces",
                key
            )));
        }
        return Ok(vec![key.to_string()]);
    };
    if close < open {
        return Err(E::custom(format!(
            "Invalid key '{}': unbalanced braces",
            key
        )));
    }
    let inner = &key[open + 1..close];
    if inner.contains('{') {
        return Err(E::custom(format!(
            "Invalid key '{}': nested braces are not supported",
            key
        )));
    }
    let (prefix, suffix) = (&key[..open], &key[close + 1..]);
    let mut out = Vec::new();
    for alt in inner.split(',') {
        let alt = alt.trim();
        if alt.is_empty() {
            return Err(E::custom(format!(
                "Invalid key '{}': empty alternative in braces",
                key
            )));
        }
        // suffix may contain further brace groups: expand recursively
        for rest in expand_braces::<E>(suffix)? {
            out.push(format!("{}{}{}", prefix, alt, rest));
        }
    }
    Ok(out)
}

/// A rule key may group suffixes with brace alternation
/// (".{png,jpg}: camelCase"); each expansion gets the same rule entry
/// and is validated like a standalone key.
fn expand_rule_keys<E: serde::de::Error>(key: &str, scope: Option<&str>) -> Result<Vec<String>, E> {
    let parts = expand_braces::<E>(key)?;
    for part in &parts {
        validate_rule_key::<E>(part, scope)?;
    }
    Ok(parts)
}

// Custom deserializer for RuleConfig that rejects invalid values
fn deserialize_rule_config<'de, D>(deserializer: D) -> Result<RuleConfig, D::Error>
where
    D: serde::Deserializer<'de>,
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
                for part in expand_rule_keys::<D::Error>(&key, None)? {
                    global_rules.insert(
                        part,
                        RuleEntry {
                            rule: s.clone(),
                            message: None,
                        },
                    );
                }
            }
            Value::Mapping(nested_map) => {
                // A mapping with a `rule` key is a rule with options
                // (`{rule: ..., message: ...}`); any other mapping is a
                // path-scoped block of extension rules.
                if nested_map.contains_key(Value::String("rule".to_string())) {
                    let entry = rule_entry_from_mapping::<D>(&key, nested_map)?;
                    for part in expand_rule_keys::<D::Error>(&key, None)? {
                        global_rules.insert(part, entry.clone());
                    }
                    continue;
                }

                // The key is a path scope. Braces expand first ("src/{a,b}/*"
                // becomes two scopes), then each glob must compile; an empty
                // scope is a mistake, not a no-op
                let scope_keys = expand_braces::<D::Error>(&key)?;
                for scope_key in &scope_keys {
                    if let Err(e) = glob::Pattern::new(scope_key) {
                        return Err(D::Error::custom(format!(
                            "Invalid glob pattern for path scope '{}': {}",
                            scope_key, e
                        )));
                    }
                }
                if nested_map.is_empty() {
                    return Err(D::Error::custom(format!(
                        "Path scope '{}' has no rules",
                        key
                    )));
                }

                let mut rule_map = HashMap::new();

                for (nested_key_value, nested_value) in nested_map {
                    // Extract the nested key as a string
                    let nested_key = match nested_key_value {
                        Value::String(s) => s,
                        _ => {
                            return Err(D::Error::custom("Config keys must be strings"));
                        }
                    };

                    // Extract the nested value: a rule string or {rule, message}
                    let entry = match nested_value {
                        Value::String(s) => RuleEntry {
                            rule: s,
                            message: None,
                        },
                        Value::Mapping(rule_map_value) => {
                            rule_entry_from_mapping::<D>(&nested_key, rule_map_value)?
                        }
                        _ => {
                            return Err(D::Error::custom(format!(
                                "Value for '{}' must be a string or a map",
                                nested_key
                            )));
                        }
                    };

                    for part in expand_rule_keys::<D::Error>(&nested_key, Some(&key))? {
                        rule_map.insert(part, entry.clone());
                    }
                }

                for scope_key in scope_keys {
                    path_rules.insert(scope_key, rule_map.clone());
                }
            }
            _ => {
                return Err(D::Error::custom(format!(
                    "Value for '{}' must be a string or a map",
                    key
                )));
            }
        }
    }

    Ok(RuleConfig {
        global_rules,
        path_rules,
    })
}

fn rule_entry_from_mapping<'de, D>(
    key: &str,
    mapping: serde_yaml::Mapping,
) -> Result<RuleEntry, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let mut rule = None;
    let mut message = None;

    for (option_key, option_value) in mapping {
        let option_name = match option_key {
            Value::String(s) => s,
            _ => {
                return Err(D::Error::custom("Config keys must be strings"));
            }
        };

        let option_string = match option_value {
            Value::String(s) => s,
            _ => {
                return Err(D::Error::custom(format!(
                    "'{}' for rule '{}' must be a string",
                    option_name, key
                )));
            }
        };

        match option_name.as_str() {
            "rule" => rule = Some(option_string),
            "message" => message = Some(option_string),
            other => {
                return Err(D::Error::custom(format!(
                    "Unknown option '{}' for rule '{}': expected 'rule' or 'message'",
                    other, key
                )));
            }
        }
    }

    let rule = rule
        .ok_or_else(|| D::Error::custom(format!("Rule '{}' is missing the 'rule' field", key)))?;

    Ok(RuleEntry { rule, message })
}

pub fn load_config(path: &Path) -> Result<ParsedConfig> {
    let config_str = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    // Try to parse the YAML as a raw Value first to validate it's well-formed
    let raw_value: Result<Value, _> = serde_yaml::from_str(&config_str);
    if let Err(e) = raw_value {
        return Err(anyhow::anyhow!("Invalid YAML in config file: {}", e));
    }

    let config: Config = serde_yaml::from_str(&config_str)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    // Ignore patterns are compiled at lint time; compile them here too so
    // a bad pattern fails at load with the config location
    for pattern in &config.lintp.ignore {
        glob::Pattern::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid ignore pattern '{}': {}", pattern, e))?;
    }

    // A matcher named after a boolean literal can never be referenced —
    // the parser resolves 'true'/'false' to booleans first
    for name in config.lintp.custom_matchers.keys() {
        if name == "true" || name == "false" {
            return Err(anyhow::anyhow!(
                "Invalid matcher name '{}': shadowed by the boolean literal",
                name
            ));
        }
    }

    // Parse custom matchers
    let parsed_matchers = parse_custom_matchers(&config.lintp.custom_matchers)?;

    // Parse every rule up front so config errors surface at startup
    // instead of when a matching file first appears
    let parsed_rules = parse_rules(&config, &parsed_matchers)?;

    Ok(ParsedConfig {
        raw: config,
        parsed_matchers,
        parsed_rules,
    })
}

pub struct ParsedConfig {
    pub raw: Config,
    pub parsed_matchers: HashMap<String, Expression>,
    /// Every distinct rule string, pre-parsed. Populated by load_config;
    /// rules missing from this map are parsed lazily during linting.
    pub parsed_rules: HashMap<String, Expression>,
}

/// Eagerly parse all global and path-scoped rules and check that every
/// matcher reference resolves, so typos fail at load time with the rule
/// location instead of surfacing mid-lint.
fn parse_rules(
    config: &Config,
    matchers: &HashMap<String, Expression>,
) -> Result<HashMap<String, Expression>> {
    let mut parsed = HashMap::new();

    let global = config
        .lintp
        .config
        .global_rules
        .iter()
        .map(|(key, entry)| (format!("rule '{}'", key), &entry.rule));
    let scoped = config
        .lintp
        .config
        .path_rules
        .iter()
        .flat_map(|(path, rules)| {
            rules
                .iter()
                .map(move |(key, entry)| (format!("rule '{}' under '{}'", key, path), &entry.rule))
        });

    for (location, rule_str) in global.chain(scoped) {
        if parsed.contains_key(rule_str) {
            continue;
        }

        let expr = parse_expression(rule_str)
            .with_context(|| format!("Failed to parse {}: {}", location, rule_str))?;

        for reference in find_references_in_expression(&expr) {
            if !matchers.contains_key(&reference) {
                return Err(anyhow::anyhow!(
                    "Unknown matcher '{}' referenced by {}: {}",
                    reference,
                    location,
                    rule_str
                ));
            }
        }

        parsed.insert(rule_str.clone(), expr);
    }

    Ok(parsed)
}

fn parse_custom_matchers(
    matchers: &HashMap<String, String>,
) -> Result<HashMap<String, Expression>> {
    let mut result = HashMap::new();
    let mut processed = std::collections::HashSet::new();
    let mut in_progress = std::collections::HashSet::new();

    for (name, expr_str) in matchers {
        // Validate expression syntax before continuing
        if expr_str.contains("====") {
            return Err(anyhow::anyhow!(
                "Invalid syntax in matcher '{}': {}",
                name,
                expr_str
            ));
        }

        if !result.contains_key(name) && !processed.contains(name) {
            parse_matcher_recursive(
                name,
                matchers,
                &mut result,
                &mut processed,
                &mut in_progress,
            )?;
        }
    }

    Ok(result)
}

fn parse_matcher_recursive(
    name: &str,
    matchers: &HashMap<String, String>,
    result: &mut HashMap<String, Expression>,
    processed: &mut std::collections::HashSet<String>,
    in_progress: &mut std::collections::HashSet<String>,
) -> Result<()> {
    // Check for circular references
    if in_progress.contains(name) {
        return Err(anyhow::anyhow!(
            "Circular reference detected for matcher: {}",
            name
        ));
    }

    // Skip already processed matchers
    if processed.contains(name) {
        return Ok(());
    }

    // Mark as in progress for cycle detection
    in_progress.insert(name.to_string());

    if let Some(expr_str) = matchers.get(name) {
        // Parse the expression
        let expr = parse_expression(expr_str)
            .with_context(|| format!("Failed to parse matcher: {}", name))?;

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
