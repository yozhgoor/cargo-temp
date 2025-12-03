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
        "Each DEPENDENCY can take one of the following forms:

    (<NAME> | <URL>[#<BRANCH> | <REV>])[=|==|>|<|>=|<=|~<VERSION>][+][+<FEATURE>...]

You must provide either a `NAME` (e.g. `anyhow`) or a `URL` pointing to a git repository. URLs can
use `http(s)` or `ssh` schemes and may include a branch or a revision using `#`.

You can optionally specify a version with `=` (e.g. `tokio=1.48`) or with an operator that follows
Cargo's comparison requirements:

- `==`: Exact version (e.g. `tokio==1.48`).
- `>`: Maximal version (e.g. `tokio>1.48`).
- `<`: Minimal version (e.g. `tokio<1.48`).
- `>=`: Maximal or equal version (e.g. `tokio>=1.48`).
- `<=`: Minimal or equal version (e.g. `tokio<=1.48`).
- `~`: Minimal version with some ability to update (e.g. `tokio~1`).

Features can be enabled by appending them with `+` (e.g. `clap+derive` or `clap+derive+cargo`.
To disable default features, prefix the first feature with an additional `+`(e.g. `ratatui++termion`
or `ratatui++termion+serde`).

# Examples

```sh
cargo temp anyhow
cargo temp tokio=1.48
cargo temp clap+derive
cargo temp https://github.com/rust-random/rand#thread_rng
cargo temp ssh://git@github.com/ratatui/ratatui.git#latest++termion
```"
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
