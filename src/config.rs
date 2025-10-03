use crate::subprocess::SubProcess;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub welcome_message: bool,
    #[serde(default)]
    pub cargo_target_dir: Option<PathBuf>,
    #[serde(default)]
    pub preserved_project_dir: Option<PathBuf>,
    #[serde(default)]
    pub prompt: bool,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub editor_args: Option<Vec<String>>,
    pub temporary_project_dir: PathBuf,
    #[serde(default)]
    pub git_repo_depth: Option<Depth>,
    #[serde(default)]
    pub vcs: Option<String>,
    #[serde(default, rename = "subprocess", skip_serializing_if = "Vec::is_empty")]
    pub subprocesses: Vec<SubProcess>,
}

impl Config {
    fn new() -> Result<Self> {
        #[cfg(unix)]
        let temporary_project_dir = {
            let base = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));

            base.get_cache_home()
                .context("could not find HOME directory")?
        };
        #[cfg(windows)]
        let temporary_project_dir = dirs::cache_dir()
            .context("could not get cache directory")?
            .join(env!("CARGO_PKG_NAME"));

        Ok(Self {
            welcome_message: true,
            cargo_target_dir: None,
            preserved_project_dir: None,
            prompt: false,
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
            let config_dir = xdg::BaseDirectories::with_prefix("cargo-temp");
            config_dir.place_config_file("config.toml")?
        };
        #[cfg(windows)]
        let config_file_path = {
            let config_dir = dirs::config_dir()
                .context("could not get config directory")?
                .join(env!("CARGO_PKG_NAME"));
            let _ = fs::create_dir_all(&config_dir);

            config_dir.join("config.toml")
        };

        let config: Self = match fs::read_to_string(&config_file_path) {
            Ok(file) => toml::de::from_str(&file)?,
            Err(_) => {
                let config = Self::new()?;
                fs::write(&config_file_path, toml::ser::to_string(&config)?)?;
                log::info!("Config file created at: {}", config_file_path.display());

                config
            }
        };

        Ok(config)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Depth {
    Active(bool),
    Level(u8),
}
