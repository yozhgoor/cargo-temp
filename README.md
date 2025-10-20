# cargo-temp

[![actions status][actions-badge]][actions-url]
[![crate version][crates-version-badge]][crates-url]
[![dependencies status][deps-badge]][deps-url]
![licenses][licenses-badge]

Quickly create disposable Rust project with pre-installed dependencies.

Ever wanted to test a new crate, an idea or debug a concept, launching the [rust-playground][rust-playground]
on your system? `cargo-temp` lets you create a fully functional Rust project in a temporary
directory.

- **Instant setup**: Create a new project with dependencies in one command.
- **No clean up required**: Projects are temporary by default but can be preserved if you change
  your mind.
- [**Git-friendly**](#git): Clone and test projects from Git repositories, with history truncated
  for a clean slate.
- [**Customizable**](#configuration): Tailor `cargo-temp`'s behavior with a simple configuration
  file.

## Install

```
cargo install --locked cargo-temp
```

## Usage

Create a new temporary project:
```
cargo-temp
```

This command open a shell in the project root, letting you test or prototype using
your favorite workflow. When you're done, just exit the shell and the project will be automatically
deleted.

If you change your mind and decide to keep the project, you can just delete the
`TO_DELETE` file and the directory will not be deleted when the shell or the
editor exits.

It's possible to specify the directory where the project will be preserved with the
`preserved_project_dir` [setting](#configuration).

### Dependencies

Specify one or more dependencies directly:
```
cargo-temp rand tokio
```

By default, the latest version (`*`) is used. To specify a version, use `=`:
```
cargo-temp anyhow=1.0
```

You can also use [Cargo's comparison requirements][comparison] for more control:
```
cargo-temp anyhow=<1.0.2
```

Add dependencies directly from a Git repository using HTTP or SSH URLs:
```
cargo-temp rand=https://github.com/rust-random/rand

cargo-temp rand=ssh://git@github.com/rust-random/rand.git
```

If you encounter issues with SSH, please refer to [this guide][ssh-issue].
If it doesn't help, please file an issue.

If no package name are provided, it will be parsed from the URL. For example, this will add the
`rand` package:
```
cargo-temp https://github.com/rust-random/rand.git
```

You can also specify a branch or a revision:
```
cargo-temp rand=https://github.com/rust-random/rand.git#branch=master

cargo-temp rand=https://github.com/rust-random/rand.git#rev=7e0f77a38
```

If neither is specified, the default branch of the repository is used.

You can add features to a dependency with `+`:
```
cargo-temp serde+derive

cargo-temp serde=1.0+derive

cargo-temp serde=https://github.com/serde-rs/serde#branch=master+derive
```

For multiple features, chain them with `+`:
```
cargo-temp serde=1.0+derive+alloc
```

### Git

#### Temporary Git Clone

If you want to create a project from a Git repository, you can use the `--git` option with
the repository's URL:
```
cargo-temp --git <url>
```

Cargo-temp truncates the history to the last commit by default. You can choose how many commits
will stay in the history from the [configuration file](#configuration)

#### Git Working Tree

You can create a [git worktree][worktree] from the current repository using:
```
cargo-temp --worktree
```

This will create a new working tree at the current HEAD.
You can specify a branch like this:
```
cargo-temp --worktree <branch>
```

When exiting the shell (or your editor) the working tree will be cleaned up.
Equivalent to `git worktree prune`.

### Benchmarking

If you want to create a temporary project with benchmarking using [`criterion-rs`][criterion], you
can use the `--bench` option with an optional name for the benchmark file:
```
cargo-temp --bench my_benchmark
```

> [!NOTE]
> Other options are available via the command line, you can learn more about them by using
> `cargo-temp --help`.

## Configuration

The configuration file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically.
`CONFIG_DIR` is determined by following the [XDG system][xdg] for both Linux and OSX and the
[Known Folder system][knownfolder] on Windows.

A documented example is available [here][config_template].

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
[xdg]: https://docs.rs/xdg/latest/xdg
[knownfolder]: https://docs.rs/dirs-2/latest/dirs_2
