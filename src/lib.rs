use anyhow::{Context, Result};
use clap::Clap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, fs};

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
