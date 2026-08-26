//! Rendering lint results for the CLI.
//!
//! Kept out of the library on purpose: [`LintResult`] is the stable thing a
//! library consumer matches on, while the shapes below are this binary's
//! output contract and are free to change with the CLI.

use anyhow::Result;
use colored::Colorize;
use lintp::lint::LintResult;
use lintp::util::forward_slashes;
use serde::Serialize;
use std::io::Write;

/// How results are rendered.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// Colored, one line per reported path, for reading in a terminal
    Human,
    /// A single JSON object, for CI annotations and editor integrations
    Json,
}

/// The whole run, whatever `--verbose` chose to list.
#[derive(Serialize)]
struct JsonReport<'a> {
    summary: Summary,
    results: Vec<JsonResult<'a>>,
}

/// Counts for the entire run, not just the reported subset.
#[derive(Serialize)]
struct Summary {
    checked: usize,
    passed: usize,
    failed: usize,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum JsonResult<'a> {
    Success {
        path: String,
    },
    Failure {
        path: String,
        rule: &'a str,
        message: &'a str,
    },
}

impl<'a> From<&'a LintResult> for JsonResult<'a> {
    fn from(result: &'a LintResult) -> Self {
        // Paths are forward-slashed so a consumer parsing this on Windows
        // sees the separators the DSL and the docs use.
        let slashed = |path: &std::path::Path| forward_slashes(&path.display().to_string());

        match result {
            LintResult::Success(path) => Self::Success {
                path: slashed(path),
            },
            LintResult::Failure {
                path,
                rule,
                message,
            } => Self::Failure {
                path: slashed(path),
                rule,
                message,
            },
        }
    }
}

/// Write `results` to `out` and report whether the run passed.
///
/// Failures are always reported; passing paths only when `verbose`, so a
/// clean run on a large project prints one line rather than one per file.
///
/// # Errors
///
/// Returns an error if `out` cannot be written to (a closed pipe, say), or if
/// the JSON report fails to serialize.
pub fn write_report<W: Write>(
    out: &mut W,
    results: &[LintResult],
    format: Format,
    verbose: bool,
) -> Result<bool> {
    let failed = results
        .iter()
        .filter(|r| matches!(r, LintResult::Failure { .. }))
        .count();

    // Which paths get listed is one decision, made here, rather than each
    // format re-deriving it and drifting apart.
    let listed: Vec<&LintResult> = results
        .iter()
        .filter(|r| verbose || matches!(r, LintResult::Failure { .. }))
        .collect();

    let counts = Summary {
        checked: results.len(),
        passed: results.len() - failed,
        failed,
    };

    match format {
        Format::Human => write_human(out, &listed, &counts)?,
        Format::Json => {
            let report = JsonReport {
                summary: counts,
                results: listed.iter().copied().map(JsonResult::from).collect(),
            };

            serde_json::to_writer_pretty(&mut *out, &report)?;
            writeln!(out)?;
        }
    }

    Ok(failed == 0)
}

fn write_human<W: Write>(out: &mut W, listed: &[&LintResult], counts: &Summary) -> Result<()> {
    for result in listed {
        match result {
            LintResult::Success(path) => writeln!(out, "{} {}", "✓".green(), path.display())?,
            LintResult::Failure {
                path,
                rule,
                message,
            } => writeln!(
                out,
                "{} {} - {} - {}",
                "✗".red(),
                path.display(),
                rule,
                message
            )?,
        }
    }

    let summary = if counts.failed == 0 {
        "All files and directories match the configured rules.".green()
    } else {
        "Some files or directories do not match the configured rules.".red()
    };

    writeln!(out, "{summary}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{write_report, Format};
    use lintp::lint::LintResult;
    use std::path::PathBuf;

    fn mixed() -> Vec<LintResult> {
        vec![
            LintResult::Success(PathBuf::from("./src/good-file.js")),
            LintResult::Failure {
                path: PathBuf::from("./src/badFile.js"),
                rule: ".js".to_string(),
                message: "Does not match rule: kebab-case".to_string(),
            },
        ]
    }

    fn render(format: Format, verbose: bool, results: &[LintResult]) -> (String, bool) {
        colored::control::set_override(false);
        let mut out = Vec::new();
        let success = write_report(&mut out, results, format, verbose).expect("writing to a Vec");

        (String::from_utf8(out).expect("valid utf8"), success)
    }

    #[test]
    fn human_output_hides_passing_paths_by_default() {
        let (out, success) = render(Format::Human, false, &mixed());

        assert!(!success);
        assert!(!out.contains("good-file.js"), "got: {out}");
        assert!(out.contains("✗ ./src/badFile.js - .js -"), "got: {out}");
    }

    #[test]
    fn human_output_lists_passing_paths_when_verbose() {
        let (out, _) = render(Format::Human, true, &mixed());

        assert!(out.contains("✓ ./src/good-file.js"), "got: {out}");
        assert!(out.contains("✗ ./src/badFile.js"), "got: {out}");
    }

    #[test]
    fn a_clean_run_still_says_so() {
        let clean = vec![LintResult::Success(PathBuf::from("./src/good-file.js"))];
        let (out, success) = render(Format::Human, false, &clean);

        assert!(success);
        assert_eq!(
            out,
            "All files and directories match the configured rules.\n"
        );
    }

    #[test]
    fn json_reports_failures_and_whole_run_counts() {
        let (out, success) = render(Format::Json, false, &mixed());
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json");

        assert!(!success);
        assert_eq!(parsed["summary"]["checked"], 2);
        assert_eq!(parsed["summary"]["passed"], 1);
        assert_eq!(parsed["summary"]["failed"], 1);

        let reported = parsed["results"].as_array().expect("an array");
        assert_eq!(reported.len(), 1, "passing paths are omitted: {out}");
        assert_eq!(reported[0]["status"], "failure");
        assert_eq!(reported[0]["path"], "./src/badFile.js");
        assert_eq!(reported[0]["rule"], ".js");
        assert_eq!(reported[0]["message"], "Does not match rule: kebab-case");
    }

    #[test]
    fn json_includes_passing_paths_when_verbose() {
        let (out, _) = render(Format::Json, true, &mixed());
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json");

        let reported = parsed["results"].as_array().expect("an array");
        assert_eq!(reported.len(), 2);
        assert_eq!(reported[0]["status"], "success");
        assert_eq!(reported[0]["path"], "./src/good-file.js");
        assert!(reported[0].get("rule").is_none(), "got: {out}");
    }

    #[test]
    fn json_paths_use_forward_slashes() {
        let windows_path = vec![LintResult::Failure {
            path: PathBuf::from(r"src\components\badName.tsx"),
            rule: ".tsx".to_string(),
            message: "nope".to_string(),
        }];
        let (out, _) = render(Format::Json, false, &windows_path);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json");

        assert_eq!(parsed["results"][0]["path"], "src/components/badName.tsx");
    }

    #[test]
    fn json_of_a_clean_run_has_an_empty_result_list() {
        let clean = vec![LintResult::Success(PathBuf::from("./src/good-file.js"))];
        let (out, success) = render(Format::Json, false, &clean);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json");

        assert!(success);
        assert_eq!(parsed["summary"]["failed"], 0);
        assert!(parsed["results"].as_array().expect("an array").is_empty());
    }
}
