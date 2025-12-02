# cargo-temp

[![actions status][actions-badge]][actions-url]
[![crate version][crates-version-badge]][crates-url]
[![dependencies status][deps-badge]][deps-url]
![licenses][licenses-badge]

**Create temporary Rust projects with specified dependencies**

`cargo-temp` is a Cargo subcommand for quick experimentation, whether you're trying out a
crate, testing an idea or exploring APIs and language features. It creates a temporary project, adds
any dependencies you specify, and drops you into a shell so you can prototype using your usual
tools.

When you're done, simply exit the shell. The project is deleted automatically, unless you choose
to preserve it.

**Why use `cargo-temp`**?

- No more abandoned scratch projects, temporary by default
- Support editors, local toolchain and configuration.
- Add crates (including Git sources) on creation, with versions, features, branch, etc.
- Run experiments from a Git worktree or a temporary repository clone
- Setup Criterion-based benchmarks with one flag.

Think `cargo new`, but temporary, dependency-aware, Git-friendly and benchmark ready. A middle
ground between [the Rust playground][rust-playground] and a full repository.

## Install

Install `cargo-temp` using Cargo:

```
cargo install --locked cargo-temp
```

## Usage

Create a new temporary Rust project:
```
cargo temp
```

A shell is opened in the project directory, so you can immediately start editing, building and
running code right away. You can also [configure][config-template] `cargo-temp` to open directly the
project in your preferred editor instead of a shell.

When you exit the shell (or the editor), the project and all its files are automatically deleted.
To preserve it, simply delete the `TO_DELETE` file before exiting. The project directory will remain
intact or you can also [configure][config-template] a default destination for the preserved
projects.

### Dependencies

Specify one or more dependencies directly:
```
cargo temp rand log
```

By default, the latest version (`*`) is used. To specify a version, use `=`:
```
cargo temp anyhow=0.4
```

For more control, use [Cargo's comparison requirements][comparison]:
```
cargo temp log==0.4.28

cargo temp log>=0.4.28

cargo temp log<=0.4.28

cargo temp log~0.4.28
```

#### dependencies from Git

Add dependencies directly from a Git repository using HTTP or SSH URLs:
```
cargo temp https://github.com/rust-random/rand

cargo temp ssh://git@github.com/rust-random/rand.git
```

The name of the package is inferred from the URL and the `.git` extension is optional.

> [!NOTE] For SSH issues, please refer to [this guide][ssh-issue].
> For private repositories, ensure you have the necessary SSH keys or credentials.
> If it doesn't help, please file an issue.

You can also specify a branch or a revision:
```
cargo temp https://github.com/rust-random/rand.git#master

cargo temp https://github.com/rust-random/rand.git#7e0f77a38
```

### Features

Add features to a dependency using `+`:
```
cargo temp tokio+io_std

cargo temp tokio=1.0+io_std

cargo temp https://github.com/tokio-rs/tokio#compat+io_std
```

For multiple features, chain them together:
```
cargo temp tokio==1.0+io_std+io_utils
```

To disable default features, prefix the chain of features by a `+`:
```
cargo temp tokio+

cargo temp tokio++io_std
```

### Git

#### Clone a Git Repository

Create a temporary project from a Git repository using the `--git` option:
```
cargo temp --git <url>
```

By default, the Git history is truncated to the last commit. To retain more commits, adjust the
`git_repo_depth` setting in your configuration file.

#### Git Working Tree

Create a temporary [git worktree][worktree] from the current repository:
```
cargo temp --worktree
```

To create a worktree for a specific branch:
```
cargo temp --worktree <branch>
```

If no branch is specified, the current HEAD is used.

When exiting the shell or editor, the working tree will be cleaned up, equivalent to
`git worktree prune`.

### Benchmarking

Create a temporary project with benchmarking support using [`criterion-rs`][criterion]:
```
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
[config-template]: https://github.com/yozhgoor/cargo-temp/blob/main/config_template.toml
