use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone)]
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
mod dependency_tests {
    use super::*;

    macro_rules! test_dependency {
        ($name:ident, $dep:expr, $in:expr, $out:expr) => {
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
        };
        ($name:ident, $dep:expr, $in:expr, $in_without_name:expr, $out:expr) => {
            #[test]
            fn $name() {
                let dependency = $dep;

                assert_eq!(
                    parse_dependency($in).unwrap(),
                    dependency,
                    "cannot parse dependency"
                );
                assert_eq!(
                    parse_dependency($in_without_name).unwrap(),
                    dependency,
                    "cannot parse dependency without package name"
                );
                assert_eq!(
                    format_dependency(&dependency),
                    $out,
                    "cannot format dependency"
                );
            }
        };
    }

    test_dependency!(
        dependency,
        Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: None,
            features: Vec::new(),
        },
        "anyhow",
        "anyhow = \"*\""
    );

    test_dependency!(
        dependency_with_version,
        Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("0.1".to_string()),
            features: Vec::new(),
        },
        "anyhow=0.1",
        "anyhow = \"0.1\""
    );

    test_dependency!(
        dependency_with_exact_version,
        Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("=0.1".to_string()),
            features: Vec::new(),
        },
        "anyhow==0.1",
        "anyhow = \"=0.1\""
    );

    test_dependency!(
        dependency_with_maximal_version,
        Dependency::CratesIo {
            name: "anyhow".to_string(),
            version: Some("<1.0.2".to_string()),
            features: Vec::new(),
        },
        "anyhow=<1.0.2",
        "anyhow = \"<1.0.2\""
    );

    test_dependency!(
        dependency_with_feature,
        Dependency::CratesIo {
            name: "tokio".to_string(),
            version: None,
            features: vec!["io_std".to_string()],
        },
        "tokio+io_std",
        "tokio = { version = \"*\", features = [\"io_std\"] }"
    );

    test_dependency!(
        dependency_with_features,
        Dependency::CratesIo {
            name: "tokio".to_string(),
            version: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        },
        "tokio+io_std+io_utils",
        "tokio = { version = \"*\", features = [\"io_std\", \"io_utils\"] }"
    );

    test_dependency!(
        dependency_with_version_and_feature,
        Dependency::CratesIo {
            name: "tokio".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["io_std".to_string()],
        },
        "tokio=1.0+io_std",
        "tokio = { version = \"1.0\", features = [\"io_std\"] }"
    );

    test_dependency!(
        dependency_with_version_and_features,
        Dependency::CratesIo {
            name: "tokio".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        },
        "tokio=1.0+io_std+io_utils",
        "tokio = { version = \"1.0\", features = [\"io_std\", \"io_utils\"] }"
    );

    test_dependency!(
        repository_with_http_url,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        },
        "tokio=https://github.com/tokio-rs/tokio.git",
        "https://github.com/tokio-rs/tokio.git",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\" }"
    );

    test_dependency!(
        repository_with_http_url_and_no_extension,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        },
        "tokio=https://github.com/tokio-rs/tokio",
        "https://github.com/tokio-rs/tokio",
        "tokio = { git = \"https://github.com/tokio-rs/tokio\" }"
    );

    test_dependency!(
        repository_with_ssh_url,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        },
        "serde=ssh://git@github.com/serde-rs/serde.git",
        "ssh://git@github.com/serde-rs/serde.git",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\" }"
    );

    test_dependency!(
        repository_with_ssh_url_and_no_extension,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde".to_string(),
            branch: None,
            rev: None,
            features: Vec::new(),
        },
        "serde=ssh://git@github.com/serde-rs/serde",
        "ssh://git@github.com/serde-rs/serde",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde\" }"
    );

    test_dependency!(
        repository_with_http_url_and_feature,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["io_std".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git+io_std",
        "https://github.com/tokio-rs/tokio.git+io_std",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_and_feature,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["derive".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git+derive",
        "ssh://git@github.com/serde-rs/serde.git+derive",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\"] }"
    );

    test_dependency!(
        repository_with_http_url_and_features,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git+io_std+io_utils",
        "https://github.com/tokio-rs/tokio.git+io_std+io_utils",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\", \"io_utils\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_and_features,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: None,
            features: vec!["derive".to_string(), "alloc".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git+derive+alloc",
        "ssh://git@github.com/serde-rs/serde.git+derive+alloc",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", features = [\"derive\", \"alloc\"] }"
    );

    test_dependency!(
        repository_with_http_url_and_branch,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: Vec::new(),
        },
        "tokio=https://github.com/tokio-rs/tokio.git#branch=compat",
        "https://github.com/tokio-rs/tokio.git#branch=compat",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\" }"
    );

    test_dependency!(
        repository_with_ssh_url_and_branch,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: Vec::new(),
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#branch=watt",
        "ssh://git@github.com/serde-rs/serde.git#branch=watt",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\" }"
    );

    test_dependency!(
        repository_with_http_url_branch_and_feature,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: vec!["io_std".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git#branch=compat+io_std",
        "https://github.com/tokio-rs/tokio.git#branch=compat+io_std",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_branch_and_feature,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: vec!["derive".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#branch=watt+derive",
        "ssh://git@github.com/serde-rs/serde.git#branch=watt+derive",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\"] }"
    );

    test_dependency!(
        repository_with_http_url_branch_and_features,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: Some("compat".to_string()),
            rev: None,
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils",
        "https://github.com/tokio-rs/tokio.git#branch=compat+io_std+io_utils",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\", \"io_utils\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_branch_and_features,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: Some("watt".to_string()),
            rev: None,
            features: vec!["derive".to_string(), "alloc".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc",
        "ssh://git@github.com/serde-rs/serde.git#branch=watt+derive+alloc",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", branch = \"watt\", features = [\"derive\", \"alloc\"] }"
    );

    test_dependency!(
        repository_with_http_url_and_rev,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: Vec::new(),
        },
        "tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777",
        "https://github.com/tokio-rs/tokio.git#rev=75c0777",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\" }"
    );

    test_dependency!(
        repository_with_ssh_url_and_rev,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: Vec::new(),
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a",
        "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\" }"
    );

    test_dependency!(
        repository_with_http_url_rev_and_feature,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: vec!["io_std".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std",
        "https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_rev_and_feature,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["derive".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive",
        "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\"] }"
    );

    test_dependency!(
        repository_with_http_url_rev_and_features,
        Dependency::Repository {
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio.git".to_string(),
            branch: None,
            rev: Some("75c0777".to_string()),
            features: vec!["io_std".to_string(), "io_utils".to_string()],
        },
        "tokio=https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils",
        "https://github.com/tokio-rs/tokio.git#rev=75c0777+io_std+io_utils",
        "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"75c0777\", features = [\"io_std\", \"io_utils\"] }"
    );

    test_dependency!(
        repository_with_ssh_url_rev_and_features,
        Dependency::Repository {
            name: "serde".to_string(),
            url: "ssh://git@github.com/serde-rs/serde.git".to_string(),
            branch: None,
            rev: Some("5b140361a".to_string()),
            features: vec!["derive".to_string(), "alloc".to_string()],
        },
        "serde=ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc",
        "ssh://git@github.com/serde-rs/serde.git#rev=5b140361a+derive+alloc",
        "serde = { git = \"ssh://git@github.com/serde-rs/serde.git\", rev = \"5b140361a\", features = [\"derive\", \"alloc\"] }"
    );

    #[test]
    fn could_not_parse() {
        let res = parse_dependency("http://localhost");
        assert!(res.is_err(), "{res:?}");
    }
}
