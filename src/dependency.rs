use anyhow::{Result, bail};
use regex::Regex;
use std::sync::LazyLock;

#[cfg(test)]
mod tests;

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
        Regex::new(r"^(?:(?P<url>(?:http|https|ssh)://(?:[^:@]+(?::[^@]+)?@)?[^/:@?#+=!]+(?:/[^#+=!]+)+(?:\.git)?)|(?P<name>[^=+#!]+))(?:#(?P<git_ref>[^+=!]+))?(?:=(?P<version>(?:>=|<=|>|<|=|~)?[0-9A-Za-z\.\-]+))?(?P<default_features>!default[^+#!]?)?(?P<features>(?:\+[^+#!]+)*)$")
        .expect("dependency's regex must be compiled")
    });

    match RE.captures(s) {
        Some(caps) => {
            let version = caps.name("version").map(|x| x.as_str().to_string());
            let features = caps.name("features").map_or(vec![], |x| {
                x.as_str()
                    .split('+')
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect()
            });
            let default_features = caps.name("default_features").is_none();

            if let Some(url) = caps.name("url").map(|x| x.as_str().to_string()) {
                let name = url
                    .rsplit('/')
                    .next()
                    .unwrap_or("unknown")
                    .trim_end_matches(".git")
                    .to_string();

                let git_ref = caps.name("git_ref").map(|x| x.as_str());
                let (branch, rev) = if let Some(r) = git_ref {
                    if r.len() >= 7 && r.chars().all(|c| c.is_ascii_hexdigit()) {
                        (None, Some(r.to_string()))
                    } else {
                        (Some(r.to_string()), None)
                    }
                } else {
                    (None, None)
                };

                Ok(Dependency::Repository {
                    name,
                    url,
                    version,
                    features,
                    default_features,
                    branch,
                    rev,
                })
            } else if let Some(name) = caps.name("name").map(|x| x.as_str().to_string()) {
                if name.contains("://") {
                    bail!("invalid URL");
                }

                Ok(Dependency::CratesIo {
                    name,
                    version,
                    features,
                    default_features,
                })
            } else {
                bail!("invalid dependency source");
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

            if let Some(version) = version {
                s.push_str(", version = \"");
                s.push_str(version);
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
