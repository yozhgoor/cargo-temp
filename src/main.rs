use anyhow::Result;
use cargo_temp::config::Config;
use cargo_temp::Cli;
use clap::Parser;
use std::{env, fs};

fn main() -> Result<()> {
    // Parse the command line input.
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");

    let cli = Cli::parse_from(command.into_iter().chain(args));

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);

    cli.run(&config)?;

    Ok(())
}
