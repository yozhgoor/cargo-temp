use anyhow::{bail, Context, Result};
use clap::Clap;
use std::io::Write;
use std::{fs, process};
use tempfile::Builder;

#[derive(Clap, Debug)]
struct Cli {
    #[clap(parse(from_str = parse_dependency))]
    dependencies: Vec<(String, Option<String>)>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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

    let mut delete_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_dir.path().join("to_delete.txt"))?;
    writeln!(
        delete_file,
        "Delete this file if you want to save this project"
    )?;
    drop(delete_file);

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
