//! lintp is a filesystem linter: it validates file and directory naming
//! conventions across a project against rules written in a small DSL,
//! configured in a `lintp.yml` file.
//!
//! Rules are expressions (e.g. `matches($BASENAME, /^[a-z-]+$/)`) scoped by
//! file extension and, optionally, by path glob, so different parts of a
//! project can enforce different conventions.
//!
//! Fallible library functions return [`Error`], a `thiserror`-based enum
//! that library consumers can match on to distinguish failure kinds
//! (a missing config file, a bad rule expression, etc.) instead of parsing
//! message text.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use lintp::{config, lint};
//!
//! # fn main() -> anyhow::Result<()> {
//! let parsed_config = config::load_config(Path::new("lintp.yml"))?;
//! let results = lint::run_lint(Path::new("."), &parsed_config, false)?;
//!
//! for result in &results {
//!     println!("{:?}", result);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! See <https://narehart.github.io/lintp/> for the full DSL reference.

// Export modules so they can be imported by tests
pub mod config;
pub mod dsl;
mod error;
pub mod lint;
pub mod util;

pub use error::Error;
