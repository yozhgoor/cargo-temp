use anyhow::{bail, Context, Result};
use clap::Clap;
use regex::Regex;
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
    dependencies: Vec<Dependency>,

    /// Name of the temporary crate.
    #[clap(long = "name")]
    project_name: Option<String>,

    /// Create a library instead of a binary.
    #[clap(long)]
    lib: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum Dependency {
    CrateIo(String, Option<String>),
    Repository {
        name: String,
        url: String,
        branch: Option<String>,
        rev: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
struct Config {
    temporary_project_dir: String,
    cargo_target_dir: Option<String>,
    editor: Option<String>,
    editor_args: Option<Vec<String>>,
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

        Ok(Self {
            temporary_project_dir: cache_dir
                .to_str()
                .context("Could not convert cache path into str")?
                .to_string(),
            cargo_target_dir: None,
            editor: None,
            editor_args: None,
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

fn main() -> Result<()> {
    // Parse the command line input.
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "temp");
    let cli = Cli::parse_from(command.into_iter().chain(args));

    // Read configuration from disk or generate a default one.
    let config = Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);
    let tmp_dir = Builder::new()
        .prefix("tmp-")
        .tempdir_in(&config.temporary_project_dir)?;
    let project_name = cli.project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
    });

    // Generate the temporary project
    let mut command = process::Command::new("cargo");
    command
        .current_dir(&tmp_dir)
        .args(&["init", "--name", project_name.as_str()]);
    if cli.lib {
        command.arg("--lib");
    }
    if !command.status().context("Could not start cargo")?.success() {
        bail!("Cargo command failed");
    };

    // Generate the `TO_DELETE` file
    let delete_file = tmp_dir.path().join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    // Add dependencies to Cargo.toml from arguments given by the user
    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.path().join("Cargo.toml"))?;
    for dependency in cli.dependencies.iter() {
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
    drop(toml);

    // Prepare a new shell or an editor if its set in the config file
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

fn parse_dependency(s: &str) -> Dependency {
    let regex =
        Regex::new(r"^([^=]+)=(((\w+://([^:@]+(:[^@]+)?@)?[^#]+)(#branch=(.+)|#rev=(.+))?)|.+)$")
            .unwrap();

    if let Some(caps) = regex.captures(s) {
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
    fn dependency_with_minor_version() {
        assert_eq!(
            parse_dependency("anyhow==1.1.0"),
            Dependency::CrateIo("anyhow".to_string(), Some("=1.1.0".to_string()))
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
