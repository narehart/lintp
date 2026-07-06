use anyhow::{Context as ErrorContext, Result};
use glob::Pattern;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{ParsedConfig, RuleEntry};
use crate::dsl::ast::{BinaryOperator, Expression};
use crate::dsl::evaluator::{evaluate, EvaluationContext, FsCache, RegexCache, Value};
use crate::dsl::parser::parse_expression;
use crate::util::forward_slashes;

/// Outcome of checking a single file or directory against its applicable
/// rule.
#[derive(Debug)]
pub enum LintResult {
    /// The path matched its applicable rule (or no rule applied to it).
    Success(PathBuf),
    /// The path failed its applicable rule, or could not be checked at all
    /// (e.g. an I/O error reading the path), with `rule` identifying which
    /// rule key was applied (or `"io"` for a filesystem access failure).
    Failure {
        path: PathBuf,
        rule: String,
        message: String,
    },
}

/// Caches threaded through an entire lint run so repeated work — parsing a
/// rule string, compiling a regex, reading a glob'd directory — happens
/// once per distinct input rather than once per file visited. Bundled into
/// one struct so passing them around doesn't blow out function arity.
struct LintCaches<'a> {
    rules: &'a mut HashMap<String, Expression>,
    fs: &'a FsCache,
    regex: &'a RegexCache,
}

/// Walk `dir` and check every file and directory against `config`'s rules,
/// returning one [`LintResult`] per visited path (after ignore patterns are
/// applied). A per-path I/O error (e.g. a permission-denied subdirectory) is
/// reported as a [`LintResult::Failure`] rather than aborting the run.
///
/// # Errors
///
/// Returns [`crate::Error::Glob`] if an ignore or path-rule glob pattern
/// fails to compile, [`crate::Error::Dsl`] if a rule expression fails to
/// parse or fails to evaluate (e.g. an unknown variable or matcher
/// reference), or [`crate::Error::Internal`] if a walked path unexpectedly
/// has no file name (should not occur in practice).
pub fn run_lint(
    dir: &Path,
    config: &ParsedConfig,
    verbose: bool,
) -> std::result::Result<Vec<LintResult>, crate::Error> {
    let mut results = Vec::new();

    let ignore_patterns: Vec<Pattern> = config
        .raw
        .lintp
        .ignore
        .iter()
        .map(|pattern| {
            Pattern::new(pattern).map_err(|e| crate::Error::Glob {
                kind: "ignore pattern",
                pattern: pattern.clone(),
                source: e,
            })
        })
        .collect::<std::result::Result<Vec<Pattern>, crate::Error>>()?;

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
                .map_err(|e| crate::Error::Glob {
                    kind: "path-rule glob",
                    pattern: glob_pattern.clone(),
                    source: e,
                })
        })
        .collect::<std::result::Result<Vec<_>, crate::Error>>()?;
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

    // Regexes are compiled once per distinct pattern and reused across every
    // file evaluated against a rule, instead of recompiling on every call
    let regex_cache: RegexCache = RegexCache::default();

    let mut caches = LintCaches {
        rules: &mut rule_cache,
        fs: &fs_cache,
        regex: &regex_cache,
    };

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
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                let path = e.path().unwrap_or(dir).to_path_buf();
                results.push(LintResult::Failure {
                    path,
                    rule: "io".to_string(),
                    message: format!("Could not read: {}", e),
                });
                continue;
            }
        };
        let path = entry.path();

        // Get relative path from the linted directory. Unreachable in
        // practice: every entry here was yielded by walking `dir` itself.
        let rel_path = path.strip_prefix(dir).map_err(|_| {
            crate::Error::Internal(format!(
                "Walked path '{}' is not under '{}'",
                path.display(),
                dir.display()
            ))
        })?;

        if verbose {
            println!("Checking: {}", rel_path.display());
        }

        // Find applicable rules for this path
        let applicable_rules = find_applicable_rules(
            rel_path,
            &config.raw.lintp.config.global_rules,
            &path_rule_patterns,
        );

        // Apply rules. Unreachable in practice: `min_depth(1)` excludes the
        // root, so every walked entry has a parent and thus a file name.
        let name = path
            .file_name()
            .ok_or_else(|| {
                crate::Error::Internal(format!("Walked path '{}' has no file name", path.display()))
            })?
            .to_string_lossy();

        // WalkDir already stat'd this entry to build it; asking the entry
        // instead of re-stat'ing `path` avoids a redundant syscall per file.
        // A symlink whose target is a directory is treated as a directory
        // for rule-matching purposes (its name is checked against `.dir`
        // rules) even though WalkDir does not traverse into it; a broken
        // symlink (missing target) falls through and is treated as a file.
        let is_dir =
            entry.file_type().is_dir() || (entry.path_is_symlink() && entry.path().is_dir());
        let result = apply_rules(
            &name,
            path,
            is_dir,
            &applicable_rules,
            &config.parsed_matchers,
            &mut caches,
        )
        .map_err(|e| crate::Error::Dsl(format!("{:#}", e)))?;

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

/// Borrows rather than clones: the returned map holds references into
/// `global_rules`/`path_rule_patterns`, both of which live for the whole
/// lint run, so there is no need to clone a `RuleEntry` — or the whole
/// global-rules map — for every single file visited.
fn find_applicable_rules<'a>(
    path: &Path,
    global_rules: &'a HashMap<String, RuleEntry>,
    path_rule_patterns: &'a [(Pattern, &'a HashMap<String, RuleEntry>)],
) -> HashMap<&'a str, &'a RuleEntry> {
    let mut rules: HashMap<&str, &RuleEntry> =
        global_rules.iter().map(|(k, v)| (k.as_str(), v)).collect();

    // Find path-specific rules
    for (pattern, pattern_rules) in path_rule_patterns {
        if pattern.matches_path(path) {
            // Merge pattern rules, overriding global rules
            for (key, value) in *pattern_rules {
                rules.insert(key.as_str(), value);
            }
        }
    }

    rules
}

fn apply_rules(
    name: &str,
    path: &Path,
    is_dir: bool,
    rules: &HashMap<&str, &RuleEntry>,
    custom_matchers: &HashMap<String, Expression>,
    caches: &mut LintCaches,
) -> Result<LintResult> {
    // Setup evaluation context
    let mut variables = HashMap::new();
    variables.insert("NAME".to_string(), Value::String(name.to_string()));
    // PATH and PARENT are exposed as strings: every documented usage is a
    // string operation (contains($PATH, "/src/"), $PARENT == "."), and the
    // string functions reject Path values. Forward-slash-normalized so
    // `/`-based rules behave the same on Windows as on Unix.
    variables.insert(
        "PATH".to_string(),
        Value::String(forward_slashes(&path.display().to_string())),
    );

    if let Some(parent) = path.parent() {
        variables.insert(
            "PARENT".to_string(),
            Value::String(forward_slashes(&parent.display().to_string())),
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
        fs_cache: Some(caches.fs),
        regex_cache: Some(caches.regex),
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
            .filter(|key| key.starts_with('.') && path_str.ends_with(**key))
            .max_by_key(|key| key.len())
        {
            extension = (*key).to_string();
        }

        // If no specific extension rule found, fallback to .*
        if !rules.contains_key(extension.as_str()) {
            extension = ".*".to_string();
        }

        extension
    };

    if let Some(entry) = rules.get(rule_key.as_str()) {
        let rule_str = &entry.rule;

        // Parse the rule once per distinct rule string
        if !caches.rules.contains_key(rule_str) {
            let expr = parse_expression(rule_str)
                .with_context(|| format!("Failed to parse rule: {}", rule_str))?;
            caches.rules.insert(rule_str.clone(), expr);
        }
        let rule_expr = &caches.rules[rule_str];

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
        .map(ToString::to_string)
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
