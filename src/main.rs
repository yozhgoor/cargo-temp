use anyhow::Result;
use cargo_temp::config::Config;
use cargo_temp::run;
use std::fs;

fn main() -> Result<()> {
    // Parse the command line input.
    let cli = cargo_temp::Cli::new();

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);

    run::start(cli, &config)?;

    Ok(())
}
