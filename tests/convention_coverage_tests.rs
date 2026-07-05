//! Coverage tests for the conventions file-naming linters commonly
//! enforce: every capability in that class of tools must be expressible
//! in lintp, plus the relational rules that class cannot express.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use lintp::config::{Config, ParsedConfig};
use lintp::dsl::parser::parse_expression;
use lintp::lint::{run_lint, LintResult};

fn load(yaml: &str) -> Result<ParsedConfig> {
    let config: Config = serde_yaml::from_str(yaml)?;
    let mut parsed_matchers = HashMap::new();
    for (name, expr) in &config.lintp.custom_matchers {
        parsed_matchers.insert(name.clone(), parse_expression(expr)?);
    }
    Ok(ParsedConfig {
        raw: config,
        parsed_matchers,
        parsed_rules: HashMap::new(),
    })
}

/// Lint `root` and return the failing file names (not full paths).
fn failing_names(root: &Path, config: &ParsedConfig) -> Result<Vec<String>> {
    let mut names: Vec<String> = run_lint(root, config, false)?
        .iter()
        .filter_map(|r| match r {
            LintResult::Failure { path, .. } => {
                Some(path.file_name().unwrap().to_string_lossy().to_string())
            }
            LintResult::Success(_) => None,
        })
        .collect();
    names.sort();
    Ok(names)
}

fn touch(root: &Path, rel: &str) -> Result<()> {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, "")?;
    Ok(())
}

/// Convention: lowercase — every letter lowercase, non-letters ignored.
#[test]
fn covers_lowercase() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "logo-2.png")?;
    touch(dir.path(), "Logo.png")?;
    let config = load(
        r#"
lintp:
  config:
    .png: "matches($BASENAME, /^[^A-Z]*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["Logo.png"]);
    Ok(())
}

/// Convention: camelcase.
#[test]
fn covers_camelcase() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "dataStore.js")?;
    touch(dir.path(), "UserModel.js")?;
    let config = load(
        r#"
lintp:
  config:
    .js: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["UserModel.js"]);
    Ok(())
}

/// Convention: pascalcase.
#[test]
fn covers_pascalcase() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "MenuBar.tsx")?;
    touch(dir.path(), "statusBadge.tsx")?;
    let config = load(
        r#"
lintp:
  config:
    .tsx: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["statusBadge.tsx"]);
    Ok(())
}

/// Convention: snakecase.
#[test]
fn covers_snakecase() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "db_pool.py")?;
    touch(dir.path(), "dbPool.py")?;
    let config = load(
        r#"
lintp:
  config:
    .py: "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["dbPool.py"]);
    Ok(())
}

/// Convention: screamingsnakecase.
#[test]
fn covers_screaming_snake_case() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "SCHEMA_V2.sql")?;
    touch(dir.path(), "seed_data.sql")?;
    let config = load(
        r#"
lintp:
  config:
    .sql: "matches($BASENAME, /^[A-Z0-9]+(?:_[A-Z0-9]+)*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["seed_data.sql"]);
    Ok(())
}

/// Convention: kebabcase.
#[test]
fn covers_kebabcase() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "user-card.css")?;
    touch(dir.path(), "user_card.css")?;
    let config = load(
        r#"
lintp:
  config:
    .css: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["user_card.css"]);
    Ok(())
}

/// Convention: custom regex rule — implicitly anchored in that class of
/// tools, explicitly anchored here.
#[test]
fn covers_custom_regex() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "0001_create_users.sql")?;
    touch(dir.path(), "create_users.sql")?;
    let config = load(
        r#"
lintp:
  config:
    .sql: 'matches($BASENAME, /^\d{4}_[a-z_]+$/)'
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["create_users.sql"]);
    Ok(())
}

/// Convention: rule negation (! prefix).
#[test]
fn covers_negation() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "notes.md")?;
    touch(dir.path(), "temp-notes.md")?;
    let config = load(
        r#"
lintp:
  config:
    .md: '!contains($BASENAME, "temp")'
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["temp-notes.md"]);
    Ok(())
}

/// Convention: alternative styles combined with OR (any may match).
#[test]
fn covers_or_combination() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "AppShell.vue")?;
    touch(dir.path(), "app-shell.vue")?;
    touch(dir.path(), "app_shell.vue")?;
    let config = load(
        r#"
lintp:
  custom-matchers:
    kebab: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    pascal: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  config:
    .vue: "kebab || pascal"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["app_shell.vue"]);
    Ok(())
}

/// Convention: exact file-count restriction on a directory.
#[test]
fn covers_exists_exact_count() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "icons/arrow.svg")?;
    touch(dir.path(), "icons/close.svg")?;
    touch(dir.path(), "fonts/inter.woff")?;
    let config = load(
        r#"
lintp:
  config:
    "icons":
      .dir: 'count(children("*.svg")) == 2'
    "fonts":
      .dir: 'count(children("*.woff")) == 2'
  ignore: []
"#,
    )?;
    // icons has exactly 2 svgs (passes); fonts has 1 woff (fails)
    assert_eq!(failing_names(dir.path(), &config)?, ["fonts"]);
    Ok(())
}

/// Convention: file-count range restriction on a directory.
#[test]
fn covers_exists_range() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "locales/en.json")?;
    touch(dir.path(), "locales/de.json")?;
    let config = load(
        r#"
lintp:
  config:
    "locales":
      .dir: 'count(children("*.json")) >= 1 && count(children("*.json")) <= 3'
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, Vec::<String>::new());
    Ok(())
}

/// Convention: .dir rules — directory naming conventions.
#[test]
fn covers_dir_rules() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("api-client"))?;
    std::fs::create_dir(dir.path().join("My_Helpers"))?;
    let config = load(
        r#"
lintp:
  config:
    .dir: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["My_Helpers"]);
    Ok(())
}

/// Convention: sub-extensions — a longer suffix key wins
/// over the plain extension rule.
#[test]
fn covers_sub_extensions() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "Button.stories.tsx")?;
    touch(dir.path(), "Button.tsx")?;
    touch(dir.path(), "nav_item.stories.tsx")?; // stories must be Pascal too
    let config = load(
        r#"
lintp:
  custom-matchers:
    pascal-stories: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*\.stories$/)'
  config:
    .stories.tsx: "pascal-stories"
    .tsx: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(
        failing_names(dir.path(), &config)?,
        ["nav_item.stories.tsx"]
    );
    Ok(())
}

/// Convention: wildcard directory scoping.
#[test]
fn covers_wildcard_directory_scope() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "widgets/Toolbar.tsx")?;
    touch(dir.path(), "widgets/side-panel.tsx")?;
    touch(dir.path(), "lib/date-utils.tsx")?;
    let config = load(
        r#"
lintp:
  config:
    .tsx: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    "widgets/*":
      .tsx: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["side-panel.tsx"]);
    Ok(())
}

/// Convention: targeting a set of specific directories — expressed as
/// one path scope per directory (glob braces are not supported; the
/// outcome is identical).
#[test]
fn covers_brace_targeting_equivalent() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "api/auth/login_handler.go")?;
    touch(dir.path(), "api/billing/invoiceHandler.go")?;
    touch(dir.path(), "api/internal/whateverStyle.go")?;
    let config = load(
        r#"
lintp:
  custom-matchers:
    go-snake: "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)"
  config:
    "api/auth/*":
      .go: "go-snake"
    "api/billing/*":
      .go: "go-snake"
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["invoiceHandler.go"]);
    Ok(())
}

/// Convention: parent-directory references in rules — expressed via
/// $PARENT: a module entry file must be named after its directory.
#[test]
fn covers_parent_directory_reference() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "parser/parser.mod.ts")?;
    touch(dir.path(), "lexer/scanner.mod.ts")?;
    let config = load(
        r#"
lintp:
  config:
    .mod.ts: 'endsWith($PARENT, without($NAME, ".mod.ts"))'
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["scanner.mod.ts"]);
    Ok(())
}

/// Beyond name-rule linters: relational rules, locked in so our docs'
/// comparisons stay honest.
#[test]
fn covers_relational_rules() -> Result<()> {
    let dir = tempfile::tempdir()?;
    touch(dir.path(), "ui/Card.tsx")?;
    touch(dir.path(), "ui/Card.test.tsx")?;
    touch(dir.path(), "ui/Modal.tsx")?; // no test sibling
    let config = load(
        r#"
lintp:
  config:
    "ui/*":
      .test.tsx: "true"
      .tsx: 'in("${$BASENAME}.test.tsx", siblings("*.test.tsx"))'
  ignore: []
"#,
    )?;
    assert_eq!(failing_names(dir.path(), &config)?, ["Modal.tsx"]);
    Ok(())
}
