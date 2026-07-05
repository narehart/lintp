use anyhow::Result;
use std::path::{Path, PathBuf};

// Only import what we actually use
use lintp::config::load_config;

mod test_constants;
use test_constants::*;

struct TestConfig {
    _temp_dir: tempfile::TempDir, // Prefixed with underscore to indicate it's kept for its lifetime
    config_path: PathBuf,
}

/// Helper function to create a temporary YAML config file
fn create_test_config(content: &str) -> Result<TestConfig> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("lintp.yml");
    std::fs::write(&config_path, content)?;

    // Return both in a struct to keep the tempdir alive
    Ok(TestConfig {
        _temp_dir: temp_dir,
        config_path,
    })
}

/// Tests for loading a basic valid config file
#[test]
fn test_load_basic_config() -> Result<()> {
    let config_content = create_minimal_test_config();

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path)?;

    // Check basic structure
    assert_eq!(parsed_config.raw.lintp.custom_matchers.len(), 3);
    assert_eq!(parsed_config.raw.lintp.ignore.len(), 1);

    // Check parsed matchers
    assert!(parsed_config.parsed_matchers.contains_key("kebab-case"));
    assert!(parsed_config.parsed_matchers.contains_key("PascalCase"));
    assert!(parsed_config.parsed_matchers.contains_key("js-file"));

    // Check global rules
    assert!(parsed_config
        .raw
        .lintp
        .config
        .global_rules
        .contains_key(".dir"));
    assert!(parsed_config
        .raw
        .lintp
        .config
        .global_rules
        .contains_key(".js"));

    // Check path rules - minimal config has no path rules
    assert_eq!(parsed_config.raw.lintp.config.path_rules.len(), 0);

    Ok(())
}

/// Tests for loading config with references between custom matchers
#[test]
fn test_load_config_with_references() -> Result<()> {
    let config_content = r#"
lintp:
  custom-matchers:
    js-file: $EXT == "js"
    ts-file: $EXT == "ts"
    script-file: js-file || ts-file
    test-file: matches($NAME, /\.spec\.(j|t)s$/)
    component-file: PascalCase && script-file
    PascalCase: matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)
    
  config:
    .js: js-file
    .ts: ts-file
    .dir: PascalCase
    
    "src/components/*":
      .dir: PascalCase
      .js: component-file
      .ts: component-file
      
  ignore:
    - node_modules
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path)?;

    // Check that all matchers are parsed
    assert_eq!(parsed_config.parsed_matchers.len(), 6);
    assert!(parsed_config.parsed_matchers.contains_key("script-file"));
    assert!(parsed_config.parsed_matchers.contains_key("component-file"));

    Ok(())
}

/// Tests for loading config with complex custom matchers
#[test]
fn test_load_config_with_complex_matchers() -> Result<()> {
    let config_content = r#"
lintp:
  custom-matchers:
    kebab-case: matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)
    snake_case: matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)
    allowed-cases: kebab-case || snake_case
    
    test-file-matcher: >
      matches($NAME, /.*\.spec\.(j|t)sx?$/) &&
      any(
        map(
          siblings("*.{js,jsx,ts,tsx}"),
          without($item, ".spec")
        ),
        $NAME == "${item}.spec.${EXT}"
      )
    
    component-has-test: >
      exists("__tests__/${BASENAME}.spec.{js,jsx,ts,tsx}")
    
  config:
    .dir: allowed-cases
    .js: kebab-case
    .spec.js: test-file-matcher
    
    "src/components/*":
      .js: component-has-test
  
  ignore: []
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path)?;

    // Check that complex matchers are parsed
    assert!(parsed_config
        .parsed_matchers
        .contains_key("test-file-matcher"));
    assert!(parsed_config
        .parsed_matchers
        .contains_key("component-has-test"));

    Ok(())
}

/// Tests for handling errors in config loading
#[test]
fn test_config_loading_errors() -> Result<()> {
    // Test with malformed YAML
    let config_content = r#"
lintp:
  custom-matchers:
    - invalid
  config:
    not valid yaml
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path);
    assert!(parsed_config.is_err());

    // Test with invalid expression in matcher
    let config_content = r#"
lintp:
  custom-matchers:
    invalid-expr: $BASENAME ==== "test"
  config:
    .js: invalid-expr
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path);
    assert!(parsed_config.is_err());

    // Test with non-existent file
    let result = load_config(Path::new("/nonexistent/file.yml"));
    assert!(result.is_err());

    Ok(())
}

/// Tests for config with circular references in custom matchers
#[test]
fn test_config_with_circular_references() -> Result<()> {
    let config_content = r#"
lintp:
  custom-matchers:
    a: b
    b: c
    c: a
  
  config:
    .js: a
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path);

    // This should error due to circular references
    assert!(parsed_config.is_err());

    Ok(())
}

/// Tests for config with empty/minimal values
#[test]
fn test_minimal_config() -> Result<()> {
    let config_content = r#"
lintp:
  config:
    .js: matches($NAME, /.*\.js$/)
"#;

    let test_config = create_test_config(config_content)?;
    let parsed_config = load_config(&test_config.config_path)?;

    // Check that the minimal config loads
    assert_eq!(parsed_config.raw.lintp.custom_matchers.len(), 0);
    assert_eq!(parsed_config.raw.lintp.ignore.len(), 0);
    assert_eq!(parsed_config.parsed_matchers.len(), 0);
    assert_eq!(parsed_config.raw.lintp.config.global_rules.len(), 1);
    assert_eq!(parsed_config.raw.lintp.config.path_rules.len(), 0);

    Ok(())
}

/// Rules referencing unknown matchers must fail at config load time,
/// not when a matching file is first linted.
#[test]
fn test_unknown_matcher_reference_fails_at_load() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("lintp.yml");
    std::fs::write(
        &config_path,
        r#"
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z-]+$/)"
  config:
    .js: "keba-case"
  ignore: []
"#,
    )?;

    let Err(error) = load_config(&config_path) else {
        panic!("Expected load_config to reject the typo");
    };
    let message = format!("{:#}", error);
    assert!(
        message.contains("keba-case"),
        "Error should name the unknown matcher, got: {}",
        message
    );

    Ok(())
}

/// Malformed rule expressions must also fail at load time.
#[test]
fn test_malformed_rule_fails_at_load() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("lintp.yml");
    std::fs::write(
        &config_path,
        r#"
lintp:
  config:
    .js: "kebab-case &&"
  ignore: []
"#,
    )?;

    assert!(load_config(&config_path).is_err());

    Ok(())
}

/// Every config-shape mistake that would otherwise be silently inert must
/// fail at load time with a message naming the problem.
#[test]
fn test_config_load_rejects_silent_footguns() -> Result<()> {
    let cases: &[(&str, &str, &str)] = &[
        (
            "non-dot global rule key",
            r#"
lintp:
  config:
    js: "kebab-case"
"#,
            "Invalid rule key 'js'",
        ),
        (
            "path scope mis-indented into a rule entry",
            r#"
lintp:
  config:
    "src/*":
      rule: "kebab-case"
"#,
            "nest extension rules under it",
        ),
        (
            "non-dot key inside a path scope",
            r#"
lintp:
  config:
    "src/*":
      js: "kebab-case"
"#,
            "under path scope 'src/*'",
        ),
        (
            "empty path scope",
            r#"
lintp:
  config:
    "src/*": {}
"#,
            "has no rules",
        ),
        (
            "typo'd top-level section silently dropping matchers",
            r#"
lintp:
  custom_matchers:
    kebab-case: "matches($BASENAME, /^[a-z]+$/)"
  config:
    .js: "true"
"#,
            "unknown field",
        ),
        (
            "invalid path-scope glob",
            r#"
lintp:
  config:
    "src/[*":
      .js: "true"
"#,
            "Invalid glob pattern for path scope",
        ),
        (
            "invalid ignore pattern",
            r#"
lintp:
  config:
    .js: "true"
  ignore:
    - "[bad"
"#,
            "Invalid ignore pattern",
        ),
        (
            "matcher shadowed by a boolean literal",
            r#"
lintp:
  custom-matchers:
    "true": "$EXT == \"js\""
  config:
    .js: "true"
"#,
            "shadowed by the boolean literal",
        ),
    ];

    for (name, yaml, expected) in cases {
        let config = create_test_config(yaml)?;
        let result = load_config(&config.config_path);
        let err = format!(
            "{:#}",
            result
                .err()
                .unwrap_or_else(|| panic!("case '{}' should fail to load", name))
        );
        assert!(
            err.contains(expected),
            "case '{}': error should mention '{}', got: {}",
            name,
            expected,
            err
        );
    }

    Ok(())
}

/// Brace-expanded rule keys: ".{png,jpg}" assigns the same entry to each
/// suffix, at the global level and inside path scopes; scope globs expand
/// too, and malformed braces fail at load.
#[test]
fn test_brace_expanded_keys() -> Result<()> {
    let config = create_test_config(
        r#"
lintp:
  custom-matchers:
    camel: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"
  config:
    ".{png,jpg,webp}":
      rule: "camel"
      message: "images are camelCase"
    ".{ttf,otf}": "true"
    ".test.{ts,tsx}": "true"
    "assets/*":
      ".{svg,gif}": "camel"
    "api/{auth,billing}/*":
      .go: "camel"
"#,
    )?;
    let parsed = load_config(&config.config_path)?;
    let global = &parsed.raw.lintp.config.global_rules;
    for key in [".png", ".jpg", ".webp"] {
        let entry = global.get(key).unwrap_or_else(|| panic!("missing {}", key));
        assert_eq!(entry.rule, "camel");
        assert_eq!(entry.message.as_deref(), Some("images are camelCase"));
    }
    assert!(global.contains_key(".ttf") && global.contains_key(".otf"));
    // brace groups compose with suffix keys
    assert!(global.contains_key(".test.ts") && global.contains_key(".test.tsx"));
    let scoped = &parsed.raw.lintp.config.path_rules["assets/*"];
    assert!(scoped.contains_key(".svg") && scoped.contains_key(".gif"));
    // scope globs expand into separate scopes
    assert!(parsed
        .raw
        .lintp
        .config
        .path_rules
        .contains_key("api/auth/*"));
    assert!(parsed
        .raw
        .lintp
        .config
        .path_rules
        .contains_key("api/billing/*"));

    for (yaml, expected) in [
        ("\"{png\": \"true\"", "unbalanced braces"),
        ("\".{png,}\": \"true\"", "empty alternative"),
        ("\".{a{b,c}}\": \"true\"", "nested braces"),
        ("\".{png,jpg\": \"true\"", "unbalanced braces"),
    ] {
        let bad = create_test_config(&format!("\nlintp:\n  config:\n    {}\n", yaml))?;
        let err = match load_config(&bad.config_path) {
            Err(e) => format!("{:#}", e),
            Ok(_) => panic!("'{}' should fail to load", yaml),
        };
        assert!(err.contains(expected), "'{}': got {}", yaml, err);
    }

    Ok(())
}
