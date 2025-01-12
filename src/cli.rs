use crate::dependency::{parse_dependency, Dependency};

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow=1.0.13`
    #[arg(value_parser = parse_dependency)]
    pub dependencies: Vec<Dependency>,

    /// Create a library instead of a binary.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Name of the temporary crate.
    #[arg(long = "name", short = 'n')]
    pub project_name: Option<String>,

    /// Create a temporary Git working tree based on the repository in the
    /// current directory.
    #[arg(long = "worktree", short = 'w')]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[arg(long, short = 'g')]
    pub git: Option<String>,

    /// Add a `benches` to the temporary project.
    ///
    /// You can choose the name of the benchmark file name as argument.
    /// The default is `benchmark.rs`
    #[arg(long, short = 'b')]
    pub bench: Option<Option<String>>,

    /// Select the Rust's edition of the temporary project.
    ///
    /// Available options are:
    /// * 15 | 2015 => edition 2015,
    /// * 18 | 2018 => edition 2018,
    /// * 21 | 2021 => edition 2021,
    ///
    /// If the argument doesn't match any of the options, the default is the latest edition
    #[arg(long, short = 'e')]
    pub edition: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
