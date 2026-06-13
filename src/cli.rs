use crate::dependency::Dependency;

/// cargo-temp: Create temporary Rust projects with specified dependencies.
///
/// A Cargo subcommand for quick experimentation. It creates a temporary project, adds any
/// dependencies you specify and drops you into a shell.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to the project's `Cargo.toml`.
    #[arg(long_help(
        "Each dependency is:
- A NAME from crates.io
- A git URL (http/https/ssh)
- A local PATH (./, ../, /, or X:\\ prefix).

A git URL may include `#BRANCH`, `#TAG`, or `#REV` to select a ref. The ref is
treated as a branch unless it is 7+ hex digits (revision) or looks like a
version (tag). Use an explicit prefix (`#branch:`, `#tag:`, `#rev:`) to
override the detection.

Version: `<OP><VERSION>` where OP is:
- `=` or `==` (exact)
- `>` (greater)
- `<` (less)
- `>=` (greater or equal)
- `<=` (less or equal)
- `~` (major.minor compatible)

Features: `+<FEATURE>` (repeatable).
`+` alone disables defaults.
`++name` does both (disables defaults and adds the feature).
Version must precede features.

Examples:
    anyhow
    tokio=1.48
    clap+derive
    anyhow=1.0.100+backtrace

    https://github.com/rust-random/rand#thread_rng
    https://github.com/rust-random/rand#0.9.0
    ssh://git@github.com/serde-rs/serde.git

    ./my-local-crate
    ../shared-lib"
    ))]
    pub dependencies: Vec<Dependency>,

    /// Create a library project instead of a binary.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Select the Rust edition for the temporary project.
    ///
    /// Default to the latest stable edition. Possible values are the same as `cargo new --edition`.
    #[arg(long, short = 'e')]
    pub edition: Option<String>,

    /// Name of the temporary crate.
    ///
    /// This name is used as the directory suffix to avoid conflicts.
    /// If you preserve the project, the directory will be renamed to the value provided.
    #[arg(long = "name", short = 'n')]
    pub project_name: Option<String>,

    /// Add benchmarking support using `criterion.rs`.
    ///
    /// Optionally specify the benchmark file name (default: `benchmark.rs`).
    #[arg(long, short = 'b')]
    pub bench: Option<Option<String>>,

    /// Create a temporary Git worktree from the repository in the current directory.
    ///
    /// If no branch is specified, the current HEAD is used.
    #[arg(long = "worktree", short = 'w')]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[arg(long, short = 'g')]
    pub git: Option<String>,
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
