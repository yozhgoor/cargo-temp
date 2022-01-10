use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;

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

pub fn parse_dependency(s: &str) -> Result<Dependency> {
    // This will change when `std::lazy` is released.
    // See https://github.com/rust-lang/rust/issues/74465.
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^((?P<name>[^+=/]+)=)?(?P<version>((?P<url>\w+://([^:@]+(:[^@]+)?@)?[^#+]*?(?P<url_end>/[^#+/]+)?)(#branch=(?P<branch>[^+]+)|#rev=(?P<rev>[^+]+))?)|[^+]+)?(?P<features>(\+[^+]+)*)$")
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
            let name: Option<String> = caps.name("name").map(|x| x.as_str().to_string());

            if let Some(url) = caps.name("url").map(|x| x.as_str().to_string()) {
                let name = if let Some(name) = name {
                    name
                } else if let Some(url_end) = caps.name("url_end").map(|x| x.as_str()) {
                    url_end
                        .trim_start_matches('/')
                        .trim_end_matches(".git")
                        .to_string()
                } else {
                    bail!("could not guess name of crate in URL");
                };

                Ok(Dependency::Repository {
                    branch: caps.name("branch").map(|x| x.as_str().to_string()),
                    rev: caps.name("rev").map(|x| x.as_str().to_string()),
                    features,
                    url,
                    name,
                })
            } else if let Some(name) = name {
                Ok(Dependency::CratesIo {
                    name,
                    version: caps.name("version").map(|x| x.as_str().to_string()),
                    features,
                })
            } else {
                let end = caps.name("features").unwrap().start();
                Ok(Dependency::CratesIo {
                    name: s[..end].to_string(),
                    version: None,
                    features,
                })
            }
        }
        None => bail!("could not parse dependency"),
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
            parse_dependency("anyhow").unwrap(),
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
            parse_dependency("anyhow=0.1").unwrap(),
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
    fn dependency_with_exact_version() {
        let dependency = Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("=0.1".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow==0.1").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "anyhow = \"=0.1\"",
            "cannot format dependency"
        );
    }

    #[test]
    fn dependency_with_maximal_version() {
        let dependency = Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("<1.0.2".to_string()),
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("anyhow=<1.0.2").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            format_dependency(&dependency),
            "anyhow = \"<1.0.2\"",
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
            parse_dependency("tokio+io_std").unwrap(),
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
            parse_dependency("tokio+io_std+io_utils").unwrap(),
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
            parse_dependency("tokio=1.0+io_std").unwrap(),
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
            parse_dependency("tokio=1.0+io_std+io_utils").unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git").unwrap(),
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
    fn repository_with_http_url_and_no_extension() {
        let dependency = Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("tokio=https://github.com/tokio-rs/tokio").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio").unwrap(),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "tokio = { git = \"https://github.com/tokio-rs/tokio\" }",
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git").unwrap(),
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
    fn repository_with_ssh_url_and_no_extension() {
        let dependency = Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        };

        assert_eq!(
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde").unwrap(),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde\" }",
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git+io_std").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git+io_std").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git+derive").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git+derive").unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git+io_std+io_utils")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git+io_std+io_utils").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git+derive+alloc").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git+derive+alloc").unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#branch=compat").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#branch=watt").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt").unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#branch=compat+io_std")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat+io_std").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#branch=watt+derive")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt+derive").unwrap(),
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
            )
            .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils")
                .unwrap(),
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
            )
            .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc")
                .unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777").unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a").unwrap(),
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
            parse_dependency("tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std").unwrap(),
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
            parse_dependency("serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive")
                .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive")
                .unwrap(),
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
            )
            .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils")
                .unwrap(),
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
            )
            .unwrap(),
            dependency,
            "cannot parse dependency"
        );
        assert_eq!(
            parse_dependency("ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc")
                .unwrap(),
            dependency,
            "cannot parse dependency without package name"
        );
        assert_eq!(
            format_dependency(&dependency),
            "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\", \"alloc\"] }",
            "cannot format dependency"
        );
    }

    #[test]
    fn could_not_parse() {
        let res = parse_dependency("http://localhost");
        assert!(res.is_err(), "{:?}", res);
    }
}
