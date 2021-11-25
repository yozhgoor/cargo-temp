use crate::config::Config;
use anyhow::{bail, Result};
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

        let subprocesses = run::start_subprocesses(config, tmp_dir.path());

        let res = run::start_shell(config, tmp_dir.path());

        run::clean_up(
            delete_file,
            tmp_dir,
            self.worktree_branch.clone(),
            subprocesses,
        )?;

        match res {
            Ok(_exit_status) => Ok(()),
            Err(err) => bail!("problem within the shell process: {}", err),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Dependency {
    CratesIo {
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
        Regex::new(r"^((?P<name>[^+=]+)=)?(?P<version>((?P<url>\w+://([^:@]+(:[^@]+)?@)?[^#+]+)(#branch=(?P<branch>[^+]+)|#rev=(?P<rev>[^+]+))?)|[^+]+)?(?P<features>(\+[^+]+)*)")
            .unwrap()
    });

    match RE.captures(s) {
        Some(caps) => {
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

            if let Some(name) = caps.name("name") {
                if let Some(url) = caps.name("url") {
                    Dependency::Repository {
                        name: name.as_str().to_string(),
                        url: url.as_str().to_string(),
                        branch: caps.name("branch").map(|x| x.as_str().to_string()),
                        rev: caps.name("rev").map(|x| x.as_str().to_string()),
                        features,
                    }
                } else {
                    let mut it = name.as_str().split('#');
                    let url = it.next().unwrap();
                    let name = std::path::Path::new(&url)
                        .file_stem()
                        .expect("cannot get repo name")
                        .to_str()
                        .expect("cannot convert repo name")
                        .to_string();
                    let res = match it.next() {
                        Some("branch") => Some(true),
                        Some("rev") => Some(false),
                        _ => None,
                    };

                    if let Some(res) = res {
                        if res {
                            Dependency::Repository {
                                name,
                                url: url.to_string(),
                                branch: caps.name("version").map(|x| x.as_str().to_string()),
                                rev: None,
                                features,
                            }
                        } else {
                            Dependency::Repository {
                                name,
                                url: url.to_string(),
                                branch: None,
                                rev: caps.name("version").map(|x| x.as_str().to_string()),
                                features,
                            }
                        }
                    } else {
                        Dependency::CratesIo {
                            name,
                            version: caps.name("version").map(|x| x.as_str().to_string()),
                            features,
                        }
                    }
                }
            } else if let Some(url) = caps.name("url") {
                let url = url.as_str();
                let name = std::path::Path::new(&url)
                    .file_stem()
                    .expect("cannot get repo name")
                    .to_str()
                    .expect("cannot convert repo name");

                Dependency::Repository {
                    name: name.to_string(),
                    url: url.to_string(),
                    branch: caps.name("branch").map(|x| x.as_str().to_string()),
                    rev: caps.name("rev").map(|x| x.as_str().to_string()),
                    features,
                }
            } else {
                let mut it = s.split('+').map(|x| x.to_string());
                let name = it.next().unwrap();
                let features = it.collect::<Vec<_>>();

                Dependency::CratesIo {
                    name,
                    version: None,
                    features,
                }
            }
        }
        None => {
            let mut it = s.split('+').map(|x| x.to_string());
            let name = it.next().unwrap();
            let features = it.collect::<Vec<_>>();

            Dependency::CratesIo {
                name,
                version: None,
                features,
            }
        }
    }
}

pub fn format_dependency(dependency: &Dependency) -> String {
    match dependency {
        Dependency::CratesIo {
            name: n,
            version: v,
            features: f,
        } => {
            if let Some(version) = v {
                if !f.is_empty() {
                    format!(
                        "{} = {{ version = \"{}\", features = {:?} }}",
                        n, version, f
                    )
                } else {
                    format!("{} = \"{}\"", n, version)
                }
            } else if !f.is_empty() {
                format!("{} = {{ version = \"*\", features = {:?} }}", n, f)
            } else {
                format!("{} = \"*\"", n)
            }
        }
        Dependency::Repository {
            name,
            url,
            branch,
            rev,
            features,
        } => {
            let mut string = format!("{} = {{ git = {:?}", name, url);

            if let Some(branch) = branch {
                string.push_str(format!(", branch = {:?}", branch).as_str())
            }
            if let Some(rev) = rev {
                string.push_str(format!(", rev = {:?}", rev).as_str())
            }
            if !features.is_empty() {
                string.push_str(format!(", features = {:?}", features).as_str())
            }

            string.push_str(" }");

            string
        }
    }
}

#[cfg(test)]
mod parse_and_format_dependency_tests {
    use super::*;

    #[test]
    fn dependency() {
        let dependency = Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "anyhow = \"*\"",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_version() {
        let dependency = Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("0.1".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=0.1"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "anyhow = \"0.1\"",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_feature() {
        let dependency = Dependency::CratesIo {
            name: "tokio".to_string(),
            version: None,
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio+io_std"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { version = \"*\", features = [\"io_std\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_features() {
        let dependency = Dependency::CratesIo {
            name: "tokio".to_string(),
            version: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio+io_std+io_utils"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { version = \"*\", features = [\"io_std\", \"io_utils\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_version_and_feature() {
        let dependency = Dependency::CratesIo {
            name: "tokio".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=1.0+io_std"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { version = \"1.0\", features = [\"io_std\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_version_and_features() {
        let dependency = Dependency::CratesIo {
            name: "tokio".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=1.0+io_std+io_utils"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { version = \"1.0\", features = [\"io_std\", \"io_utils\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_and_feature() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git+io_std"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git+io_std"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_and_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["derive".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git+derive"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git+derive"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_and_features() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git+io_std+io_utils"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git+io_std+io_utils"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\", \"io_utils\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_and_features() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["derive".to_string(), "alloc".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git+derive+alloc"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git+derive+alloc"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\", \"alloc\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_and_branch() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#branch=compat"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_and_branch() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#branch=watt"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_branch_and_feature() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#branch=compat+io_std"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat+io_std"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_branch_and_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: vec!["derive".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#branch=watt+derive"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt+derive"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_branch_and_features() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        };

        assert_eq!(
            parse_dependency(
                "tokio=https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils"
            ),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\", \"io_utils\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_branch_and_features() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: vec!["derive".to_string(), "alloc".to_string()],
        };

        assert_eq!(
            parse_dependency(
                "serde=ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc"
            ),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\", \"alloc\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_and_rev() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_and_rev() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\" }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_rev_and_feature() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: vec!["io_std".to_string()],
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_rev_and_feature() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["derive".to_string()],
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive"),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_http_url_rev_and_features() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        };

        assert_eq!(
            parse_dependency(
                "tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils"
            ),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\", \"io_utils\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn repository_with_ssh_url_rev_and_features() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["derive".to_string(), "alloc".to_string()],
        };

        assert_eq!(
            parse_dependency(
                "serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc"
            ),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc"),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\", \"alloc\"] }",
            "cannot format dependency"
        );
    }
}
