//! Structured error type for lintp's library API.

use std::path::PathBuf;

/// Errors returned by lintp's public library API: [`crate::config::load_config`],
/// [`crate::lint::run_lint`], and the `dsl` module functions ([`crate::dsl::parser::parse_expression`],
/// [`crate::dsl::evaluator::evaluate`]) they build on.
///
/// The `lintp` binary doesn't match on these variants — it converts them to
/// `anyhow::Error` with `?` and prints the message — but library consumers
/// embedding lintp can match on the failure kind instead of parsing message
/// text.
///
/// Marked `#[non_exhaustive]`: new variants may be added in a minor release.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The config file does not exist at the given path.
    #[error("Config file not found: {path}")]
    ConfigNotFound {
        /// The path that was passed to `load_config`.
        path: PathBuf,
    },

    /// The config file exists but could not be read (permission denied, a
    /// directory instead of a file, etc).
    #[error("Failed to read config file: {path}: {source}")]
    Io {
        /// The path that could not be read.
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The config file is not valid YAML, or doesn't match the expected
    /// `lintp.yml` shape: unknown fields, a rule key that isn't an
    /// extension pattern, an empty path scope, an invalid path-scope glob,
    /// an unbalanced brace group, etc.
    #[error("Failed to parse config file: {path}: {source}")]
    ConfigParse {
        /// The config file that failed to parse.
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    /// An `ignore:` pattern or a path-rule scope glob failed to compile.
    #[error("Invalid {kind} '{pattern}': {source}")]
    Glob {
        /// What kind of glob this was (e.g. `"ignore pattern"`,
        /// `"path-rule glob"`), for a message that names the right thing.
        kind: &'static str,
        /// The pattern text that failed to compile.
        pattern: String,
        #[source]
        source: glob::PatternError,
    },

    /// A rule or custom-matcher DSL expression failed to parse, or failed
    /// to evaluate against a file: a syntax error, an unknown
    /// variable/matcher reference, a circular custom-matcher reference, a
    /// built-in function called with the wrong argument count/type, etc.
    #[error("{0}")]
    Dsl(String),

    /// An internal invariant was violated while walking the filesystem
    /// (e.g. a visited path unexpectedly had no file name). Should not
    /// occur in practice; reported rather than panicking.
    #[error("{0}")]
    Internal(String),
}
