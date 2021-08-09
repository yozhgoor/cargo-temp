use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // Parse the command line input.
    let cli = cargo_temp::Cli::new();

    // Read configuration from disk or generate a default one.
    let config = cargo_temp::Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);

    cargo_temp::run(cli, &config)?;

    Ok(())
}
