use anyhow::{bail, Context, Result};
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env, fs, process};
use tempfile::Builder;

/// This tool allow you to create a new Rust temporary project in
/// a temporary directory.
///
/// The dependencies can be provided in arguments
/// (e.g. `cargo-temp anyhow tokio`).
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
    /// Name of the temporary crate.
    #[clap(long = "name")]
    project_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    temporary_project_dir: String,
    cargo_target_dir: Option<String>,
    editor: Option<String>,
    editor_args: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .expect("Could not get cache directory")
            .join(env!("CARGO_PKG_NAME"));
        Config {
            temporary_project_dir: cache_dir
                .to_str()
                .expect("Could not convert cache path into str")
                .to_string(),
            cargo_target_dir: None,
            editor: None,
            editor_args: None,
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");
    let cli = Cli::parse_from(command.into_iter().chain(args));

    let config_dir = dirs::config_dir()
        .context("Could not get config directory")?
        .join(env!("CARGO_PKG_NAME"));
    let _ = fs::create_dir_all(&config_dir);

    let config_file_path = config_dir.join("config.toml");

    let config: Config = match fs::read(&config_file_path) {
        Ok(file) => toml::de::from_slice(&file)?,
        Err(_) => {
            let config = Config::default();
            fs::write(&config_file_path, toml::ser::to_string(&config)?)?;

            config
        }
    };

    let _ = fs::create_dir(&config.temporary_project_dir);

    let tmp_dir = Builder::new()
        .prefix("tmp-")
        .tempdir_in(&config.temporary_project_dir)?;
    let project_name = cli.project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .expect("Cannot retrieve temporary directory name")
            .to_str()
            .expect("Cannot convert temporary directory name into str")
            .to_lowercase()
    });

    if !process::Command::new("cargo")
        .current_dir(&tmp_dir)
        .args(&["init", "--name", project_name.as_str()])
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

    let mut shell_process = match config.editor {
        None => process::Command::new(get_shell()),
        Some(ref editor) => {
            let mut ide_process = process::Command::new(editor);
            ide_process
                .args(config.editor_args.iter().flatten())
                .arg(tmp_dir.path());
            ide_process
        }
    };

    if env::var("CARGO_TARGET_DIR").is_err() {
        if let Some(path) = config.cargo_target_dir {
            shell_process.env("CARGO_TARGET_DIR", path);
        }
    }

    shell_process
        .current_dir(&tmp_dir)
        .status()
        .context("Cannot start shell")?;

    #[cfg(windows)]
    if config.editor.is_some() {
        unsafe {
           cargo_temp_bindings::Windows::Win32::SystemServices::FreeConsole();
        }
    }

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

    #[cfg(windows)]
    {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string())
    }
}

fn parse_dependency(s: &str) -> (String, Option<String>) {
    let mut it = s.splitn(2, '=').map(|x| x.to_string());
    (it.next().unwrap(), it.next())
}
