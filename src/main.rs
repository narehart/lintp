use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod config;
mod dsl;
mod lint;

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

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Find config file
    let config_path = match cli.config {
        Some(path) => path,
        None => {
            let default_path = PathBuf::from("lintp.yml");
            if !default_path.exists() {
                eprintln!(
          "No config file found. Use --config to specify a config file path or create lintp.yml in the current directory."
        );
                std::process::exit(1);
            }
            default_path
        }
    };

    // Load and parse config
    let config = config::load_config(&config_path)?;

    // Run the linter
    let results = lint::run_lint(&cli.dir, &config, cli.verbose)?;

    // Report results
    let success = report_results(&results);

    if !success {
        std::process::exit(1);
    }

    Ok(())
}

fn report_results(results: &[lint::LintResult]) -> bool {
    use colored::Colorize;

    let mut success = true;

    for result in results {
        match result {
            lint::LintResult::Success(path) => {
                println!("{} {}", "✓".green(), path.display());
            }
            lint::LintResult::Failure {
                path,
                rule,
                message,
            } => {
                success = false;
                println!("{} {} - {} - {}", "✗".red(), path.display(), rule, message);
            }
        }
    }

    if success {
        println!(
            "{}",
            "All files and directories match the configured rules.".green()
        );
    } else {
        println!(
            "{}",
            "Some files or directories do not match the configured rules.".red()
        );
    }

    success
}
