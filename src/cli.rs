use crate::dependency::{Dependency, parse_dependency};

/// cargo-temp: Create disposable Rust projects with pre-installed dependencies.
///
/// This tool creates a new Rust project in a temporary directory, automatically installs the
/// dependencies you specify and deletes the project on exit, unless you choose to preserve it.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// Default version: `*` (latest).
    ///
    /// Specify versions with `=` (e.g. `anyhow=1.0`).
    ///
    /// Specify features using `+` (e.g. `serde+derive`).
    ///
    /// Specify branch using `#` (e.g. `https://github.com/rust-random/rand.git#branch=master`).
    #[arg(value_parser = parse_dependency)]
    pub dependencies: Vec<Dependency>,

    /// Create a library project instead of a binary.
    ///
    /// This generate a `lib.rs` file instead of `main.rs`.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Name of the temporary crate.
    ///
    /// This name is used as the directory suffix. If you preserve the project, the directory is
    /// renamed to this name.
    #[arg(long = "name", short = 'n')]
    pub project_name: Option<String>,

    /// Create a temporary Git worktree from the repository in the current directory.
    ///
    /// If no branch is specified, the current HEAD is used.
    #[arg(long = "worktree", short = 'w')]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[arg(long, short = 'g')]
    pub git: Option<String>,

    /// Add benchmarking support using `criterion.rs`.
    ///
    /// Optionally specify the benchmark file name (default: `benchmark.rs`).
    #[arg(long, short = 'b')]
    pub bench: Option<Option<String>>,

    /// Select the Rust edition for the temporary project.
    /// Available editions: 2015, 2018, 2021, 2024.
    ///
    /// Default: latest stable edition.
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
