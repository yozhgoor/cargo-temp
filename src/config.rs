use crate::subprocess::SubProcess;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
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
            temporary_project_dir
                .to_str()
                .expect("path shouldn't contains invalid unicode")
                .replace('\\', "\\\\"),
            welcome_message,
            editor
                .to_str()
                .expect("path shouldn't contains invalid unicode")
                .replace('\\', "\\\\"),
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
                let config_str = Config::template()?;

                fs::write(&config_file_path, &config_str)?;
                log::info!("Config file created at: {}", config_file_path.display());

                toml::de::from_str(&config_str)?
            }
        };

        Ok(config)
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
        let template_str = Config::template().expect("can generate template");
        let template: Config = toml::de::from_str(&template_str).expect("can deserialize template");

        let default = Config {
            welcome_message: true,
            temporary_project_dir: Some(
                Config::default_temporary_project_dir()
                    .expect("can determine default temporary project directory"),
            ),
            ..Default::default()
        };

        assert_eq!(template, default);
    }
}
