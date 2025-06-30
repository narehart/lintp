use anyhow::{Context as ErrorContext, Result};
use glob::Pattern;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::ParsedConfig;
use crate::dsl::ast::Expression;
use crate::dsl::evaluator::{evaluate, EvaluationContext, Value};
use crate::dsl::parser::parse_expression;

#[derive(Debug)]
pub enum LintResult {
    Success(PathBuf),
    Failure {
        path: PathBuf,
        rule: String,
        message: String,
    },
}

pub fn run_lint(dir: &Path, config: &ParsedConfig, verbose: bool) -> Result<Vec<LintResult>> {
    let mut results = Vec::new();

    let ignore_patterns: Vec<Pattern> = config
        .raw
        .lintp
        .ignore
        .iter()
        .map(|pattern| Pattern::new(pattern))
        .collect::<Result<Vec<Pattern>, _>>()
        .with_context(|| "Failed to compile ignore patterns")?;

    // Process all files and directories
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            if let Ok(rel_path) = e.path().strip_prefix(dir) {
                !is_ignored(rel_path, &ignore_patterns)
            } else {
                true // If we can't get relative path, don't ignore
            }
        })
    {
        let entry = entry?;
        let path = entry.path();

        // Get relative path from the linted directory
        let rel_path = path.strip_prefix(dir)?;

        if verbose {
            println!("Checking: {}", rel_path.display());
        }

        // Find applicable rules for this path
        let applicable_rules = find_applicable_rules(rel_path, &config.raw.lintp.config)?;

        // Apply rules
        let name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?
            .to_string_lossy();

        let is_dir = path.is_dir();
        let result = apply_rules(
            &name,
            path,
            is_dir,
            &applicable_rules,
            &config.parsed_matchers,
        )?;

        results.push(result);
    }

    Ok(results)
}

fn is_ignored(path: &Path, ignore_patterns: &[Pattern]) -> bool {
    for pattern in ignore_patterns {
        if pattern.matches_path(path) {
            return true;
        }
    }

    false
}

fn find_applicable_rules(
    path: &Path,
    config: &crate::config::RuleConfig,
) -> Result<HashMap<String, String>> {
    let mut rules = config.global_rules.clone();

    // Find path-specific rules
    for (glob_pattern, pattern_rules) in &config.path_rules {
        let pattern = Pattern::new(glob_pattern)
            .with_context(|| format!("Invalid glob pattern: {}", glob_pattern))?;

        if pattern.matches_path(path) {
            // Merge pattern rules, overriding global rules
            for (key, value) in pattern_rules {
                rules.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(rules)
}

fn apply_rules(
    name: &str,
    path: &Path,
    is_dir: bool,
    rules: &HashMap<String, String>,
    custom_matchers: &HashMap<String, Expression>,
) -> Result<LintResult> {
    // Setup evaluation context
    let mut variables = HashMap::new();
    variables.insert("NAME".to_string(), Value::String(name.to_string()));
    variables.insert("PATH".to_string(), Value::Path(path.to_path_buf()));

    if let Some(parent) = path.parent() {
        variables.insert("PARENT".to_string(), Value::Path(parent.to_path_buf()));
    }

    if let Some(ext) = path.extension() {
        variables.insert(
            "EXT".to_string(),
            Value::String(ext.to_string_lossy().to_string()),
        );
    }

    let basename = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    variables.insert("BASENAME".to_string(), Value::String(basename));

    let context = EvaluationContext {
        variables,
        path,
        custom_matchers,
        item_context: None,
    };

    // Get rule to apply
    let rule_key = if is_dir {
        ".dir".to_string()
    } else {
        // Get file extension pattern
        let mut extension = String::new();
        if let Some(ext) = path.extension() {
            extension = format!(".{}", ext.to_string_lossy());
        }

        // Check for extensions with patterns like .d.ts or .stories.tsx
        let path_str = path.to_string_lossy();
        for key in rules.keys() {
            if key.starts_with('.') && path_str.ends_with(key) {
                extension = key.clone();
                break;
            }
        }

        // If no specific extension rule found, fallback to .*
        if !rules.contains_key(&extension) {
            extension = ".*".to_string();
        }

        extension
    };

    if let Some(rule_str) = rules.get(&rule_key) {
        // Parse the rule
        let rule_expr = parse_expression(rule_str)
            .with_context(|| format!("Failed to parse rule: {}", rule_str))?;

        // Evaluate the rule
        match evaluate(&rule_expr, &context)? {
            Value::Boolean(true) => Ok(LintResult::Success(path.to_path_buf())),
            Value::Boolean(false) => Ok(LintResult::Failure {
                path: path.to_path_buf(),
                rule: rule_key,
                message: format!("Does not match rule: {}", rule_str),
            }),
            _ => Ok(LintResult::Failure {
                path: path.to_path_buf(),
                rule: rule_key,
                message: format!("Rule did not evaluate to a boolean: {}", rule_str),
            }),
        }
    } else {
        // No rule found for this file/directory
        Ok(LintResult::Success(path.to_path_buf()))
    }
}
