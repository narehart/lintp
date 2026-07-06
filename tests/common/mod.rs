// Shared test support module, included via `mod common;` from several
// integration test binaries. Each binary only exercises a subset of these
// helpers/constants, so per-binary "never used" warnings are expected and
// not a sign of dead code in the shared module itself.
#![allow(dead_code)]

pub mod constants;

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lintp::config::{Config, LintPConfig, RuleConfig, RuleEntry};
use lintp::dsl::ast::Expression;
use lintp::dsl::evaluator::{EvaluationContext, Value};
use lintp::dsl::parser::parse_expression;

/// Shorthand for a rule entry without a custom message
pub fn rule(rule: &str) -> RuleEntry {
    RuleEntry {
        rule: rule.to_string(),
        message: None,
    }
}

/// Create a temporary directory for testing
pub fn create_temp_dir() -> Result<PathBuf> {
    let temp_dir = tempfile::tempdir()?;
    Ok(temp_dir.path().to_path_buf())
}

/// Create a test file with the given content
pub fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<PathBuf> {
    let file_path = dir.join(name);
    std::fs::write(&file_path, content)?;
    Ok(file_path)
}

/// Create a test directory structure
pub fn create_test_dir_structure(root: &Path) -> Result<()> {
    // Create nested directory structure
    let src_dir = root.join("src");
    std::fs::create_dir(&src_dir)?;

    let components_dir = src_dir.join("components");
    std::fs::create_dir(&components_dir)?;

    let utils_dir = src_dir.join("utils");
    std::fs::create_dir(&utils_dir)?;

    // Create some test files
    create_test_file(&src_dir, "index.js", "// Main entry file")?;
    create_test_file(&components_dir, "Button.js", "// Button component")?;
    create_test_file(&utils_dir, "format-date.js", "// Date formatter utility")?;

    Ok(())
}

/// Create a basic test config file
pub fn create_test_config_file(dir: &Path, content: &str) -> Result<PathBuf> {
    create_test_file(dir, "lintp.yml", content)
}

/// Create a basic evaluation context for testing
pub fn create_test_evaluation_context<'a>(
    path: &'a Path,
    custom_matchers: &'a HashMap<String, Expression>,
) -> EvaluationContext<'a> {
    let mut variables = HashMap::new();

    let name = path
        .file_name()
        .map_or("".to_string(), |n| n.to_string_lossy().to_string());

    variables.insert("NAME".to_string(), Value::String(name.clone()));
    variables.insert(
        "PATH".to_string(),
        Value::String(path.display().to_string()),
    );

    if let Some(ext) = path.extension() {
        variables.insert(
            "EXT".to_string(),
            Value::String(ext.to_string_lossy().to_string()),
        );
    }

    if let Some(stem) = path.file_stem() {
        variables.insert(
            "BASENAME".to_string(),
            Value::String(stem.to_string_lossy().to_string()),
        );
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

/// Parse multiple expressions into custom matchers
pub fn parse_custom_matchers(
    matchers: &HashMap<String, &str>,
) -> Result<HashMap<String, Expression>> {
    let mut result = HashMap::new();

    for (name, expr_str) in matchers {
        let expr = parse_expression(expr_str)?;
        result.insert(name.clone(), expr);
    }

    Ok(result)
}

/// Create a basic test config
pub fn create_test_config() -> Result<Config> {
    // Create global rules
    let mut global_rules = HashMap::new();
    global_rules.insert(".dir".to_string(), rule("kebab-case or PascalCase"));
    global_rules.insert(".js".to_string(), rule("js-file"));

    // Create path rules
    let mut path_rules = HashMap::new();

    // Rules for src/components
    let mut component_rules = HashMap::new();
    component_rules.insert(".dir".to_string(), rule("PascalCase"));
    component_rules.insert(".js".to_string(), rule("PascalCase and js-file"));
    path_rules.insert("src/components/*".to_string(), component_rules);

    // Rules for src/utils
    let mut utils_rules = HashMap::new();
    utils_rules.insert(".dir".to_string(), rule("kebab-case"));
    utils_rules.insert(".js".to_string(), rule("kebab-case and js-file"));
    path_rules.insert("src/utils/*".to_string(), utils_rules);

    // Create rule config
    let rule_config = RuleConfig {
        global_rules,
        path_rules,
    };

    // Create custom matchers
    let mut custom_matchers = HashMap::new();
    custom_matchers.insert(
        "kebab-case".to_string(),
        "matches($NAME, /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/)".to_string(),
    );
    custom_matchers.insert(
        "PascalCase".to_string(),
        "matches($NAME, /^[A-Z][a-zA-Z0-9]*$/)".to_string(),
    );
    custom_matchers.insert("js-file".to_string(), "$EXT == \"js\"".to_string());

    // Create ignore list
    let ignore = vec!["node_modules".to_string(), ".git".to_string()];

    // Create config
    let config = Config {
        lintp: LintPConfig {
            custom_matchers,
            config: rule_config,
            ignore,
        },
    };

    Ok(config)
}

/// Compare two files to check if they have the same content
pub fn files_have_same_content(file1: &Path, file2: &Path) -> Result<bool> {
    let content1 = std::fs::read_to_string(file1)?;
    let content2 = std::fs::read_to_string(file2)?;
    Ok(content1 == content2)
}

/// Check if a file contains a specific string
pub fn file_contains(file: &Path, search_str: &str) -> Result<bool> {
    let content = std::fs::read_to_string(file)?;
    Ok(content.contains(search_str))
}
