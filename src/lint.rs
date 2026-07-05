use anyhow::{Context as ErrorContext, Result};
use glob::Pattern;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{ParsedConfig, RuleEntry};
use crate::dsl::ast::{BinaryOperator, Expression};
use crate::dsl::evaluator::{evaluate, EvaluationContext, FsCache, Value};
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

    // Compile path-rule globs once instead of once per visited file
    let mut path_rule_patterns: Vec<(Pattern, &HashMap<String, RuleEntry>)> = config
        .raw
        .lintp
        .config
        .path_rules
        .iter()
        .map(|(glob_pattern, pattern_rules)| {
            Pattern::new(glob_pattern)
                .map(|pattern| (pattern, pattern_rules))
                .with_context(|| format!("Invalid glob pattern: {}", glob_pattern))
        })
        .collect::<Result<Vec<_>>>()?;
    // Overlapping scopes merge in order, later entries overwriting earlier
    // ones, so sort ascending by specificity (pattern length, then text):
    // when both src/* and src/ui/* match, src/ui/* wins — deterministically.
    // Map iteration order would make the winner random per run.
    path_rule_patterns.sort_by(|(a, _), (b, _)| {
        (a.as_str().len(), a.as_str()).cmp(&(b.as_str().len(), b.as_str()))
    });

    // Rules are pre-parsed at config load; anything else (e.g. configs
    // constructed programmatically) parses once here and is reused
    let mut rule_cache: HashMap<String, Expression> = config.parsed_rules.clone();

    // Glob results are cached across files: a siblings("*") rule evaluated
    // for every file in a directory reads that directory once, not O(n) times
    let fs_cache: FsCache = FsCache::default();

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
        let applicable_rules = find_applicable_rules(
            rel_path,
            &config.raw.lintp.config.global_rules,
            &path_rule_patterns,
        );

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
            &mut rule_cache,
            &fs_cache,
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
    global_rules: &HashMap<String, RuleEntry>,
    path_rule_patterns: &[(Pattern, &HashMap<String, RuleEntry>)],
) -> HashMap<String, RuleEntry> {
    let mut rules = global_rules.clone();

    // Find path-specific rules
    for (pattern, pattern_rules) in path_rule_patterns {
        if pattern.matches_path(path) {
            // Merge pattern rules, overriding global rules
            for (key, value) in *pattern_rules {
                rules.insert(key.clone(), value.clone());
            }
        }
    }

    rules
}

fn apply_rules(
    name: &str,
    path: &Path,
    is_dir: bool,
    rules: &HashMap<String, RuleEntry>,
    custom_matchers: &HashMap<String, Expression>,
    rule_cache: &mut HashMap<String, Expression>,
    fs_cache: &FsCache,
) -> Result<LintResult> {
    // Setup evaluation context
    let mut variables = HashMap::new();
    variables.insert("NAME".to_string(), Value::String(name.to_string()));
    // PATH and PARENT are exposed as strings: every documented usage is a
    // string operation (contains($PATH, "/src/"), $PARENT == "."), and the
    // string functions reject Path values
    variables.insert(
        "PATH".to_string(),
        Value::String(path.display().to_string()),
    );

    if let Some(parent) = path.parent() {
        variables.insert(
            "PARENT".to_string(),
            Value::String(parent.display().to_string()),
        );
    }

    // EXT is always present — empty for extensionless files — so a rule
    // like `$EXT == "js"` evaluates to false on LICENSE instead of
    // aborting the whole run with "Unknown variable: EXT"
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    variables.insert("EXT".to_string(), Value::String(ext));

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
        fs_cache: Some(fs_cache),
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

        // Check for extensions with patterns like .d.ts or .stories.tsx.
        // The longest matching suffix wins so `.test.tsx` beats `.tsx`;
        // picking by map iteration order would be nondeterministic.
        let path_str = path.to_string_lossy();
        if let Some(key) = rules
            .keys()
            .filter(|key| key.starts_with('.') && path_str.ends_with(key.as_str()))
            .max_by_key(|key| key.len())
        {
            extension = key.clone();
        }

        // If no specific extension rule found, fallback to .*
        if !rules.contains_key(&extension) {
            extension = ".*".to_string();
        }

        extension
    };

    if let Some(entry) = rules.get(&rule_key) {
        let rule_str = &entry.rule;

        // Parse the rule once per distinct rule string
        if !rule_cache.contains_key(rule_str) {
            let expr = parse_expression(rule_str)
                .with_context(|| format!("Failed to parse rule: {}", rule_str))?;
            rule_cache.insert(rule_str.clone(), expr);
        }
        let rule_expr = &rule_cache[rule_str];

        // Evaluate the rule
        match evaluate(rule_expr, &context)? {
            Value::Boolean(true) => Ok(LintResult::Success(path.to_path_buf())),
            Value::Boolean(false) => {
                // A configured message replaces the raw expression; the
                // failing-conjunct breakdown is appended either way
                let mut message = match &entry.message {
                    Some(custom) => custom.clone(),
                    None => format!("Does not match rule: {}", rule_str),
                };
                if let Some(failed) = explain_failure(rule_expr, &context) {
                    message.push_str(&format!(" (failed: {})", failed));
                }
                Ok(LintResult::Failure {
                    path: path.to_path_buf(),
                    rule: rule_key,
                    message,
                })
            }
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

/// When a rule is a chain of `&&` conjuncts, pinpoint the ones that failed
/// so the user doesn't have to bisect a composed rule by hand. Returns None
/// when the rule has no conjunction to decompose (the whole rule failed).
fn explain_failure(rule_expr: &Expression, context: &EvaluationContext) -> Option<String> {
    let mut conjuncts = Vec::new();
    collect_conjuncts(rule_expr, &mut conjuncts);

    if conjuncts.len() < 2 {
        return None;
    }

    let failed: Vec<String> = conjuncts
        .iter()
        .filter(|conjunct| !matches!(evaluate(conjunct, context), Ok(Value::Boolean(true))))
        .map(|conjunct| conjunct.to_string())
        .collect();

    if failed.is_empty() {
        None
    } else {
        Some(failed.join(", "))
    }
}

fn collect_conjuncts<'a>(expr: &'a Expression, out: &mut Vec<&'a Expression>) {
    match expr {
        Expression::BinaryOp {
            op: BinaryOperator::And,
            left,
            right,
        } => {
            collect_conjuncts(left, out);
            collect_conjuncts(right, out);
        }
        _ => out.push(expr),
    }
}
