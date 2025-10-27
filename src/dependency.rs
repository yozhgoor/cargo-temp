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
                    return write!(f, "{} = \"{}\"", name, version);
                } else if !self.is_long() {
                    write!(f, "{} = {{ version = \"{}\"", name, version)?;
                } else {
                    writeln!(f, "[dependencies.{}]", name)?;

                    if *default_features && !features.is_empty() {
                        write!(f, "version = \"{}\"", version)?;
                    } else {
                        writeln!(f, "version = \"{}\"", version)?;
                    }
                }

                if !default_features {
                    if !self.is_long() {
                        write!(f, ", default-features = false")?;
                    } else if features.is_empty() {
                        write!(f, "default-features = false")?;
                    } else {
                        writeln!(f, "default-features = false")?;
                    }
                }

                if !features.is_empty() {
                    if !self.is_long() {
                        write!(f, ", features = [\"{}\"]", features.join("\", \""))?;
                    } else {
                        write!(f, "features = [\"{}\"]", features.join("\", \""))?;
                    }
                }

                if !self.is_long() {
                    write!(f, " }}")?;
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
                if !self.is_long() {
                    write!(f, "{} = {{ git = \"{}\"", name, url)?;
                } else {
                    writeln!(f, "[dependencies.{}]", name)?;
                    if branch.is_none() && rev.is_none() && *default_features && features.is_empty()
                    {
                        write!(f, "git = \"{}\"", url)?;
                    } else {
                        writeln!(f, "git = \"{}\"", url)?;
                    }
                }

                if let Some(branch) = branch.as_deref() {
                    if !self.is_long() {
                        write!(f, ", branch = \"{}\"", branch)?;
                    } else if version.is_none() && *default_features && features.is_empty() {
                        write!(f, "branch = \"{}\"", branch)?;
                    } else {
                        writeln!(f, "branch = \"{}\"", branch)?;
                    }
                }

                if let Some(rev) = rev.as_deref() {
                    if !self.is_long() {
                        write!(f, ", rev = \"{}\"", rev)?;
                    } else if version.is_none() && *default_features && features.is_empty() {
                        write!(f, "rev = \"{}\"", rev)?;
                    } else {
                        writeln!(f, "rev = \"{}\"", rev)?;
                    }
                }

                if let Some(version) = version.as_deref() {
                    if !self.is_long() {
                        write!(f, ", version = \"{}\"", version)?;
                    } else if *default_features && features.is_empty() {
                        write!(f, "version = \"{}\"", version)?;
                    } else {
                        writeln!(f, "version = \"{}\"", version)?;
                    }
                }

                if !default_features {
                    if !self.is_long() {
                        write!(f, ", default-features = false")?;
                    } else if features.is_empty() {
                        write!(f, "default-features = false")?;
                    } else {
                        writeln!(f, "default-features = false")?;
                    }
                }

                if !features.is_empty() {
                    if !self.is_long() {
                        write!(f, ", features = [\"{}\"]", features.join("\", \""))?;
                    } else {
                        write!(f, "features = [\"{}\"]", features.join("\", \""))?;
                    }
                }

                if !self.is_long() {
                    write!(f, " }}")?;
                }
            }
        }

        Ok(())
    }
}

impl Dependency {
    pub fn is_long(&self) -> bool {
        let mut len = 0;

        match self {
            Self::CratesIo {
                name,
                version,
                features,
                default_features,
            } => {
                len += name.len() + 5;

                if let Some(version) = version.as_deref() {
                    len += version.len() + 10;
                }

                len += 14;
                for feature in features {
                    len += feature.len() + 6;
                }

                if !default_features {
                    len += 24;
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
                len += name.len() + 5;

                if let Some(version) = version.as_deref() {
                    len += version.len() + 9;
                }

                len += 13;
                for feature in features {
                    len += feature.len() + 6;
                }

                if !default_features {
                    len += 24;
                }

                len += url.len() + 10;

                if let Some(branch) = branch.as_deref() {
                    len += branch.len() + 10;
                }

                if let Some(rev) = rev.as_deref() {
                    len += rev.len() + 6;
                }
            }
        }

        len >= 100
    }
}
