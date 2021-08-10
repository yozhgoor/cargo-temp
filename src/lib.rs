use crate::config::Config;
use anyhow::Result;
use clap::Clap;
use once_cell::sync::Lazy;
use regex::Regex;

pub mod config;
pub mod run;

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`
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
    pub fn run(&self, config: &Config) -> Result<()> {
        let tmp_dir = run::generate_tmp_project(
            self.worktree_branch.clone(),
            self.project_name.clone(),
            self.lib,
            config.temporary_project_dir.clone(),
        )?;

        run::add_dependencies_to_project(tmp_dir.path(), &self.dependencies)?;

        let delete_file = run::generate_delete_file(tmp_dir.path())?;

        run::start_shell(config, tmp_dir.path())?;

        run::clean_up(delete_file, tmp_dir, self.worktree_branch.clone())?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Dependency {
    CrateIo {
        name: String,
        version: Option<String>,
    },
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
            Dependency::CrateIo {
                name: caps[1].to_string(),
                version: Some(caps[2].to_string()),
            }
        }
    } else {
        Dependency::CrateIo {
            name: s.to_string(),
            version: None,
        }
    }
}

#[cfg(test)]
mod parse_dependency_tests {
    use super::*;

    #[test]
    fn simple_dependency() {
        assert_eq!(
            parse_dependency("anyhow"),
            Dependency::CrateIo {
                name: "anyhow".to_string(),
                version: None,
            }
        );
    }

    #[test]
    fn dependency_with_version() {
        assert_eq!(
            parse_dependency("anyhow=1.0"),
            Dependency::CrateIo {
                name: "anyhow".to_string(),
                version: Some("1.0".to_string()),
            }
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
