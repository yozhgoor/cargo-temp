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
        tag: Option<String>,
    },
    Path {
        name: String,
        path: String,
        version: Option<String>,
        features: Vec<String>,
        default_features: bool,
    },
}

impl FromStr for Dependency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?x)^
(?:
    (?P<url>
        (?:https?|ssh)://
        (?:[^:@]+(?::[^@]+)?@)?
        [^/:@?\#=><~+]+
        (?:/[^\#=><~+]+)+
        (?:\.git)?
    )
    |
    (?P<path>
        (?:
            \.\.?[/\\]
            |
            /
            |
            [A-Za-z]:[/\\]
        )
        [^\#=><~+]+
    )
    |
    (?P<name>[^=><~+]+)
)
(?:\#(?P<git_ref>[^=><~+]+))?
(?:
    (?P<op>==|>=|<=|>|<|~|=)?
    (?P<version>[0-9A-Za-z.\-]+)
)?
(?P<default_features>\+)?
(?P<features>(?:\+[^+]+)*)$
",
            )
            .expect("dependency regex must compile")
        });

        match RE.captures(s) {
            Some(caps) => {
                let op = caps.name("op").map(|x| x.as_str());
                let ver = caps.name("version").map(|x| x.as_str());
                let version = match (op, ver) {
                    (Some("=="), Some(v)) => Some(format!("={}", v)),
                    (Some("="), Some(v)) => Some(v.to_string()),
                    (Some(o), Some(v)) => Some(format!("{}{}", o, v)),
                    (None, Some(v)) => Some(v.to_string()),
                    _ => None,
                };
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

                    let (branch, rev, tag) = caps
                        .name("git_ref")
                        .map(|x| classify_git_ref(x.as_str()))
                        .unwrap_or((None, None, None));

                    Ok(Self::Repository {
                        name,
                        url,
                        version,
                        features,
                        default_features,
                        branch,
                        rev,
                        tag,
                    })
                } else if let Some(path) = caps.name("path").map(|x| x.as_str().to_string()) {
                    let comp = ['/', '\\'];
                    let name = path
                        .trim_end_matches(comp)
                        .rsplit(comp)
                        .next()
                        .ok_or_else(|| "could not infer name from path".to_string())?
                        .to_string();

                    if name.is_empty() {
                        return Err("could not infer name from path".to_string());
                    }

                    Ok(Self::Path {
                        name,
                        path,
                        version,
                        features,
                        default_features,
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
                tag,
            } => {
                if !self.is_long() {
                    write!(f, "{} = {{ git = \"{}\"", name, url)?;
                } else {
                    writeln!(f, "[dependencies.{}]", name)?;
                    if branch.is_none()
                        && rev.is_none()
                        && tag.is_none()
                        && *default_features
                        && features.is_empty()
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

                if let Some(tag) = tag.as_deref() {
                    if !self.is_long() {
                        write!(f, ", tag = \"{}\"", tag)?;
                    } else if version.is_none() && *default_features && features.is_empty() {
                        write!(f, "tag = \"{}\"", tag)?;
                    } else {
                        writeln!(f, "tag = \"{}\"", tag)?;
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
            Dependency::Path {
                name,
                path,
                version,
                features,
                default_features,
            } => {
                if !self.is_long() {
                    write!(f, "{} = {{ path = \"{}\"", name, path)?;
                } else {
                    writeln!(f, "[dependencies.{}]", name)?;
                    if version.is_none() && *default_features && features.is_empty() {
                        write!(f, "path = \"{}\"", path)?;
                    } else {
                        writeln!(f, "path = \"{}\"", path)?;
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

fn classify_git_ref(r: &str) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(stripped) = r.strip_prefix("branch:") {
        (Some(stripped.to_string()), None, None)
    } else if let Some(stripped) = r.strip_prefix("tag:") {
        (None, None, Some(stripped.to_string()))
    } else if let Some(stripped) = r.strip_prefix("rev:") {
        (None, Some(stripped.to_string()), None)
    } else if r.len() >= 7 && r.chars().all(|c| c.is_ascii_hexdigit()) {
        (None, Some(r.to_string()), None)
    } else if is_version_like(r) {
        (None, None, Some(r.to_string()))
    } else {
        (Some(r.to_string()), None, None)
    }
}

fn is_version_like(s: &str) -> bool {
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut chars = s.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut has_dot_digit = false;
    while let Some(c) = chars.next() {
        if c == '.' {
            if chars.next().is_some_and(|c| c.is_ascii_digit()) {
                has_dot_digit = true;
            } else {
                return false;
            }
        } else if !c.is_ascii_alphanumeric() && c != '-' {
            return false;
        }
    }
    has_dot_digit
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
                tag,
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

                if let Some(tag) = tag.as_deref() {
                    len += tag.len() + 6;
                }
            }
            Self::Path {
                name,
                path,
                version,
                features,
                default_features,
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

                len += path.len() + 11;
            }
        }

        len >= 100
    }
}
