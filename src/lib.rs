use clap::Clap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;

pub mod config;
pub mod run;

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
