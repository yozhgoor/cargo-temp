use anyhow::Result;
use regex::Regex;
use std::{fmt, str::FromStr, sync::LazyLock};

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

impl FromStr for Dependency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

                    Ok(Self::Repository {
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
                        return Err("invalid URL".to_string());
                    }

                    Ok(Self::CratesIo {
                        name,
                        version,
                        features,
                        default_features,
                    })
                } else {
                    Err("invalid dependency source".to_string())
                }
            }
            None => Err("could not parse dependency".to_string()),
        }
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CratesIo {
                name,
                version,
                features,
                default_features,
            } => {
                let version = version.as_deref().unwrap_or("*");

                if *default_features && features.is_empty() {
                    write!(f, "{} = \"{}\"", name, version)
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

                    write!(f, "{}", s)
                }
            }
            Self::Repository {
                name,
                version,
                features,
                default_features,
                url,
                branch,
                rev,
            } => {
                let mut s = format!("{name} = {{ git = {url:?}");

                if let Some(branch) = branch.as_deref() {
                    s.push_str(", branch = \"");
                    s.push_str(branch);
                    s.push('"');
                }

                if let Some(rev) = rev.as_deref() {
                    s.push_str(", rev = \"");
                    s.push_str(rev);
                    s.push('"');
                }

                if let Some(version) = version.as_deref() {
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

                write!(f, "{}", s)
            }
        }
    }
}
