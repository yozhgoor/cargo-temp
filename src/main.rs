use anyhow::Result;
use clap::Parser;
use std::{env, fs};

mod config;
mod dependency;
mod run;

use crate::{config::*, dependency::*, run::*};

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`.
#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow=1.0.13`
    #[clap(parse(try_from_str = parse_dependency))]
    pub dependencies: Vec<Dependency>,

    /// Create a library instead of a binary.
    #[clap(long)]
    pub lib: bool,

    /// Name of the temporary crate.
    #[clap(long = "name")]
    pub project_name: Option<String>,

    /// Create a temporary Git working tree based on the repository in the
    /// current directory.
    #[clap(long = "worktree")]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[clap(long)]
    pub git: Option<String>,

    /// Add a `benches` to the temporary project.
    ///
    /// You can choose the name of the benchmark file name as argument.
    /// The default is `benchmark.rs`
    #[clap(long)]
    pub bench: Option<Option<String>>,

    /// Select the Rust's edition of the temporary project.
    ///
    /// Available options are:
    /// * 15 | 2015 => edition 2015,
    /// * 18 | 2018 => edition 2018,
    /// * 21 | 2021 => edition 2021,
    ///
    /// If the argument doesn't match any of the options, the default is the latest edition
    #[clap(long)]
    pub edition: Option<u32>,
}

fn main() -> Result<()> {
    env_logger::init();

    // Parse the command line input.
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");

    let cli = Cli::parse_from(command.into_iter().chain(args));

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create().expect("cannot get config");
    let _ = fs::create_dir(&config.temporary_project_dir);

    execute(cli, config)?;

    Ok(())
}
