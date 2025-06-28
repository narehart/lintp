use anyhow::Result;
use std::path::{ Path, PathBuf };

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
  assert!(parsed_config.raw.lintp.config.global_rules.contains_key(".dir"));
  assert!(parsed_config.raw.lintp.config.global_rules.contains_key(".js"));

  // Check path rules - minimal config has no path rules
  assert_eq!(parsed_config.raw.lintp.config.path_rules.len(), 0);

  Ok(())
}

/// Tests for loading config with references between custom matchers
#[test]
fn test_load_config_with_references() -> Result<()> {
  let config_content =
    r#"
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
  let config_content =
    r#"
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
  assert!(parsed_config.parsed_matchers.contains_key("test-file-matcher"));
  assert!(parsed_config.parsed_matchers.contains_key("component-has-test"));

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
  let config_content =
    r#"
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
