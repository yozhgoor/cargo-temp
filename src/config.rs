use crate::subprocess::SubProcess;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub welcome_message: bool,
    #[serde(default)]
    pub temporary_project_dir: Option<PathBuf>,
    #[serde(default)]
    pub preserved_project_dir: Option<PathBuf>,
    #[serde(default)]
    pub prompt: bool,
    #[serde(default)]
    pub vcs: Option<String>,
    #[serde(default)]
    pub git_repo_depth: Option<Depth>,
    #[serde(default)]
    pub cargo_target_dir: Option<PathBuf>,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub editor_args: Option<Vec<String>>,
    #[serde(default, rename = "subprocess", skip_serializing_if = "Vec::is_empty")]
    pub subprocesses: Vec<SubProcess>,
}

impl Config {
    fn template() -> Result<String> {
        let temporary_project_dir = Config::default_temporary_project_dir()?;

        let welcome_message = true;

        let editor = if cfg!(windows) {
            PathBuf::from("C:\\Program Files\\Microsoft VS Code\\Code.exe")
        } else {
            PathBuf::from("/usr/bin/code")
        };

        Ok(format!(
            include_str!("../config_template.toml"),
            temporary_project_dir.as_os_str().display(),
            welcome_message,
            editor.as_os_str().display(),
        ))
    }

    #[cfg(unix)]
    pub(crate) fn default_temporary_project_dir() -> Result<PathBuf> {
        let base = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));

        base.get_cache_home()
            .context("could not find HOME directory")
    }

    #[cfg(windows)]
    pub(crate) fn default_temporary_project_dir() -> Result<PathBuf> {
        Ok(dirs::cache_dir()
            .context("could not get cache directory")?
            .join(env!("CARGO_PKG_NAME")))
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
                fs::write(&config_file_path, Config::template()?)?;
                log::info!("Config file created at: {}", config_file_path.display());

                Config::default()
            }
        };

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            welcome_message: true,
            cargo_target_dir: None,
            preserved_project_dir: None,
            prompt: false,
            editor: None,
            editor_args: None,
            temporary_project_dir: None,
            git_repo_depth: None,
            vcs: None,
            subprocesses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum Depth {
    Active(bool),
    Level(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_empty_config() {
        let _: Config = toml::from_str("").unwrap();
    }

    #[test]
    fn from_template() {
        let template: Config = toml::de::from_str(&Config::template().unwrap()).unwrap();

        let default = Config {
            temporary_project_dir: Some(Config::default_temporary_project_dir().unwrap()),
            ..Default::default()
        };

        assert_eq!(template, default);
    }
}
