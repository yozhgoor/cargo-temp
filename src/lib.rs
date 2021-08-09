use anyhow::{ensure, Context, Result};
use clap::Clap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process};
use tempfile::{Builder, TempDir};

/// This tool allow you to create a new Rust temporary project in
/// a temporary directory.
///
/// The dependencies can be provided in arguments
/// (e.g. `cargo-temp anyhow tokio`).
/// When the shell is exited, the temporary directory is deleted
/// unless you removed the file `TO_DELETE`
#[derive(Clap, Debug)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow=1.0.13`
    #[clap(parse(from_str = parse_dependency))]
    pub dependencies: Vec<Dependency>,

    /// Create a library instead of a binary.
    #[clap(long)]
    pub lib: bool,

    /// Name of the temporary crate.
    #[clap(long = "name")]
    pub project_name: Option<String>,

    /// Create a temporary Git working tree based on the repository in the
    /// current directory
    #[clap(long = "worktree")]
    pub worktree_branch: Option<Option<String>>,
}

impl Cli {
    pub fn new() -> Cli {
        let mut args = env::args().peekable();
        let command = args.next();
        args.next_if(|x| x.as_str() == "temp");

        Cli::parse_from(command.into_iter().chain(args))
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cargo_target_dir: Option<String>,
    pub editor: Option<String>,
    pub editor_args: Option<Vec<String>>,
    pub temporary_project_dir: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        #[cfg(unix)]
        let cache_dir = {
            let cache_dir = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))
                .context("Could not find HOME directory")?;

            cache_dir.get_cache_home()
        };
        #[cfg(windows)]
        let cache_dir = dirs::cache_dir()
            .context("Could not get cache directory")?
            .join(env!("CARGO_PKG_NAME"));

        let temporary_project_dir = cache_dir
            .to_str()
            .context("Cannot convert temporary project path into str")?
            .to_string();

        Ok(Self {
            cargo_target_dir: None,
            editor: None,
            editor_args: None,
            temporary_project_dir,
        })
    }

    pub fn get_or_create() -> Result<Self> {
        #[cfg(unix)]
        let config_file_path = {
            let config_dir = xdg::BaseDirectories::with_prefix("cargo-temp")?;
            config_dir.place_config_file("config.toml")?
        };
        #[cfg(windows)]
        let config_file_path = {
            let config_dir = dirs::config_dir()
                .context("Could not get config directory")?
                .join(env!("CARGO_PKG_NAME"));
            let _ = fs::create_dir_all(&config_dir);

            config_dir.join("config.toml")
        };

        let config: Self = match fs::read(&config_file_path) {
            Ok(file) => toml::de::from_slice(&file)?,
            Err(_) => {
                let config = Self::new()?;
                fs::write(&config_file_path, toml::ser::to_string(&config)?)?;
                println!("Config file created at: {}", config_file_path.display());

                config
            }
        };

        Ok(config)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Dependency {
    CrateIo(String, Option<String>),
    Repository {
        branch: Option<String>,
        name: String,
        rev: Option<String>,
        url: String,
    },
}

fn parse_dependency(s: &str) -> Dependency {
    // This will change when `std::lazy` is released.
    // See https://github.com/rust-lang/rust/issues/74465.
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^([^=]+)=(((\w+://([^:@]+(:[^@]+)?@)?[^#]+)(#branch=(.+)|#rev=(.+))?)|.+)$")
            .unwrap()
    });

    if let Some(caps) = RE.captures(s) {
        if let Some(url) = caps.get(4) {
            Dependency::Repository {
                name: caps[1].to_string(),
                url: url.as_str().to_string(),
                branch: caps.get(8).map(|x| x.as_str().to_string()),
                rev: caps.get(9).map(|x| x.as_str().to_string()),
            }
        } else {
            Dependency::CrateIo(caps[1].to_string(), Some(caps[2].to_string()))
        }
    } else {
        Dependency::CrateIo(s.to_string(), None)
    }
}

pub fn get_shell() -> String {
    #[cfg(unix)]
    {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }

    #[cfg(windows)]
    {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string())
    }
}

pub fn run(cli: Cli, config: &Config) -> Result<()> {
    let tmp_dir = generate_tmp_project(
        cli.worktree_branch.clone(),
        cli.project_name,
        cli.lib,
        config.temporary_project_dir.clone(),
    )?;

    add_dependencies_to_toml(&tmp_dir, cli.dependencies)?;

    let delete_file = generate_delete_file(&tmp_dir)?;

    start_shell(&config, &tmp_dir)?;

    exit_shell(delete_file, tmp_dir, cli.worktree_branch)?;

    Ok(())
}

fn generate_tmp_project(
    worktree_branch: Option<Option<String>>,
    project_name: Option<String>,
    lib: bool,
    temporary_project_dir: String,
) -> Result<TempDir> {
    let tmp_dir = {
        let mut builder = Builder::new();

        if worktree_branch.is_some() {
            builder.prefix("wk-");
        } else {
            builder.prefix("tmp-");
        }

        builder.tempdir_in(temporary_project_dir)?
    };

    let project_name = project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
    });

    if let Some(maybe_branch) = worktree_branch.as_ref() {
        let mut command = process::Command::new("git");
        command.args(["worktree", "add"]);

        match maybe_branch {
            Some(branch) => command.arg(tmp_dir.path()).arg(branch),
            None => command.arg("-d").arg(tmp_dir.path()),
        };

        ensure!(
            command.status().context("Could not start git")?.success(),
            "Cannot create working tree"
        );
    } else {
        let mut command = process::Command::new("cargo");
        command
            .current_dir(&tmp_dir)
            .args(["init", "--name", project_name.as_str()]);

        if lib {
            command.arg("--lib");
        }

        ensure!(
            command.status().context("Could not start cargo")?.success(),
            "Cargo command failed"
        );
    }

    Ok(tmp_dir)
}

fn add_dependencies_to_toml(tmp_dir: &TempDir, dependencies: Vec<Dependency>) -> Result<()> {
    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.path().join("Cargo.toml"))?;
    for dependency in dependencies.iter() {
        match dependency {
            Dependency::CrateIo(s, v) => match &v {
                Some(version) => writeln!(toml, "{} = \"{}\"", s, version)?,
                None => writeln!(toml, "{} = \"*\"", s)?,
            },
            Dependency::Repository {
                name,
                url,
                branch,
                rev,
            } => {
                write!(toml, "{name} = {{ git = {url:?}", name = name, url = url)?;
                if let Some(branch) = branch {
                    write!(toml, ", branch = {:?}", branch)?;
                }
                if let Some(rev) = rev {
                    write!(toml, ", rev = {:?}", rev)?;
                }
                writeln!(toml, " }}")?;
            }
        }
    }

    Ok(())
}

fn generate_delete_file(tmp_dir: &TempDir) -> Result<PathBuf> {
    let delete_file = tmp_dir.path().join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    Ok(delete_file)
}

fn start_shell(config: &Config, tmp_dir: &TempDir) -> Result<()> {
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
        if let Some(path) = &config.cargo_target_dir {
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

    Ok(())
}

pub fn exit_shell(
    delete_file: PathBuf,
    tmp_dir: TempDir,
    worktree_branch: Option<Option<String>>,
) -> Result<()> {
    if !delete_file.exists() {
        println!(
            "Project directory preserved at: {}",
            tmp_dir.into_path().display()
        );
    } else if worktree_branch.is_some() {
        let mut command = process::Command::new("git");
        command
            .args(["worktree", "remove"])
            .arg(&tmp_dir.path())
            .arg("--force");
        ensure!(
            command.status().context("Could not start git")?.success(),
            "Cannot remove working tree"
        );
    }

    Ok(())
}

#[cfg(test)]
mod parse_dependency_tests {
    use super::*;

    #[test]
    fn simple_dependency() {
        assert_eq!(
            parse_dependency("anyhow"),
            Dependency::CrateIo("anyhow".to_string(), None)
        );
    }

    #[test]
    fn dependency_with_version() {
        assert_eq!(
            parse_dependency("anyhow=1.0"),
            Dependency::CrateIo("anyhow".to_string(), Some("1.0".to_string()))
        )
    }

    #[test]
    fn repository_with_http() {
        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git"),
            Dependency::Repository {
                name: "anyhow".to_string(),
                url: "https://github.com/dtolnay/anyhow.git".to_string(),
                branch: None,
                rev: None,
            }
        )
    }

    #[test]
    fn repository_with_ssh_repository() {
        assert_eq!(
            parse_dependency("anyhow=ssh://git@github.com/dtolnay/anyhow.git"),
            Dependency::Repository {
                name: "anyhow".to_string(),
                url: "ssh://git@github.com/dtolnay/anyhow.git".to_string(),
                branch: None,
                rev: None,
            }
        )
    }

    #[test]
    fn repository_with_branch() {
        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git#branch=main"),
            Dependency::Repository {
                name: "anyhow".to_string(),
                url: "https://github.com/dtolnay/anyhow.git".to_string(),
                branch: Some("main".to_string()),
                rev: None,
            }
        )
    }

    #[test]
    fn with_rev() {
        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git#rev=7e0f77a38"),
            Dependency::Repository {
                name: "anyhow".to_string(),
                url: "https://github.com/dtolnay/anyhow.git".to_string(),
                branch: None,
                rev: Some("7e0f77a38".to_string()),
            }
        )
    }
}
