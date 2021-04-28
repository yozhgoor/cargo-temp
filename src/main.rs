use anyhow::{bail, Context, Result};
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{fs, process};
use tempfile::Builder;

/// This tool allow you to create a new Rust temporary project in a temporary directory.
///
/// The dependencies can be provided in arguments (e.g. `cargo-temp anyhow tokio`).
/// When the shell is exited, the temporary directory is deleted
/// unless you removed the file `TO_DELETE`
#[derive(Clap, Debug)]
struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow==1.0.13`
    #[clap(parse(from_str = parse_dependency))]
    dependencies: Vec<(String, Option<String>)>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    build_dir: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Config { build_dir: "" }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_dir = dirs::config_dir()
        .context("Could not get cache directory")?
        .join(env!("CARGO_PKG_NAME"));
    let _ = fs::create_dir_all(&config_dir);

    let cache_dir = dirs::cache_dir()
        .context("Could not get cache directory")?
        .join(env!("CARGO_PKG_NAME"));
    let _ = fs::create_dir_all(&cache_dir);

    let tmp_dir = Builder::new().prefix("tmp-").tempdir_in(&cache_dir)?;

    if !process::Command::new("cargo")
        .current_dir(&tmp_dir)
        .arg("init")
        .status()
        .context("Could not start cargo")?
        .success()
    {
        bail!("Cargo command failed");
    }

    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(&tmp_dir.path().join("Cargo.toml"))?;
    for (s, v) in cli.dependencies.iter() {
        match &v {
            Some(version) => writeln!(toml, "{} = \"{}\"", s, version)?,
            None => writeln!(toml, "{} = \"*\"", s)?,
        }
    }
    drop(toml);

    process::Command::new(get_shell())
        .current_dir(&tmp_dir)
        .status()
        .context("Cannot start shell")?;

    Ok(())
}

fn get_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }

    #[cfg(not(unix))]
    {
        compile_error!(
            "Only unix systems supported at the moment. \
            Help would be appreciated =D"
        )
    }
}

fn parse_dependency(s: &str) -> (String, Option<String>) {
    let mut it = s.splitn(2, '=').map(|x| x.to_string());
    (it.next().unwrap(), it.next())
}
