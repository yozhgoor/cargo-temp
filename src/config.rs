use crate::run;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, process};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cargo_target_dir: Option<String>,
    pub editor: Option<String>,
    pub editor_args: Option<Vec<String>>,
    pub temporary_project_dir: PathBuf,
    pub git_repo_depth: Option<Depth>,
    pub vcs: Option<String>,
    #[serde(rename = "subprocess")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subprocesses: Vec<SubProcess>,
}

impl Config {
    fn new() -> Result<Self> {
        #[cfg(unix)]
        let temporary_project_dir = {
            let cache_dir = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))
                .context("Could not find HOME directory")?;

            cache_dir.get_cache_home()
        };
        #[cfg(windows)]
        let temporary_project_dir = dirs::cache_dir()
            .context("Could not get cache directory")?
            .join(env!("CARGO_PKG_NAME"));

        Ok(Self {
            cargo_target_dir: None,
            editor: None,
            editor_args: None,
            git_repo_depth: None,
            temporary_project_dir,
            vcs: None,
            subprocesses: Default::default(),
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

#[derive(Serialize, Deserialize)]
pub struct SubProcess {
    pub command: String,
    pub foreground: bool,
    #[serde(default)]
    pub keep_on_exit: bool,
    pub working_dir: Option<PathBuf>,
    pub stdout: Option<bool>,
    pub stderr: Option<bool>,
}

impl SubProcess {
    pub fn spawn(&self, tmp_dir: &Path) -> Option<process::Child> {
        let mut process = process::Command::new(run::get_shell());

        process.current_dir(self.working_dir.as_deref().unwrap_or(tmp_dir));

        #[cfg(unix)]
        process.arg("-c");
        #[cfg(windows)]
        process.arg("/c");

        process.arg(&self.command).stdin(process::Stdio::null());

        if !self.foreground {
            if !self.stdout.unwrap_or(false) {
                process.stdout(process::Stdio::null());
            }

            if !self.stderr.unwrap_or(false) {
                process.stderr(process::Stdio::null());
            }

            match process.spawn().ok() {
                Some(child) => Some(child).filter(|_| !self.keep_on_exit),
                None => {
                    log::error!("An error occurred within the subprocess");
                    None
                }
            }
        } else {
            if !self.stdout.unwrap_or(true) {
                process.stdout(process::Stdio::null());
            }

            if !self.stderr.unwrap_or(true) {
                process.stderr(process::Stdio::null());
            }

            match process.status() {
                Ok(_) => None,
                Err(err) => {
                    log::error!(
                        "An error occurred within the foreground subprocess: {}",
                        err
                    );
                    None
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Depth {
    Active(bool),
    Level(u8),
}
