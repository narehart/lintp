use anyhow::{ Context as AnyhowContext, Result };
use std::path::PathBuf;
use std::process::Command;

mod test_constants;
use test_constants::*;

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
    "// Button component"
  )?;
  std::fs::write(root_path.join("src").join("components").join("Card.js"), "// Card component")?;
  std::fs::write(root_path.join("src").join("utils").join("format-date.js"), "// Date formatter")?;
  std::fs::write(root_path.join("src").join("api").join("users.js"), "// Users API")?;
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
    "// Invalid filename - should be kebab-case or PascalCase"
  )?;
  std::fs::write(
    test_project.root_path.join("src").join("components").join("select.js"),
    "// Invalid PascalCase for component"
  )?;
  std::fs::write(
    test_project.root_path.join("src").join("utils").join("DateFormatter.js"),
    "// Invalid kebab-case for utility"
  )?;
  std::fs::write(
    test_project.root_path.join("src").join("api").join("INVALID-API.js"),
    "// Invalid camelCase for API"
  )?;
  std::fs::write(
    test_project.root_path.join("tests").join("app.js"),
    "// Missing .test in filename"
  )?;

  Ok(test_project)
}

/// Build the binary for testing
fn build_binary() -> Result<PathBuf> {
  // Get the project root directory
  let project_root = std::env::current_dir()?;

  // Run cargo build
  let status = Command::new("cargo").current_dir(&project_root).args(["build"]).status()?;

  if !status.success() {
    return Err(anyhow::anyhow!("Failed to build binary"));
  }

  // Construct the path to the binary
  let binary_path = project_root.join("target").join("debug").join("lintp");

  // Verify the binary exists
  if !binary_path.exists() {
    // Try to list files in the directory to see what's available
    let debug_dir = binary_path.parent().unwrap();
    let entries = std::fs
      ::read_dir(debug_dir)
      .with_context(|| format!("Failed to read directory: {}", debug_dir.display()))?;

    let files: Vec<String> = entries
      .filter_map(Result::ok)
      .map(|entry| entry.file_name().to_string_lossy().to_string())
      .collect();

    return Err(
      anyhow::anyhow!(
        "Binary not found at {}. Files in {}: {:?}",
        binary_path.display(),
        debug_dir.display(),
        files
      )
    );
  }

  Ok(binary_path)
}

/// Integration test for a valid project
#[test]
fn test_valid_project() -> Result<()> {
  let test_project = create_test_project()?;
  let binary_path = build_binary()?;

  let output = Command::new(&binary_path).current_dir(&test_project.root_path).output()?;

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
    "Expected success message, got: {}",
    stdout
  );

  Ok(())
}

/// Integration test for a project with errors
#[test]
fn test_project_with_errors() -> Result<()> {
  let test_project = create_test_project_with_errors()?;
  let binary_path = build_binary()?;

  let output = Command::new(&binary_path).current_dir(&test_project.root_path).output()?;

  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();

  // Check that the command failed (non-zero exit code)
  assert!(!output.status.success(), "Command should have failed but succeeded");

  // Check that the output contains error messages for each invalid file
  assert!(
    stdout.contains("INVALID-CASE.js") || stderr.contains("INVALID-CASE.js"),
    "Should report INVALID-CASE.js"
  );
  assert!(stdout.contains("select.js") || stderr.contains("select.js"), "Should report select.js");
  assert!(
    stdout.contains("DateFormatter.js") || stderr.contains("DateFormatter.js"),
    "Should report DateFormatter.js"
  );
  assert!(
    stdout.contains("INVALID-API.js") || stderr.contains("INVALID-API.js"),
    "Should report INVALID-API.js"
  );
  assert!(stdout.contains("app.js") || stderr.contains("app.js"), "Should report app.js");

  // Check that the output contains the failure message
  assert!(
    stdout.contains("Some files or directories do not match the configured rules") ||
      stderr.contains("Some files or directories do not match the configured rules"),
    "Expected failure message"
  );

  Ok(())
}

/// Integration test with custom config path
#[test]
fn test_with_custom_config_path() -> Result<()> {
  let test_project = create_test_project()?;
  let binary_path = build_binary()?;

  // Move the config file to a different location
  let custom_config_path = test_project.root_path.join("custom-config.yml");
  std::fs::rename(test_project.root_path.join("lintp.yml"), &custom_config_path)?;

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
    "Expected success message, got: {}",
    stdout
  );

  Ok(())
}

/// Integration test with verbose output
#[test]
fn test_with_verbose_output() -> Result<()> {
  let test_project = create_test_project()?;
  let binary_path = build_binary()?;

  let output = Command::new(&binary_path)
    .current_dir(&test_project.root_path)
    .args(["--verbose"])
    .output()?;

  let stdout = String::from_utf8_lossy(&output.stdout).to_string();

  // Check that the output contains "Checking" lines
  assert!(stdout.contains("Checking:"), "Should have verbose output with 'Checking' lines");

  // Count the number of "Checking" lines
  let checking_count = stdout
    .lines()
    .filter(|line| line.contains("Checking:"))
    .count();

  // Should be checking at least 10 items (including directories and files)
  assert!(checking_count >= 10, "Should check at least 10 items, found {}", checking_count);

  Ok(())
}

/// Integration test with missing config
#[test]
fn test_with_missing_config() -> Result<()> {
  let test_project = create_test_project()?;
  let binary_path = build_binary()?;

  // Remove the config file
  std::fs::remove_file(test_project.root_path.join("lintp.yml"))?;

  let output = Command::new(&binary_path).current_dir(&test_project.root_path).output()?;

  let stderr = String::from_utf8_lossy(&output.stderr).to_string();

  // Check that the command failed
  assert!(!output.status.success(), "Command should have failed with missing config");

  // Check that the output contains error message about missing config
  assert!(
    stderr.contains("No config file found"),
    "Should report missing config file, got: {}",
    stderr
  );

  Ok(())
}
