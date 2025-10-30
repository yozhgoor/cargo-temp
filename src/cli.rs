use crate::dependency::Dependency;

/// cargo-temp: Create disposable Rust projects with pre-installed dependencies.
///
/// This tool creates a new Rust project in a temporary directory, automatically installs the
/// dependencies you specify and deletes the project on exit, unless you choose to preserve it.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to the project's `Cargo.toml`.
    #[arg(long_help(
"Each DEPENDENCY can take one of the following forms:

    (<NAME> | <URL>[#<BRANCH> | <REV>])[=<VERSION>][%default][<FEATURE>...]

You must provide either a `NAME` (e.g. `anyhow`) or a `URL` pointing to a git repository.

URLs can use `http(s)` or `ssh` schemes and may include a branch or a revision using `#`.

You can optionally a version with `=` (e.g. `tokio=1.48`). Operators following cargo's
comparison requirements can also be provided after the `=`:

- `=`: Exact version (e.g. `tokio==1.48`).
- `>`: Maximal version (e.g. `tokio=>1.48`).
- `<`: Minimal version (e.g. `tokio=<1.48`).
- `~`: Minimal version with some ability to update (e.g. `tokio=~1`).

Features can be enabled using `+` (e.g. `derive+derive` or `clap+derive+cargo`. If you want
to disable the default features, you can use `%default` (e.g. `ratatui%default+termion`).

# Examples

```sh
cargo temp anyhow
cargo temp tokio=1.48
cargo temp clap+derive
cargo temp https://github.com/rust-random/rand#thread_rng
cargo temp ssh://git@github.com/ratatui/ratatui.git=0.28%default+termion
```"
))]
    pub dependencies: Vec<Dependency>,

    /// Create a library project instead of a binary.
    ///
    /// This generate a `lib.rs` file instead of `main.rs`.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Name of the temporary crate.
    ///
    /// This name is used as the directory suffix to avoid conflicts. If you preserve the project,
    /// the directory will be renamed to the value you provided.
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
