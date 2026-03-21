use cargo_declared::{CargoDeclared, Error};
use clap::Parser;
use std::path::PathBuf;

/// Audit the gap between declared and compiled dependencies
#[derive(Parser, Debug)]
#[command(name = "cargo-declared")]
#[command(about = "Audit the gap between declared and compiled dependencies", long_about = None)]
struct Cli {
    /// Path to the Cargo.toml file
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Show all four sets of dependencies
    #[arg(short, long)]
    full: bool,

    /// Output in JSON format
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match (cli.json, cli.full) {
        (true, true) => run_full_json(cli.path),
        (true, false) => run_json(cli.path),
        (false, true) => run_full_human(cli.path),
        (false, false) => run_human(cli.path),
    };

    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_human(path: Option<PathBuf>) -> Result<String, Error> {
    let tool = path.map_or_else(CargoDeclared::new, |path| {
        CargoDeclared::new().with_path(path)
    });
    tool.run_human()
}

fn run_json(path: Option<PathBuf>) -> Result<String, Error> {
    let tool = path.map_or_else(CargoDeclared::new, |path| {
        CargoDeclared::new().with_path(path)
    });
    tool.run_json()
}

fn run_full_human(path: Option<PathBuf>) -> Result<String, Error> {
    run_human(path)
}

fn run_full_json(path: Option<PathBuf>) -> Result<String, Error> {
    run_json(path)
}
