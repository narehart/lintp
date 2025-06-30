/// Standardized regex patterns && constants for lintp tests
///
/// This module provides consistent regex patterns for all naming conventions
/// used throughout the test suite to ensure reliability && maintainability.

// =============================================================================
// NAMING CONVENTION PATTERNS
// =============================================================================

/// Kebab-case pattern: lowercase letters, numbers, && hyphens
/// Examples: "hello-world", "my-component", "test-123"
pub const KEBAB_CASE_PATTERN: &str = r"^[a-z0-9]+(?:-[a-z0-9]+)*$";

/// PascalCase pattern: first letter uppercase, camelCase for the rest
/// Examples: "HelloWorld", "MyComponent", "TestComponent"
pub const PASCAL_CASE_PATTERN: &str = r"^[A-Z][a-zA-Z0-9]*$";

/// camelCase pattern: first letter lowercase, PascalCase for the rest
/// Examples: "helloWorld", "myComponent", "testFunction"
pub const CAMEL_CASE_PATTERN: &str = r"^[a-z][a-zA-Z0-9]*$";

/// snake_case pattern: lowercase letters, numbers, && underscores
/// Examples: "hello_world", "my_component", "test_123"
pub const SNAKE_CASE_PATTERN: &str = r"^[a-z0-9]+(?:_[a-z0-9]+)*$";

/// UPPER_SNAKE_CASE pattern: uppercase letters, numbers, && underscores
/// Examples: "HELLO_WORLD", "MY_CONSTANT", "TEST_123"
pub const UPPER_SNAKE_CASE_PATTERN: &str = r"^[A-Z0-9]+(?:_[A-Z0-9]+)*$";

// =============================================================================
// FILE EXTENSION PATTERNS
// =============================================================================

/// JavaScript file pattern
pub const JS_FILE_PATTERN: &str = r"\.js$";

/// TypeScript file pattern
pub const TS_FILE_PATTERN: &str = r"\.ts$";

/// JSX file pattern
pub const JSX_FILE_PATTERN: &str = r"\.jsx$";

/// TSX file pattern
pub const TSX_FILE_PATTERN: &str = r"\.tsx$";

/// Any script file pattern (js, ts, jsx, tsx)
pub const SCRIPT_FILE_PATTERN: &str = r"\.(js|ts|jsx|tsx)$";

// =============================================================================
// TEST FILE PATTERNS
// =============================================================================

/// Jest test file pattern (.test.js, .test.ts, etc.)
pub const JEST_TEST_PATTERN: &str = r"\.test\.(js|ts|jsx|tsx)$";

/// Spec test file pattern (.spec.js, .spec.ts, etc.)
pub const SPEC_TEST_PATTERN: &str = r"\.spec\.(js|ts|jsx|tsx)$";

/// Any test file pattern (includes both jest && spec patterns)
pub const ANY_TEST_PATTERN: &str = r"\.(test|spec)\.(js|ts|jsx|tsx)$";

// =============================================================================
// DIRECTORY PATTERNS
// =============================================================================

/// Node modules directory pattern
pub const NODE_MODULES_PATTERN: &str = r"node_modules";

/// Git directory pattern
pub const GIT_DIR_PATTERN: &str = r"\.git";

/// Dist/build directory pattern
pub const DIST_DIR_PATTERN: &str = r"(dist|build)";

/// Common ignore patterns
pub const COMMON_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
    ".nyc_output",
    "target",
];

// =============================================================================
// LINTP DSL EXPRESSIONS
// =============================================================================

/// Kebab-case matcher expression for lintp DSL
pub const KEBAB_CASE_EXPR: &str = "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)";

/// PascalCase matcher expression for lintp DSL
pub const PASCAL_CASE_EXPR: &str = "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)";

/// camelCase matcher expression for lintp DSL
pub const CAMEL_CASE_EXPR: &str = "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)";

/// snake_case matcher expression for lintp DSL
pub const SNAKE_CASE_EXPR: &str = "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)";

/// JavaScript file matcher expression for lintp DSL
pub const JS_FILE_EXPR: &str = r#"$EXT == "js""#;

/// TypeScript file matcher expression for lintp DSL
pub const TS_FILE_EXPR: &str = r#"$EXT == "ts""#;

/// JSX file matcher expression for lintp DSL
pub const JSX_FILE_EXPR: &str = r#"$EXT == "jsx""#;

/// TSX file matcher expression for lintp DSL
pub const TSX_FILE_EXPR: &str = r#"$EXT == "tsx""#;

/// Test file matcher expression for lintp DSL
pub const TEST_FILE_EXPR: &str = "matches($BASENAME, /\\.test\\.(js|ts|jsx|tsx)$/)";

/// Spec file matcher expression for lintp DSL
pub const SPEC_FILE_EXPR: &str = "matches($BASENAME, /\\.spec\\.(js|ts|jsx|tsx)$/)";

// =============================================================================
// COMMON COMBINED EXPRESSIONS
// =============================================================================

/// Any case expression (kebab-case || PascalCase)
pub const ANY_CASE_EXPR: &str = "kebab-case || PascalCase";

/// Script file expression (js || ts)
pub const SCRIPT_FILE_EXPR: &str = "js-file || ts-file";

/// React file expression (jsx || tsx)
pub const REACT_FILE_EXPR: &str = "jsx-file || tsx-file";

/// All script files expression (js, ts, jsx, tsx)
pub const ALL_SCRIPT_FILES_EXPR: &str = "js-file || ts-file || jsx-file || tsx-file";

/// Component file expression (PascalCase && script file)
pub const COMPONENT_FILE_EXPR: &str = "PascalCase && (js-file || jsx-file || ts-file || tsx-file)";

/// Utility file expression (kebab-case && script file)
pub const UTILITY_FILE_EXPR: &str = "kebab-case && (js-file || ts-file)";

// =============================================================================
// TEST HELPER FUNCTIONS
// =============================================================================

/// Creates a standard test config YAML content with the standardized patterns
pub fn create_standard_test_config() -> &'static str {
    r#"
lintp:
  custom-matchers:
    kebab-case: matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)
    PascalCase: matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)
    camelCase: matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)
    snake_case: matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)
    js-file: $EXT == "js"
    ts-file: $EXT == "ts"
    jsx-file: $EXT == "jsx"
    tsx-file: $EXT == "tsx"
    test-file: matches($NAME, /\.test\.(js|ts|jsx|tsx)$/)
    spec-file: matches($NAME, /\.spec\.(js|ts|jsx|tsx)$/)
    
  config:
    .dir: kebab-case || PascalCase
    .js: (kebab-case || PascalCase) && js-file
    .ts: (kebab-case || PascalCase) && ts-file
    .jsx: PascalCase && jsx-file
    .tsx: PascalCase && tsx-file
    
    "src/components/*":
      .dir: PascalCase
      .js: PascalCase && js-file
      .jsx: PascalCase && jsx-file
      .ts: PascalCase && ts-file
      .tsx: PascalCase && tsx-file
      
    "src/utils/*":
      .dir: kebab-case
      .js: kebab-case && js-file
      .ts: kebab-case && ts-file
      
    "src/api/*":
      .dir: camelCase
      .js: camelCase && js-file
      .ts: camelCase && ts-file
      
    "tests/*":
      .js: test-file
        
  ignore:
    - node_modules
    - .git
    - dist
    - build
"#
}

/// Creates a minimal test config for basic testing
pub fn create_minimal_test_config() -> &'static str {
    r#"
lintp:
  custom-matchers:
    kebab-case: matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)
    PascalCase: matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)
    js-file: $EXT == "js"
    
  config:
    .dir: kebab-case || PascalCase
    .js: (kebab-case || PascalCase) && js-file
        
  ignore:
    - node_modules
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    /// Test that all regex patterns compile successfully
    #[test]
    fn test_all_patterns_compile() {
        let patterns = [
            KEBAB_CASE_PATTERN,
            PASCAL_CASE_PATTERN,
            CAMEL_CASE_PATTERN,
            SNAKE_CASE_PATTERN,
            UPPER_SNAKE_CASE_PATTERN,
            JS_FILE_PATTERN,
            TS_FILE_PATTERN,
            JSX_FILE_PATTERN,
            TSX_FILE_PATTERN,
            SCRIPT_FILE_PATTERN,
            JEST_TEST_PATTERN,
            SPEC_TEST_PATTERN,
            ANY_TEST_PATTERN,
        ];

        for pattern in patterns {
            assert!(
                Regex::new(pattern).is_ok(),
                "Pattern should compile successfully: {}",
                pattern
            );
        }
    }

    /// Test kebab-case pattern validation
    #[test]
    fn test_kebab_case_pattern() {
        let pattern = Regex::new(KEBAB_CASE_PATTERN).unwrap();

        // Valid kebab-case
        assert!(pattern.is_match("hello-world"));
        assert!(pattern.is_match("my-component"));
        assert!(pattern.is_match("test-123"));
        assert!(pattern.is_match("a"));
        assert!(pattern.is_match("component"));

        // Invalid kebab-case
        assert!(!pattern.is_match("HelloWorld"));
        assert!(!pattern.is_match("hello_world"));
        assert!(!pattern.is_match("-hello"));
        assert!(!pattern.is_match("hello-"));
        assert!(!pattern.is_match("hello--world"));
        assert!(!pattern.is_match(""));
    }

    /// Test PascalCase pattern validation
    #[test]
    fn test_pascal_case_pattern() {
        let pattern = Regex::new(PASCAL_CASE_PATTERN).unwrap();

        // Valid PascalCase
        assert!(pattern.is_match("HelloWorld"));
        assert!(pattern.is_match("MyComponent"));
        assert!(pattern.is_match("A"));
        assert!(pattern.is_match("Component"));
        assert!(pattern.is_match("Test123"));

        // Invalid PascalCase
        assert!(!pattern.is_match("helloWorld"));
        assert!(!pattern.is_match("hello-world"));
        assert!(!pattern.is_match("hello_world"));
        assert!(!pattern.is_match("123Hello"));
        assert!(!pattern.is_match(""));
    }

    /// Test camelCase pattern validation
    #[test]
    fn test_camel_case_pattern() {
        let pattern = Regex::new(CAMEL_CASE_PATTERN).unwrap();

        // Valid camelCase
        assert!(pattern.is_match("helloWorld"));
        assert!(pattern.is_match("myComponent"));
        assert!(pattern.is_match("a"));
        assert!(pattern.is_match("component"));
        assert!(pattern.is_match("test123"));

        // Invalid camelCase
        assert!(!pattern.is_match("HelloWorld"));
        assert!(!pattern.is_match("hello-world"));
        assert!(!pattern.is_match("hello_world"));
        assert!(!pattern.is_match("123hello"));
        assert!(!pattern.is_match(""));
    }

    /// Test snake_case pattern validation
    #[test]
    fn test_snake_case_pattern() {
        let pattern = Regex::new(SNAKE_CASE_PATTERN).unwrap();

        // Valid snake_case
        assert!(pattern.is_match("hello_world"));
        assert!(pattern.is_match("my_component"));
        assert!(pattern.is_match("test_123"));
        assert!(pattern.is_match("a"));
        assert!(pattern.is_match("component"));

        // Invalid snake_case
        assert!(!pattern.is_match("HelloWorld"));
        assert!(!pattern.is_match("hello-world"));
        assert!(!pattern.is_match("_hello"));
        assert!(!pattern.is_match("hello_"));
        assert!(!pattern.is_match("hello__world"));
        assert!(!pattern.is_match(""));
    }

    /// Test file extension patterns
    #[test]
    fn test_file_patterns() {
        let js_pattern = Regex::new(JS_FILE_PATTERN).unwrap();
        let ts_pattern = Regex::new(TS_FILE_PATTERN).unwrap();
        let test_pattern = Regex::new(JEST_TEST_PATTERN).unwrap();

        // JavaScript files
        assert!(js_pattern.is_match("component.js"));
        assert!(js_pattern.is_match("index.js"));
        assert!(!js_pattern.is_match("component.ts"));

        // TypeScript files
        assert!(ts_pattern.is_match("component.ts"));
        assert!(ts_pattern.is_match("index.ts"));
        assert!(!ts_pattern.is_match("component.js"));

        // Test files
        assert!(test_pattern.is_match("component.test.js"));
        assert!(test_pattern.is_match("utils.test.ts"));
        assert!(!test_pattern.is_match("component.spec.js"));
        assert!(!test_pattern.is_match("component.js"));
    }
}
