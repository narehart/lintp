//! The `lintp` command-line entry point: resolves the config file, runs the
//! lint, prints the results, and sets the exit code.

use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;

use lintp::{config, lint};

mod report;
use report::Format;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "File system linter with DSL",
    after_help = "Docs: https://narehart.github.io/lintp/"
)]
struct Cli {
    /// Path to the lintp.yml config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Directory to lint
    #[arg(value_name = "DIR", default_value = ".")]
    dir: PathBuf,

    /// Show passing files as well as failures
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    format: Format,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = if let Some(path) = cli.config {
        path
    } else {
        let default_path = PathBuf::from("lintp.yml");
        if !default_path.exists() {
            bail!(
                "No config file found. Use --config to specify a config file path or create lintp.yml in the current directory."
            );
        }
        default_path
    };

    let config = config::load_config(&config_path)?;
    // Not cli.verbose: run_lint's own verbose flag prints "Checking: <path>"
    // to stdout, which would sit in front of the JSON document and break any
    // consumer parsing it. Verbosity is a reporting concern, and the ✓ lines
    // below already say every path that was checked.
    let results = lint::run_lint(&cli.dir, &config, false)?;

    // Locked once rather than per line: a large --verbose run is thousands of
    // writes, and stdout's lock is re-acquired on every one otherwise.
    let stdout = std::io::stdout();
    let success = report::write_report(&mut stdout.lock(), &results, cli.format, cli.verbose)?;

    if !success {
        std::process::exit(1);
    }

    Ok(())
}
