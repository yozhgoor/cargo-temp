# cargo-temp

[![actions status][actions-badge]][actions-url]
[![crate version][crates-version-badge]][crates-url]
[![dependencies status][deps-badge]][deps-url]
![licenses][licenses-badge]


**Quickly create disposable Rust projects with pre-installed dependencies.**

Tired of manually setting up and cleaning up Rust projects for testing crates, debugging concepts,
or prototyping ideas? `cargo-temp` automates the process, creating a fully functional Rust project
in a temporary directory with your specified dependencies.

When you're done, just exit the shell. Your project is automatically deleted unless you choose to
preserve it.


## Install

Install `cargo-temp` using Cargo:

```sh
cargo install --locked cargo-temp
```

## Usage

Create a new temporary Rust project:
```sh
cargo temp
```

This command opens a shell in the project directory, where you can immediately start coding. When
you exit the shell, the project and all its files are automatically deleted.

### Preserve the project

If you decide to keep the project, delete the `TO_DELETE` file in the project root before exiting
the shell. The project directory will remain intact.

You can also specify a directory where preserved projects will be moved in the configuration file by
setting `preserved_project_dir`.

### Add Dependencies

Specify one or more dependencies directly:
```sh
cargo temp rand tokio
```

By default, the latest version (`*`) is used. To specify a version, use `=`:
```sh
cargo temp anyhow=1.0
```

For more control, use [Cargo's comparison requirements][comparison]:
```sh
cargo temp anyhow=<1.0.2
```

### Add dependencies from Git

Add dependencies directly from a Git repository using HTTP or SSH URLs:
```sh
cargo temp rand=https://github.com/rust-random/rand

cargo temp rand=ssh://git@github.com/rust-random/rand.git
```

If no package name is provided, it is inferred from the URL. For example:
```sh
cargo temp https://github.com/rust-random/rand.git
```

> [!NOTE] For SSH issues, please refer to [this guide][ssh-issue].
> For private repositories, ensure you have the necessary SSH keys or credentials.
> If it doesn't help, please file an issue.

You can also specify a branch or a revision:
```sh
cargo temp rand=https://github.com/rust-random/rand.git#branch=master

cargo temp rand=https://github.com/rust-random/rand.git#rev=7e0f77a38
```

### Dependencies features

Add features to a dependency using `+`:
```sh
cargo temp serde+derive

cargo temp serde=1.0+derive

cargo temp serde=https://github.com/serde-rs/serde#branch=master+derive
```

For multiple features, chain them together:
```sh
cargo temp serde=1.0+derive+alloc
```

### Git

#### Clone a Git Repository

Create a temporary project from a Git repository using the `--git` option:
```sh
cargo temp --git <url>
```

By default, the Git history is truncated to the last commit. To retain more commits, adjust the
`git_repo_depth` setting in your configuration file.

#### Git Working Tree

Create a temporary [git worktree][worktree] from the current repository:
```sh
cargo temp --worktree
```

To create a worktree for a specific branch:
```sh
cargo temp --worktree <branch>
```

If no branch is specified, the current HEAD is used.

When exiting the shell or editor, the working tree will be cleaned up, equivalent to
`git worktree prune`.

### Benchmarking

Create a temporary project with benchmarking support using [`criterion-rs`][criterion]:
```sh
cargo temp --bench <name>
```

This adds `criterion` as a dev-dependency and generates a benchmark file named `<name>.rs` (default:
`benchmark.rs`).

> [!NOTE]
>
> For a full list of options, run `cargo temp --help`.

## Configuration

`cargo-temp` uses a configuration file located at:
- Linux/OSX: `~/.config/cargo-temp/config.toml`
- Windows: `%APPDATA%\cargo-temp\config.toml`

The file is created automatically when you run `cargo-temp` for the first time. For a detailed
example, see the [configuration template][configuration_template].

Common settings are:
- `temporary_project_dir`: Directory where temporary projects are created (defaults to system
  cache).
- `preserved_project_dir`: Directory where preserved projects are moved.
- `git_repo_depth`: Number of commits to retain when cloning.

[actions-badge]: https://github.com/yozhgoor/cargo-temp/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/yozhgoor/cargo-temp/actions
[crates-version-badge]: https://img.shields.io/crates/v/cargo-temp
[crates-url]: https://crates.io/crates/cargo-temp
[deps-badge]: https://deps.rs/repo/github/yozhgoor/cargo-temp/status.svg
[deps-url]: https://deps.rs/crate/cargo-temp
[licenses-badge]: https://img.shields.io/crates/l/cargo-temp
[rust-playground]: https://play.rust-lang.org
[comparison]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#comparison-requirements
[ssh-issue]: https://github.com/rust-lang/cargo/issues/1851
[worktree]: https://git-scm.com/docs/git-worktree
[criterion]: https://docs.rs/criterion/latest/criterion
[config_template]: https://github.com/yozhgoor/cargo-temp/blob/main/config_template.toml
