use anyhow::{bail, Context, Result};
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env, fs, process};
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
    temporary_project_path: String,
    cargo_target_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir().context("Could not get cache directory");
        Config {
            temporary_project_path: cache_dir
                .unwrap()
                .to_str()
                .expect("Could not convert cache path into str")
                .to_string(),
            cargo_target_dir: None,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_dir = dirs::config_dir()
        .context("Could not get config directory")?
        .join(env!("CARGO_PKG_NAME"));
    let _ = fs::create_dir_all(&config_dir);

    let config_file_path = config_dir.join("config.toml");

    let config: Config = match fs::read(&config_file_path) {
        Ok(file) => toml::de::from_slice(&file[..])?,
        Err(_) => {
            let config = Config::default();
            fs::write(&config_file_path, toml::ser::to_string(&config)?)?;

            config
        }
    };

    let tmp_dir = Builder::new()
        .prefix("tmp-")
        .tempdir_in(&config.temporary_project_path)?;

    if !process::Command::new("cargo")
        .current_dir(&tmp_dir)
        .arg("init")
        .status()
        .context("Could not start cargo")?
        .success()
    {
        bail!("Cargo command failed");
    };

    let delete_file = tmp_dir.path().join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.path().join("Cargo.toml"))?;
    for (s, v) in cli.dependencies.iter() {
        match &v {
            Some(version) => writeln!(toml, "{} = \"{}\"", s, version)?,
            None => writeln!(toml, "{} = \"*\"", s)?,
        }
    }
    drop(toml);

    let mut shell_process = process::Command::new(get_shell());

    if let Some(path) = config.cargo_target_dir {
        shell_process.env("CARGO_TARGET_DIR", path);
    }

    shell_process
        .current_dir(&tmp_dir)
        .status()
        .context("Cannot start shell")?;

    if !delete_file.exists() {
        println!("Project preserved at: {}", tmp_dir.into_path().display());
    }

    Ok(())
}

fn get_shell() -> String {
    #[cfg(unix)]
    {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
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
