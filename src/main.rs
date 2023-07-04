use anyhow::Result;
use clap::Parser;
use std::{env, fs::create_dir, io::Write};

#[cfg(windows)]
mod binding;

mod cli;
mod config;
mod dependency;
mod project;
mod subprocess;

use crate::{cli::Cli, config::Config, project::Project};

fn main() -> Result<()> {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{} cargo-temp] {}", record.level(), record.args()))
        .filter(Some("cargo_temp"), log::LevelFilter::Info)
        .init();

    // Parse the command line input.
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");

    let cli = Cli::parse_from(command.into_iter().chain(args));

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create()?;
    let _ = create_dir(&config.temporary_project_dir);

    Project::execute(cli, config)
}
