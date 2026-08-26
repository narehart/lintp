//! Tests for the errors `lintp.yml` loading reports.
//!
//! A config mistake that loads quietly is worse than one that fails: the lint
//! passes while enforcing nothing. Each of these asserts the message names the
//! offending key, so the fix is obvious from the output alone.

use anyhow::Result;
use std::path::PathBuf;

use lintp::config::load_config;

struct Written {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

fn write(content: &str) -> Result<Written> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("lintp.yml");
    std::fs::write(&path, content)?;

    Ok(Written { _dir: dir, path })
}

/// Load `content` and return the error it produced.
fn load_err(content: &str) -> String {
    let written = write(content).expect("failed to write the temp config");
    match load_config(&written.path) {
        Ok(_) => panic!("expected the config to be rejected:\n{content}"),
        Err(e) => format!("{e:#}"),
    }
}

#[test]
fn rejects_a_config_block_that_is_not_a_map() {
    let err = load_err("lintp:\n  config: \"just a string\"\n");
    assert!(err.contains("Expected a map for config"), "got: {err}");
}

#[test]
fn rejects_non_string_keys() {
    let err = load_err("lintp:\n  config:\n    123: \"true\"\n");
    assert!(err.contains("Config keys must be strings"), "got: {err}");
}

#[test]
fn rejects_a_rule_value_that_is_neither_string_nor_map() {
    let err = load_err("lintp:\n  config:\n    .js: 42\n");
    assert!(
        err.contains("must be a string or a map"),
        "the message should name the accepted shapes, got: {err}"
    );
}

#[test]
fn rejects_an_unknown_option_inside_a_rule_entry() {
    let err =
        load_err("lintp:\n  config:\n    .js:\n      rule: \"true\"\n      colour: \"red\"\n");
    assert!(err.contains("Unknown option 'colour'"), "got: {err}");
    assert!(
        err.contains("'rule' or 'message'"),
        "the message should list what is accepted, got: {err}"
    );
}

/// Inside a path scope every mapping is a rule entry, so one without a
/// `rule:` is a mistake rather than a nested scope.
#[test]
fn rejects_a_scoped_rule_entry_with_no_rule_field() {
    let err =
        load_err("lintp:\n  config:\n    \"src/*\":\n      .js:\n        message: \"nope\"\n");
    assert!(err.contains("missing the 'rule' field"), "got: {err}");
}

/// At the top level the distinction is the `rule:` key: a mapping without one
/// is read as a path scope, so its keys are validated as rule keys.
#[test]
fn treats_a_top_level_mapping_without_rule_as_a_path_scope() {
    let err = load_err("lintp:\n  config:\n    .js:\n      message: \"nope\"\n");
    assert!(
        err.contains("Invalid rule key 'message' under path scope '.js'"),
        "got: {err}"
    );
}

#[test]
fn rejects_a_non_string_message() {
    let err = load_err("lintp:\n  config:\n    .js:\n      rule: \"true\"\n      message: 7\n");
    assert!(err.contains("must be a string"), "got: {err}");
}

#[test]
fn rejects_a_path_scope_with_no_rules() {
    let err = load_err("lintp:\n  config:\n    \"src/*\": {}\n");
    assert!(err.contains("has no rules"), "got: {err}");
}

#[test]
fn rejects_a_path_scope_whose_glob_does_not_compile() {
    let err = load_err("lintp:\n  config:\n    \"src/[\":\n      .js: \"true\"\n");
    assert!(
        err.contains("Invalid glob pattern for path scope"),
        "got: {err}"
    );
}

#[test]
fn rejects_a_rule_referring_to_a_matcher_that_does_not_exist() {
    let err = load_err("lintp:\n  config:\n    .js: \"keba-case\"\n");
    assert!(err.contains("Unknown matcher 'keba-case'"), "got: {err}");
}

#[test]
fn rejects_matchers_that_reference_each_other_in_a_cycle() {
    let err = load_err(
        "lintp:\n  custom-matchers:\n    a: \"b\"\n    b: \"a\"\n  config:\n    .js: \"a\"\n",
    );
    assert!(err.contains("Circular reference"), "got: {err}");
}

#[test]
fn rejects_a_matcher_named_after_a_boolean_literal() {
    let err = load_err(
        "lintp:\n  custom-matchers:\n    true: \"$EXT == \\\"js\\\"\"\n  config:\n    .js: \"true\"\n",
    );
    assert!(
        err.contains("shadowed by the boolean literal"),
        "got: {err}"
    );
}

#[test]
fn rejects_a_rule_that_does_not_parse() {
    let err = load_err("lintp:\n  config:\n    .js: \"kebab-case &&\"\n");
    assert!(err.contains("Failed to parse"), "got: {err}");
}

#[test]
fn rejects_a_matcher_that_does_not_parse() {
    let err = load_err(
        "lintp:\n  custom-matchers:\n    broken: \"matches($NAME\"\n  config:\n    .js: \"broken\"\n",
    );
    assert!(
        err.contains("Failed to parse matcher: broken"),
        "got: {err}"
    );
}

#[test]
fn reports_a_missing_config_file_rather_than_defaulting() {
    let dir = tempfile::tempdir().expect("temp dir");
    let missing = dir.path().join("nope.yml");

    let err = match load_config(&missing) {
        Ok(_) => panic!("a missing config file should not load"),
        Err(e) => format!("{e:#}"),
    };
    assert!(
        err.contains("No config file found") || err.contains("not found"),
        "a missing file gets its own error, not a read failure, got: {err}"
    );
}

#[test]
fn rejects_yaml_that_is_not_valid() {
    let err = load_err("lintp:\n  config:\n    .js: 'unterminated\n");
    assert!(err.contains("Failed to parse config file"), "got: {err}");
}

#[test]
fn rejects_unknown_top_level_fields() {
    let err = load_err("lintp:\n  config:\n    .js: \"true\"\n  extra: 1\n");
    assert!(err.contains("Failed to parse config file"), "got: {err}");
}
