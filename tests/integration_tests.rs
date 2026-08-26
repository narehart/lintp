//! End-to-end tests that run the compiled `lintp` binary against a
//! temporary project and assert on its output and exit code.

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

mod common;
use common::constants::*;

/// Structure to hold both the temporary directory and the path
struct TestProject {
    _temp_dir: tempfile::TempDir, // Underscore prefix indicates it's kept for its lifetime
    root_path: PathBuf,
}

/// Helper function to create a temporary test project
fn create_test_project() -> Result<TestProject> {
    let temp_dir: tempfile::TempDir = tempfile::tempdir()?;
    let root_path: PathBuf = temp_dir.path().to_path_buf();

    // Create project structure
    std::fs::create_dir(root_path.join("src"))?;
    std::fs::create_dir(root_path.join("src").join("components"))?;
    std::fs::create_dir(root_path.join("src").join("utils"))?;
    std::fs::create_dir(root_path.join("src").join("api"))?;
    std::fs::create_dir(root_path.join("tests"))?;
    std::fs::create_dir(root_path.join("dist"))?;
    std::fs::create_dir(root_path.join("node_modules"))?;

    // Create some files
    std::fs::write(root_path.join("src").join("index.js"), "// Entry point")?;
    std::fs::write(
        root_path.join("src").join("components").join("Button.js"),
        "// Button component",
    )?;
    std::fs::write(
        root_path.join("src").join("components").join("Card.js"),
        "// Card component",
    )?;
    std::fs::write(
        root_path.join("src").join("utils").join("format-date.js"),
        "// Date formatter",
    )?;
    std::fs::write(
        root_path.join("src").join("api").join("users.js"),
        "// Users API",
    )?;
    std::fs::write(root_path.join("tests").join("app.test.js"), "// App tests")?;

    // Create config file using standardized patterns
    let config_content = create_standard_test_config();
    std::fs::write(root_path.join("lintp.yml"), config_content)?;

    Ok(TestProject {
        _temp_dir: temp_dir,
        root_path,
    })
}

/// Helper function to create a test project with invalid files
fn create_test_project_with_errors() -> Result<TestProject> {
    let test_project = create_test_project()?;

    // Create files that violate rules
    std::fs::write(
        test_project.root_path.join("src").join("INVALID-CASE.js"),
        "// Invalid filename - should be kebab-case or PascalCase",
    )?;
    std::fs::write(
        test_project
            .root_path
            .join("src")
            .join("components")
            .join("select.js"),
        "// Invalid PascalCase for component",
    )?;
    std::fs::write(
        test_project
            .root_path
            .join("src")
            .join("utils")
            .join("DateFormatter.js"),
        "// Invalid kebab-case for utility",
    )?;
    std::fs::write(
        test_project
            .root_path
            .join("src")
            .join("api")
            .join("INVALID-API.js"),
        "// Invalid camelCase for API",
    )?;
    std::fs::write(
        test_project.root_path.join("tests").join("app.js"),
        "// Missing .test in filename",
    )?;

    Ok(test_project)
}

/// Integration test for a valid project
#[test]
fn test_valid_project() -> Result<()> {
    let test_project = create_test_project()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status;

    // Check that the command was successful
    assert!(
        output.status.success(),
        "Command failed with status {:?}.\nSTDOUT:\n{}\nSTDERR:\n{}",
        status.code(),
        stdout,
        stderr
    );

    // Check that the output contains success message
    assert!(
        stdout.contains("All files and directories match the configured rules"),
        "Expected success message, got: {stdout}"
    );

    Ok(())
}

/// Integration test for a project with errors
#[test]
fn test_project_with_errors() -> Result<()> {
    let test_project = create_test_project_with_errors()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check that the command failed (non-zero exit code)
    assert!(
        !output.status.success(),
        "Command should have failed but succeeded"
    );

    // Check that the output contains error messages for each invalid file
    assert!(
        stdout.contains("INVALID-CASE.js") || stderr.contains("INVALID-CASE.js"),
        "Should report INVALID-CASE.js"
    );
    assert!(
        stdout.contains("select.js") || stderr.contains("select.js"),
        "Should report select.js"
    );
    assert!(
        stdout.contains("DateFormatter.js") || stderr.contains("DateFormatter.js"),
        "Should report DateFormatter.js"
    );
    assert!(
        stdout.contains("INVALID-API.js") || stderr.contains("INVALID-API.js"),
        "Should report INVALID-API.js"
    );
    assert!(
        stdout.contains("app.js") || stderr.contains("app.js"),
        "Should report app.js"
    );

    // Check that the output contains the failure message
    assert!(
        stdout.contains("Some files or directories do not match the configured rules")
            || stderr.contains("Some files or directories do not match the configured rules"),
        "Expected failure message"
    );

    Ok(())
}

/// Integration test with custom config path
#[test]
fn test_with_custom_config_path() -> Result<()> {
    let test_project = create_test_project()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    // Move the config file to a different location
    let custom_config_path = test_project.root_path.join("custom-config.yml");
    std::fs::rename(
        test_project.root_path.join("lintp.yml"),
        &custom_config_path,
    )?;

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .args(["--config", "custom-config.yml"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status;

    // Check that the command was successful
    assert!(
        output.status.success(),
        "Command failed with status {:?}.\nSTDOUT:\n{}\nSTDERR:\n{}",
        status.code(),
        stdout,
        stderr
    );

    // Check that the output contains success message
    assert!(
        stdout.contains("All files and directories match the configured rules"),
        "Expected success message, got: {stdout}"
    );

    Ok(())
}

/// Verbose lists every path checked, not just the failures.
#[test]
fn test_with_verbose_output() -> Result<()> {
    let test_project = create_test_project()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .args(["--verbose"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let checked = stdout.lines().filter(|line| line.contains('✓')).count();

    assert!(
        checked >= 10,
        "should list every checked path, found {checked}:\n{stdout}"
    );

    Ok(())
}

/// The default output is failures only: a clean run on a large project should
/// print a single line, not one per file.
#[test]
fn test_default_output_lists_only_failures() -> Result<()> {
    let test_project = create_test_project_with_errors()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        !stdout.contains('✓'),
        "passing paths should be hidden by default:\n{stdout}"
    );
    assert!(stdout.contains("INVALID-CASE.js"), "got:\n{stdout}");

    Ok(())
}

/// A clean run says so in one line rather than listing the whole tree.
#[test]
fn test_clean_run_prints_one_line() -> Result<()> {
    let test_project = create_test_project()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected a single summary line, got:\n{stdout}"
    );

    Ok(())
}

/// The JSON document has to be the only thing on stdout, or a consumer
/// piping it into a parser gets a syntax error.
#[test]
fn test_json_output_is_parseable() -> Result<()> {
    let test_project = create_test_project_with_errors()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    for args in [
        vec!["--format", "json"],
        vec!["--format", "json", "--verbose"],
    ] {
        let output = Command::new(&binary_path)
            .current_dir(&test_project.root_path)
            .args(&args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("{args:?} did not produce JSON: {e}\n{stdout}"));

        let summary = &parsed["summary"];
        assert!(summary["checked"].as_u64().expect("checked") > 0);
        assert!(summary["failed"].as_u64().expect("failed") > 0);

        let reported = parsed["results"].as_array().expect("results array");
        assert!(reported.iter().any(|r| r["status"] == "failure"));
    }

    Ok(())
}

/// Exit code is the CI contract and must not depend on the format.
#[test]
fn test_exit_code_is_the_same_in_both_formats() -> Result<()> {
    let clean = create_test_project()?;
    let failing = create_test_project_with_errors()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    for format in ["human", "json"] {
        let ok = Command::new(&binary_path)
            .current_dir(&clean.root_path)
            .args(["--format", format])
            .output()?;
        assert!(ok.status.success(), "{format} should exit 0 on a clean run");

        let bad = Command::new(&binary_path)
            .current_dir(&failing.root_path)
            .args(["--format", format])
            .output()?;
        assert_eq!(
            bad.status.code(),
            Some(1),
            "{format} should exit 1 on a violation"
        );
    }

    Ok(())
}

/// Integration test with missing config
#[test]
fn test_with_missing_config() -> Result<()> {
    let test_project = create_test_project()?;
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_lintp"));

    // Remove the config file
    std::fs::remove_file(test_project.root_path.join("lintp.yml"))?;

    let output = Command::new(&binary_path)
        .current_dir(&test_project.root_path)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check that the command failed
    assert!(
        !output.status.success(),
        "Command should have failed with missing config"
    );

    // Check that the output contains error message about missing config
    assert!(
        stderr.contains("No config file found"),
        "Should report missing config file, got: {stderr}"
    );

    Ok(())
}
