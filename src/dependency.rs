use anyhow::{Result, bail};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Dependency {
    CratesIo {
        name: String,
        version: Option<String>,
        features: Vec<String>,
        default_features: bool,
    },
    Repository {
        name: String,
        version: Option<String>,
        features: Vec<String>,
        default_features: bool,
        url: String,
        branch: Option<String>,
        rev: Option<String>,
    },
}

pub fn parse_dependency(s: &str) -> Result<Dependency> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^((?P<name>[^+=/]+)=)?(?P<version>((?P<url>\w+://([^:@]+(:[^@]+)?@)?[^#+]*?(?P<url_end>/[^#+/]+)?)(#branch=(?P<branch>[^+]+)|#rev=(?P<rev>[^+]+))?)|[^+]+)?(?P<features>(\+[^+]+)*)$")
          .expect("dependency's regex must be compiled")
    });

    match RE.captures(s) {
        Some(caps) => {
            let name: Option<String> = caps.name("name").map(|x| x.as_str().to_string());
            let features = caps
                .name("features")
                .map(|x| {
                    x.as_str()
                        .split('+')
                        .map(|x| x.to_string())
                        .skip(1)
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            let default_features = caps.name("default_features").is_none();

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
                    name,
                    url,
                    version: caps.name("version").map(|x| x.as_str().to_string()),
                    features,
                    default_features,
                    branch: caps.name("branch").map(|x| x.as_str().to_string()),
                    rev: caps.name("rev").map(|x| x.as_str().to_string()),
                })
            } else if let Some(name) = name {
                Ok(Dependency::CratesIo {
                    name,
                    version: caps.name("version").map(|x| x.as_str().to_string()),
                    features,
                    default_features,
                })
            } else {
                let end = caps.name("features").unwrap().start();
                Ok(Dependency::CratesIo {
                    name: s[..end].to_string(),
                    version: None,
                    features,
                    default_features,
                })
            }
        }
        None => bail!("could not parse dependency"),
    }
}

pub fn format_dependency(dependency: &Dependency) -> String {
    match dependency {
        Dependency::CratesIo {
            name,
            version,
            features,
            default_features,
        } => {
            let version = version.as_deref().unwrap_or("*");

            if *default_features && features.is_empty() {
                format!("{name} = \"{version}\"")
            } else {
                let mut s = format!("{name} = {{ version = \"{version}\"");

                if !default_features {
                    s.push_str(", default-features = false");
                }

                if !features.is_empty() {
                    s.push_str(", features = [\"");
                    s.push_str(&features.join("\", \""));
                    s.push_str("\"]");
                }

                s.push_str(" }");

                s
            }
        }
        Dependency::Repository {
            name,
            version,
            features,
            default_features,
            url,
            branch,
            rev,
        } => {
            let mut s = format!("{name} = {{ git = {url:?}");

            if let Some(version) = version {
                s.push_str(", version = \"");
                s.push_str(version);
                s.push('"');
            }

            if let Some(branch) = branch {
                s.push_str(", branch = \"");
                s.push_str(branch);
                s.push('"');
            }

            if let Some(rev) = rev {
                s.push_str(", rev = \"");
                s.push_str(rev);
                s.push('"');
            }

            if !default_features {
                s.push_str(", default-features = false")
            }

            if !features.is_empty() {
                s.push_str(", features = [\"");
                s.push_str(&features.join("\", \""));
                s.push_str("\"]");
            }

            s.push_str(" }");

            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_module {
        (
            $mod_name:ident,
            $(($name:ident, $dep:expr, $in:expr, $out:expr)),*
            $(,)+
        ) => {
            mod $mod_name {
                use super::*;

                $(
                    #[test]
                    fn $name() {
                        let dependency = $dep;

                        assert_eq!(
                            parse_dependency($in).unwrap(),
                            dependency,
                            "cannot parse dependency"
                        );
                        assert_eq!(
                            format_dependency(&dependency),
                            $out,
                            "cannot format dependency"
                        );
                    }
                )*
            }
        };
    }

    mod dependency {
        use super::*;

        test_module!(
            simple,
            (
                dep,
                Dependency::CratesIo {
                    name: "anyhow".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                },
                "anyhow",
                "anyhow = \"*\""
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\" }"
            ),
        );

        test_module!(
            version,
            (
                dep,
                Dependency::CratesIo {
                    name: "anyhow".to_string(),
                    version: Some("0.1".to_string()),
                    features: Vec::new(),
                    default_features: true,
                },
                "anyhow=0.1",
                "anyhow = \"0.1\""
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: Some("1.0".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git=1.0",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git, version = \"1.0\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: Some("1.0".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git=1.0",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", version = \"1.0\" }"
            ),
        );

        test_module!(
            exact_version,
            (
                dep,
                Dependency::CratesIo {
                    name: "anyhow".to_string(),
                    version: Some("=0.1".to_string()),
                    features: Vec::new(),
                    default_features: true,
                },
                "anyhow==0.1",
                "anyhow = \"=0.1\""
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: Some("=1.0".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git==1.0",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git, version = \"=1.0\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: Some("=1.0".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git==1.0",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", version = \"=1.0\" }"
            ),
        );

        test_module!(
            maximal_version,
            (
                dep,
                Dependency::CratesIo {
                    name: "anyhow".to_string(),
                    version: Some("<1.0.2".to_string()),
                    features: Vec::new(),
                    default_features: true,
                },
                "anyhow=<1.0.2",
                "anyhow = \"<1.0.2\""
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: Some("<1.48".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git=<1.48",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git, version = \"<1.48\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: Some("<1.0".to_string()),
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git=<1.0",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", version = \"<1.0\" }"
            ),
        );

        test_module!(
            feature,
            (
                dep,
                Dependency::CratesIo {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string()],
                    default_features: true,
                },
                "tokio+io_std",
                "tokio = { version = \"*\", features = [\"io_std\"] }"
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git+io_std",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git+derive",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\"] }"
            ),
        );

        test_module!(
            features,
            (
                dep,
                Dependency::CratesIo {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string(), "io_utils".to_string()],
                    default_features: true,
                },
                "tokio+io_std+io_utils",
                "tokio = { version = \"*\", features = [\"io_std\", \"io_utils\"] }"
            ),
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string(), "io_utils".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git+io_std+io_utils",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\", \"io_utils\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string(), "alloc".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git+derive+alloc",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\", \"alloc\"] }"
            ),
        );

        test_module!(
            version_and_features,
            (
                dep,
                Dependency::CratesIo {
                    name: "tokio".to_string(),
                    version: Some("1.0".to_string()),
                    features: vec!["io_std".to_string(), "io_utils".to_string()],
                    default_features: true,
                },
                "tokio=1.0+io_std+io_utils",
                "tokio = { version = \"1.0\", features = [\"io_std\", \"io_utils\"] }"
            ),
        );
    }

    mod repository {
        use super::*;

        test_module!(
            no_extension,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio".to_string(),
                    branch: None,
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio",
                "tokio = { git = \"https://github.com/tokio-rs/tokio\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde".to_string(),
                    branch: None,
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde\" }"
            ),
        );

        test_module!(
            branch,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: Some("compat".to_string()),
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git#branch=compat",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: Some("watt".to_string()),
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git#branch=watt",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\" }"
            ),
        );

        test_module!(
            branch_and_feature,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: Some("compat".to_string()),
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git#branch=compat+io_std",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: Some("watt".to_string()),
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git#branch=watt+derive",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\"] }"
            ),
        );

        test_module!(
            branch_and_features,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string(), "io_utils".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: Some("compat".to_string()),
                    rev: None,
                },
                "https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\", \"io_utils\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string(), "alloc".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: Some("watt".to_string()),
                    rev: None,
                },
                "ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\", \"alloc\"] }"
            ),
        );

        test_module!(
            rev,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: Some("75c0777".to_string()),
                },
                "https://github.com/tokio-rs/tokio.git#rev=75c0777",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\" }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: Vec::new(),
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: Some("5b140361a".to_string()),
                },
                "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\" }"
            ),
        );

        test_module!(
            rev_and_feature,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: Some("75c0777".to_string()),
                },
                "https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: Some("5b140361a".to_string()),
                },
                "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\"] }"
            ),
        );

        test_module!(
            rev_and_features,
            (
                http,
                Dependency::Repository {
                    name: "tokio".to_string(),
                    version: None,
                    features: vec!["io_std".to_string(), "io_utils".to_string()],
                    default_features: true,
                    url: "https://github.com/tokio-rs/tokio.git".to_string(),
                    branch: None,
                    rev: Some("75c0777".to_string()),
                },
                "https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils",
                "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\", \"io_utils\"] }"
            ),
            (
                ssh,
                Dependency::Repository {
                    name: "serde".to_string(),
                    version: None,
                    features: vec!["derive".to_string(), "alloc".to_string()],
                    default_features: true,
                    url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
                    branch: None,
                    rev: Some("5b140361a".to_string()),
                },
                "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc",
                "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\", \"alloc\"] }"
            ),
        );
    }

    #[test]
    fn could_not_parse() {
        let res = parse_dependency("http://localhost");
        assert!(res.is_err(), "{res:?}");
    }
}
