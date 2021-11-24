use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

pub mod config;
pub mod run;

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`.
#[derive(Parser, Debug)]
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
    /// current directory.
    #[clap(long = "worktree")]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[clap(long = "git")]
    pub git: Option<String>,
}

impl Cli {
    pub fn run(&self, config: &Config) -> Result<()> {
        let tmp_dir = run::generate_tmp_project(
            self.worktree_branch.clone(),
            self.project_name.clone(),
            self.lib,
            self.git.clone(),
            config.temporary_project_dir.clone(),
            config.git_repo_depth.clone(),
            config.vcs.clone(),
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
        features: Vec<String>,
    },
    Repository {
        branch: Option<String>,
        name: String,
        rev: Option<String>,
        url: String,
        features: Vec<String>,
    },
}

fn parse_dependency(s: &str) -> Dependency {
    // This will change when `std::lazy` is released.
    // See https://github.com/rust-lang/rust/issues/74465.
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?P<name>[^+=]+)=(?P<version>((?P<url>\w+://([^:@]+(:[^@]+)?@)?[^#+]+)(#branch=(?P<branch>[^+]+)|#rev=(?P<rev>[^+]+))?)|[^+]+)?(?P<features>(\+[^+]+)*)")
            .unwrap()
    });

    if let Some(caps) = RE.captures(s) {
        let features = caps
            .name("features")
            .map(|x| {
                x.as_str()
                    .split('+')
                    .map(|x| x.to_string())
                    .skip(1)
                    .collect::<Vec<String>>()
            })
            .unwrap();
        if let Some(url) = caps.name("url") {
            Dependency::Repository {
                name: caps.name("name").unwrap().as_str().to_string(),
                url: url.as_str().to_string(),
                branch: caps.name("branch").map(|x| x.as_str().to_string()),
                rev: caps.name("rev").map(|x| x.as_str().to_string()),
                features,
            }
        } else {
            Dependency::CrateIo {
                name: caps.name("name").unwrap().as_str().to_string(),
                version: caps.name("version").map(|x| x.as_str().to_string()),
                features,
            }
        }
    } else {
        let mut it = s.split('+').map(|x| x.to_string());
        let name = it.next().unwrap();
        let features = it.collect::<Vec<_>>();

        if name.ends_with(".git") {
            let url = name.clone();
            let name = std::path::Path::new(&name)
                .file_stem()
                .expect("cannot parse repo name")
                .to_str()
                .expect("cannot convert repo name")
                .to_string();

            Dependency::Repository {
                name,
                url,
                branch: None,
                rev: None,
                features,
            }
        } else {
            Dependency::CrateIo {
                name,
                version: None,
                features,
            }
        }
    }
}

#[cfg(test)]
mod parse_and_format_dependency_tests {
    use super::*;

    #[test]
    fn simple_dependency() {
        let dependency = Dependency::CrateIo {
            name: "anyhow".to_string(),
            version: None,
            features: Vec::new(),
        };

        assert_eq!(parse_dependency("anyhow"), dependency);
        assert_eq!(run::format_dependency(&dependency), "anyhow = \"*\"");
    }

    #[test]
    fn dependency_with_version() {
        let dependency = Dependency::CrateIo {
            name: "anyhow".to_string(),
            version: Some("1.0".to_string()),
            features: Vec::new(),
        };

        assert_eq!(dependency, parse_dependency("anyhow=1.0"));
        assert_eq!(run::format_dependency(&dependency), "anyhow = \"1.0\"");
    }

    #[test]
    fn dependency_with_one_feature() {
        let dependency = Dependency::CrateIo {
            name: "serde".to_string(),
            version: None,
            features: vec!["derive".to_string()],
        };

        assert_eq!(parse_dependency("serde+derive"), dependency);
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { version = \"*\", features = [\"derive\"] }"
        );
    }

    #[test]
    fn dependency_with_two_features() {
        let dependency = Dependency::CrateIo {
            name: "serde".to_string(),
            version: None,
            features: vec!["derive".to_string(), "alloc".to_string()],
        };

        assert_eq!(parse_dependency("serde+derive+alloc"), dependency);
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { version = \"*\", features = [\"derive\", \"alloc\"] }"
        );
    }

    #[test]
    fn dependency_with_a_version_and_one_feature() {
        let dependency = Dependency::CrateIo {
            name: "serde".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["derive".to_string()],
        };

        assert_eq!(parse_dependency("serde=1.0+derive"), dependency);
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { version = \"1.0\", features = [\"derive\"] }"
        );
    }

    #[test]
    fn dependency_with_a_version_and_two_features() {
        let dependency = Dependency::CrateIo {
            name: "serde".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["derive".to_string(), "alloc".to_string()],
        };

        assert_eq!(parse_dependency("serde=1.0+derive+alloc"), dependency);
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { version = \"1.0\", features = [\"derive\", \"alloc\"] }"
        );
    }

    #[test]
    fn repository_without_package_name() {
        let dependency = Dependency::Repository {
            name: "anyhow".to_string(),
            url: "https://github.com/dtolnay/anyhow.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("https://github.com/dtolnay/anyhow.git"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "anyhow = { git = \"https://github.com/dtolnay/anyhow.git\" }",
        );
    }

    #[test]
    fn repository_without_package_name_and_a_branch() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/anyhow.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("https://github.com/tokio.rs/tokio.git#branch=compat"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\" }"
        );
    }

    #[test]
    fn repository_without_package_name_a_branch_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/anyhow.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("https://github.com/tokio.rs/tokio.git#branch=compat+io_std"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\"] }"
        );
    }

    #[test]
    fn repository_without_package_name_and_a_rev() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["alloc".to_string()],
        };

        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+alloc"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"alloc\"] }"
        );
    }

    #[test]
    fn repository_without_package_name_a_rev_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["alloc".to_string()],
        };

        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+alloc"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"alloc\"] }"
        );
    }

    #[test]
    fn repository_with_http() {
        let dependency = Dependency::Repository {
            name: "anyhow".to_string(),
            url: "https://github.com/dtolnay/anyhow.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "anyhow = { git = \"https://github.com/dtolnay/anyhow.git\" }"
        );
    }

    #[test]
    fn repository_with_http_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "https://github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["derive".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=https://github.com/serde-rs/serde.git+derive"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"https://github.com/serde-rs/serde.git\", features = [\"derive\"] }"
        );
    }

    #[test]
    fn repository_with_ssh_repository() {
        let dependency = Dependency::Repository {
            name: "anyhow".to_string(),
            url: "ssh://git@github.com/dtolnay/anyhow.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=ssh://git@github.com/dtolnay/anyhow.git"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "anyhow = { git = \"ssh://git@github.com/dtolnay/anyhow.git\" }"
        );
    }

    #[test]
    fn repository_with_ssh_repository_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["alloc".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git+alloc"),
            dependency,
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"alloc\"] }"
        );
    }

    #[test]
    fn repository_with_branch() {
        let dependency = Dependency::Repository {
            name: "anyhow".to_string(),
            url: "https://github.com/dtolnay/anyhow.git".to_string(),
            branch: Some("main".to_string()),
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git#branch=main"),
            dependency,
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "anyhow = { git = \"https://github.com/dtolnay/anyhow.git\", branch = \"main\" }"
        );
    }

    #[test]
    fn repository_with_branch_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "https://github.com/serde-rs/serde.git".to_string(),
            branch: Some("main".to_string()),
            rev: None,
            features: vec!["derive".to_string()],
        };
        assert_eq!(
            parse_dependency("serde=https://github.com/serde-rs/serde.git#branch=main+derive"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"https://github.com/serde-rs/serde.git\", branch = \"main\", features = [\"derive\"] }"
        );
    }

    #[test]
    fn repository_with_rev() {
        let dependency = Dependency::Repository {
            name: "anyhow".to_string(),
            url: "https://github.com/dtolnay/anyhow.git".to_string(),
            branch: None,
            rev: Some("7e0f77a38".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=https://github.com/dtolnay/anyhow.git#rev=7e0f77a38"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "anyhow = { git = \"https://github.com/dtolnay/anyhow.git\", rev = \"7e0f77a38\" }"
        );
    }

    #[test]
    fn repository_with_rev_and_a_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["alloc".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+alloc"),
            dependency
        );
        assert_eq!(
            run::format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"alloc\"] }"
        );
    }
}
