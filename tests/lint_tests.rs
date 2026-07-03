use anyhow::Result;
use lintp::config::{Config, ParsedConfig};
use lintp::dsl::parser::parse_expression;
use lintp::lint::{run_lint, LintResult};
use std::collections::HashMap;
use std::path::PathBuf;

mod test_constants;
use test_constants::*;

/// Structure to hold both the temporary directory and the path
struct TestDirectory {
    _temp_dir: tempfile::TempDir, // Underscore prefix indicates it's kept for its lifetime
    path: PathBuf,
}

/// Helper function to create a temporary directory structure for testing
fn create_test_directory() -> Result<TestDirectory> {
    let temp_dir = tempfile::tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create directory structure
    let src_dir = root_path.join("src");
    std::fs::create_dir(&src_dir)?;

    let components_dir = src_dir.join("components");
    std::fs::create_dir(&components_dir)?;

    let utils_dir = src_dir.join("utils");
    std::fs::create_dir(&utils_dir)?;

    let api_dir = src_dir.join("api");
    std::fs::create_dir(&api_dir)?;

    let tests_dir = root_path.join("tests");
    std::fs::create_dir(&tests_dir)?;

    // Create valid test files using standardized naming
    std::fs::write(src_dir.join("index.js"), "// Main entry file")?;
    std::fs::write(src_dir.join("app.js"), "// App file")?;
    std::fs::write(components_dir.join("Button.js"), "// Button component")?;
    std::fs::write(components_dir.join("Card.jsx"), "// Card component")?;
    std::fs::write(utils_dir.join("format-date.js"), "// Date formatting")?;
    std::fs::write(utils_dir.join("helper-utils.js"), "// Helper utilities")?;
    std::fs::write(api_dir.join("userService.js"), "// User service API")?;
    std::fs::write(tests_dir.join("app.test.js"), "// App tests")?;

    // Create directories that should be ignored
    let node_modules_dir = root_path.join("node_modules");
    std::fs::create_dir(&node_modules_dir)?;
    std::fs::write(
        node_modules_dir.join("some-package.js"),
        "// Should be ignored",
    )?;

    let dist_dir = root_path.join("dist");
    std::fs::create_dir(&dist_dir)?;
    std::fs::write(dist_dir.join("bundle.js"), "// Should be ignored")?;

    Ok(TestDirectory {
        _temp_dir: temp_dir,
        path: root_path,
    })
}

/// Helper function to create a test config
fn create_test_config() -> Result<ParsedConfig> {
    // Create config content as YAML
    let config_content = create_standard_test_config();

    // Parse the config string
    let config: Config = serde_yaml::from_str(config_content)?;

    // Create custom matchers map
    let mut custom_matchers = HashMap::new();
    for (name, expr) in &config.lintp.custom_matchers {
        custom_matchers.insert(name.clone(), parse_expression(expr)?);
    }

    // Create parsed config
    let parsed_config = ParsedConfig {
        raw: config,
        parsed_matchers: custom_matchers,
        parsed_rules: HashMap::new(),
    };

    Ok(parsed_config)
}

/// Helper function to create invalid test files
fn create_invalid_files(test_dir: &TestDirectory) -> Result<()> {
    // Files that violate naming conventions
    std::fs::write(
        test_dir.path.join("src").join("INVALID_CASE.js"),
        "// Invalid naming",
    )?;
    std::fs::write(
        test_dir
            .path
            .join("src")
            .join("components")
            .join("invalidbutton.js"),
        "// Invalid PascalCase for component",
    )?;
    std::fs::write(
        test_dir
            .path
            .join("src")
            .join("utils")
            .join("INVALID_HELPER.js"),
        "// Invalid kebab-case for utility",
    )?;
    std::fs::write(
        test_dir.path.join("src").join("api").join("INVALID-API.js"),
        "// Invalid camelCase for API",
    )?;

    Ok(())
}

/// Helper function to count lint results by type
fn count_results(results: &[LintResult]) -> (usize, usize) {
    let successes = results
        .iter()
        .filter(|r| matches!(r, LintResult::Success(_)))
        .count();
    let failures = results
        .iter()
        .filter(|r| matches!(r, LintResult::Failure { .. }))
        .count();
    (successes, failures)
}

/// Helper function to count failures for a specific file
fn count_file_failures(results: &[LintResult], filename: &str) -> usize {
    results
        .iter()
        .filter(|r| {
            if let LintResult::Failure { path, .. } = r {
                path.to_string_lossy().contains(filename)
            } else {
                false
            }
        })
        .count()
}

/// Tests for running lint on a valid directory structure
#[test]
fn test_lint_valid_structure() -> Result<()> {
    let test_dir = create_test_directory()?;
    let config = create_test_config()?;

    let results = run_lint(&test_dir.path, &config, false)?;

    // Count successes and failures
    let successes = results
        .iter()
        .filter(|r| matches!(r, LintResult::Success(_)))
        .count();
    let failures = results
        .iter()
        .filter(|r| matches!(r, LintResult::Failure { .. }))
        .count();

    // All files should pass
    assert!(failures == 0, "Expected 0 failures, got {}", failures);
    assert!(successes > 0, "Expected more than 0 successes");

    Ok(())
}

/// Tests for running lint with invalid files
#[test]
fn test_lint_invalid_structure() -> Result<()> {
    let test_dir = create_test_directory()?;
    let config = create_test_config()?;

    // Create invalid files
    create_invalid_files(&test_dir)?;

    let results = run_lint(&test_dir.path, &config, false)?;
    let (_, failures) = count_results(&results);

    // Should have at least 4 failures (one for each invalid file)
    assert!(
        failures >= 4,
        "Expected at least 4 failures, got {}",
        failures
    );

    // Verify specific file failures
    assert_eq!(
        count_file_failures(&results, "INVALID_CASE.js"),
        1,
        "INVALID_CASE.js should fail"
    );
    assert_eq!(
        count_file_failures(&results, "invalidbutton.js"),
        1,
        "invalidbutton.js should fail"
    );
    assert_eq!(
        count_file_failures(&results, "INVALID_HELPER.js"),
        1,
        "INVALID_HELPER.js should fail"
    );
    assert_eq!(
        count_file_failures(&results, "INVALID-API.js"),
        1,
        "INVALID-API.js should fail"
    );

    Ok(())
}

/// Tests for validating ignore patterns
#[test]
fn test_lint_ignore_patterns() -> Result<()> {
    let test_dir = create_test_directory()?;
    let mut config = create_test_config()?;

    // Add additional ignore patterns for testing
    config.raw.lintp.ignore.push("src/**/*.jsx".to_string());
    config.raw.lintp.ignore.push("tests".to_string());

    let results = run_lint(&test_dir.path, &config, false)?;

    // Helper function to check if any results contain a pattern
    let contains_pattern = |pattern: &str| {
        results.iter().any(|r| match r {
            LintResult::Success(path) => path.to_string_lossy().contains(pattern),
            LintResult::Failure { path, .. } => path.to_string_lossy().contains(pattern),
        })
    };

    // Verify ignored patterns are not present in results
    assert!(!contains_pattern(".jsx"), "JSX files should be ignored");
    assert!(
        !contains_pattern("node_modules"),
        "node_modules should be ignored"
    );
    assert!(
        !contains_pattern("dist"),
        "dist directory should be ignored"
    );
    assert!(
        !contains_pattern("tests"),
        "tests directory should be ignored"
    );

    Ok(())
}

/// Tests for finding applicable rules
#[test]
fn test_find_applicable_rules() -> Result<()> {
    let test_dir = create_test_directory()?;
    let config = create_test_config()?;

    let results = run_lint(&test_dir.path, &config, false)?;
    let (successes, failures) = count_results(&results);

    // All valid files should pass their respective rules
    assert_eq!(failures, 0, "No files should fail with valid structure");
    assert!(successes > 0, "Should have successful validations");

    // Helper function to check if a file passed validation
    let file_passed = |filename: &str| {
        results.iter().any(|r| {
            if let LintResult::Success(path) = r {
                path.to_string_lossy().contains(filename)
            } else {
                false
            }
        })
    };

    // Verify specific files pass their rules
    assert!(
        file_passed("Button.js"),
        "Button.js should pass component rules"
    );
    assert!(
        file_passed("format-date.js"),
        "format-date.js should pass utils rules"
    );
    assert!(
        file_passed("userService.js"),
        "userService.js should pass API rules"
    );
    assert!(
        file_passed("app.test.js"),
        "app.test.js should pass test rules"
    );

    Ok(())
}

/// Tests for handling directory validation
#[test]
fn test_directory_validation() -> Result<()> {
    let test_dir = create_test_directory()?;
    let config = create_test_config()?;

    // Create directories with invalid names
    std::fs::create_dir(test_dir.path.join("INVALID-dir-case"))?;
    std::fs::create_dir(test_dir.path.join("src").join("bad_Dir_Name"))?;

    let results = run_lint(&test_dir.path, &config, false)?;
    let (_, failures) = count_results(&results);

    // Should have failures for invalid directory names
    assert!(
        failures >= 2,
        "Should have failures for invalid directory names"
    );

    // Verify specific directory failures
    assert_eq!(
        count_file_failures(&results, "INVALID-dir-case"),
        1,
        "INVALID-dir-case should fail"
    );
    assert_eq!(
        count_file_failures(&results, "bad_Dir_Name"),
        1,
        "bad_Dir_Name should fail"
    );

    Ok(())
}

/// Tests for handling errors during linting
#[test]
fn test_lint_error_handling() -> Result<()> {
    let test_dir = create_test_directory()?;
    let mut config = create_test_config()?;

    // Create a rule with an invalid expression
    config.raw.lintp.config.global_rules.insert(
        ".test".to_string(),
        lintp::config::RuleEntry {
            rule: "invalid expression syntax".to_string(),
            message: None,
        },
    );

    // Create a file that would match this rule
    std::fs::write(
        test_dir.path.join("test.test"),
        "// Test file with invalid rule",
    )?;

    // This should return an error due to invalid expression
    let result = run_lint(&test_dir.path, &config, false);
    assert!(
        result.is_err(),
        "Should fail with invalid expression syntax"
    );

    Ok(())
}

/// Tests for linting with verbose output
#[test]
fn test_lint_verbose_output() -> Result<()> {
    let test_dir = create_test_directory()?;
    let config = create_test_config()?;

    // Run with verbose flag - this doesn't really test the output
    // but makes sure the function runs without error
    let results = run_lint(&test_dir.path, &config, true)?;

    assert!(!results.is_empty(), "Expected results from linting");

    Ok(())
}

/// Compound-extension rules must be selected deterministically: the longest
/// matching suffix wins, regardless of rule map iteration order.
#[test]
fn test_compound_extension_rule_precedence_is_deterministic() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    std::fs::write(root.join("Button.test.tsx"), "// test file")?;

    let config_content = r#"
lintp:
  config:
    .tsx: "false"
    .test.tsx: "true"
  ignore: []
"#;
    let config: Config = serde_yaml::from_str(config_content)?;
    let parsed_config = ParsedConfig {
        raw: config,
        parsed_matchers: HashMap::new(),
        parsed_rules: HashMap::new(),
    };

    // Before the longest-suffix fix the winner depended on HashMap iteration
    // order, so a single run could pass or fail at random. Run repeatedly to
    // guard against that nondeterminism creeping back in.
    for _ in 0..20 {
        let results = run_lint(root, &parsed_config, false)?;
        assert_eq!(results.len(), 1, "Expected exactly one linted file");
        assert!(
            matches!(results[0], LintResult::Success(_)),
            "The .test.tsx rule (true) must win over the .tsx rule (false)"
        );
    }

    Ok(())
}

/// The documented lambda syntax — any(siblings("*"), endsWith($item, ".js"))
/// with the lambda unquoted — must work end-to-end. It previously failed
/// with "Unknown variable: item" because lambdas were evaluated eagerly.
#[test]
fn test_documented_lambda_syntax_works() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    std::fs::write(root.join("app.js"), "// js")?;
    std::fs::write(root.join("util.js"), "// js")?;
    std::fs::write(root.join("readme.md"), "# md")?;

    let config_content = r#"
lintp:
  custom-matchers:
    has-js-sibling: 'any(siblings("*"), endsWith($item, ".js"))'
    no-py-sibling: '!any(siblings("*"), endsWith($item, ".py"))'
  config:
    .js: "has-js-sibling && no-py-sibling"
    .md: 'all(siblings("*.js"), matches($item, /^[a-z]+\.js$/))'
  ignore: []
"#;
    let config: Config = serde_yaml::from_str(config_content)?;
    let mut parsed_matchers = HashMap::new();
    for (name, expr) in &config.lintp.custom_matchers {
        parsed_matchers.insert(name.clone(), parse_expression(expr)?);
    }
    let parsed_config = ParsedConfig {
        raw: config,
        parsed_matchers,
        parsed_rules: HashMap::new(),
    };

    let results = run_lint(root, &parsed_config, false)?;
    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(
            matches!(result, LintResult::Success(_)),
            "Expected success, got: {:?}",
            result
        );
    }

    Ok(())
}

/// Failure messages must name the specific conjunct(s) that failed so
/// users don't have to bisect composed rules by hand.
#[test]
fn test_failure_message_explains_failed_conjunct() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    std::fs::write(root.join("BadName.js"), "// js")?;

    let config_content = r#"
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    js-file: '$EXT == "js"'
  config:
    .js: "kebab-case && js-file"
  ignore: []
"#;
    let config: Config = serde_yaml::from_str(config_content)?;
    let mut parsed_matchers = HashMap::new();
    for (name, expr) in &config.lintp.custom_matchers {
        parsed_matchers.insert(name.clone(), parse_expression(expr)?);
    }
    let parsed_config = ParsedConfig {
        raw: config,
        parsed_matchers,
        parsed_rules: HashMap::new(),
    };

    let results = run_lint(root, &parsed_config, false)?;
    assert_eq!(results.len(), 1);
    match &results[0] {
        LintResult::Failure { message, .. } => {
            assert!(
                message.contains("(failed: kebab-case)"),
                "Expected the failing conjunct to be named, got: {}",
                message
            );
            assert!(
                !message.contains("js-file)"),
                "The passing conjunct must not be reported as failed: {}",
                message
            );
        }
        other => panic!("Expected failure, got: {:?}", other),
    }

    Ok(())
}

/// Rules configured as `{rule: ..., message: ...}` report the custom
/// message instead of the raw expression.
#[test]
fn test_custom_rule_message() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    std::fs::write(root.join("badName.tsx"), "// tsx")?;

    let config_content = r#"
lintp:
  custom-matchers:
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  config:
    .tsx:
      rule: "PascalCase"
      message: "Component files must be PascalCase (see CONTRIBUTING.md)"
  ignore: []
"#;
    let config: Config = serde_yaml::from_str(config_content)?;
    let mut parsed_matchers = HashMap::new();
    for (name, expr) in &config.lintp.custom_matchers {
        parsed_matchers.insert(name.clone(), parse_expression(expr)?);
    }
    let parsed_config = ParsedConfig {
        raw: config,
        parsed_matchers,
        parsed_rules: HashMap::new(),
    };

    let results = run_lint(root, &parsed_config, false)?;
    assert_eq!(results.len(), 1);
    match &results[0] {
        LintResult::Failure { message, .. } => {
            assert!(
                message.contains("Component files must be PascalCase"),
                "Expected the custom message, got: {}",
                message
            );
            assert!(
                !message.contains("Does not match rule"),
                "Custom message should replace the default text: {}",
                message
            );
        }
        other => panic!("Expected failure, got: {:?}", other),
    }

    Ok(())
}
