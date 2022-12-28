use anyhow::Result;
use clap::Parser;
use std::{env, fs, io::Write};

mod config;
mod dependency;
mod generate;
mod run;

use crate::{config::*, dependency::*, run::*};

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow=1.0.13`
    #[arg(value_parser = parse_dependency)]
    pub dependencies: Vec<Dependency>,

    /// Create a library instead of a binary.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Name of the temporary crate.
    #[arg(long = "name", short = 'n')]
    pub project_name: Option<String>,

    /// Create a temporary Git working tree based on the repository in the
    /// current directory.
    #[arg(long = "worktree", short = 'w')]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[arg(long, short = 'g')]
    pub git: Option<String>,

    /// Add a `benches` to the temporary project.
    ///
    /// You can choose the name of the benchmark file name as argument.
    /// The default is `benchmark.rs`
    #[arg(long, short = 'b')]
    pub bench: Option<Option<String>>,

    /// Select the Rust's edition of the temporary project.
    ///
    /// Available options are:
    /// * 15 | 2015 => edition 2015,
    /// * 18 | 2018 => edition 2018,
    /// * 21 | 2021 => edition 2021,
    ///
    /// If the argument doesn't match any of the options, the default is the latest edition
    #[arg(long, short = 'e')]
    pub edition: Option<u32>,

    #[cfg(feature = "generate")]
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Generate a temporary project from a template using `cargo-generate`.
    #[cfg(feature = "generate")]
    Generate(generate::Args),
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter(Some("cargo_temp"), log::LevelFilter::Info)
        .format(|buf, record| writeln!(buf, "[{} cargo-temp] {}", record.level(), record.args()))
        .init();

    // Parse the command line input.
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");

    let cli = Cli::parse_from(command.into_iter().chain(args));

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);

    #[cfg(feature = "generate")]
    match cli.subcommand {
        Some(Subcommand::Generate(args)) => args.generate(config)?,
        None => execute(cli, config)?,
    }

    #[cfg(not(feature = "generate"))]
    execute(cli, config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
