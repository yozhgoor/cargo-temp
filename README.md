# cargo-temp

[![actions status][actions-badge]][actions-url]
[![crate version][crates-version-badge]][crates-url]
[![dependencies status][deps-badge]][deps-url]
![licenses][licenses-badge]

Quickly create disposable Rust project with pre-installed dependencies.

Ever wanted to test a new crate, an idea or debug a concept, like in the [rust-playground][rust-playground]
but on your system? `cargo-temp` lets you create a fully functional Rust project in a temporary
directory.

- **Instant setup**: Create a new project with dependencies in one command.
- **No clean up required**: Projects are temporary by default but can be preserved if you change your
  mind.
- [**Git-friendly**][#git]: Clone and test projects from Git repos, with history truncated for a clean
  slate.
- [**Customizable**][#configuration]: Tailor `cargo-temp`'s behavior with a simple configuration file.

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
deleted. However, if you want to keep the project, simply remove the [`TO_DELETE`](#features) file
before exiting.

## Command-line arguments

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

### Repositories

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

## Features

### The TO_DELETE file

If you change your mind and decide to keep the project, you can just delete the
`TO_DELETE` file and the directory will not be deleted when the shell or the
editor exits.

It's possible to specify the directory where the project will be preserved with the
`preserved_project_dir` [setting](#configuration).

### Git

#### Temporary Git Clone

If you want to create a project from a Git repository, you can use the `--git` option with
the repository's URL:

```
cargo-temp --git <url>
```

Cargo-temp truncates the history to the last commit by default. You can change this behavior in the
config file:

* You can choose how many commits will stay in the history.
    ```toml
    git_repo_depth = 3
    ```
    This will leave the 3 last commits of the history.
* If you do not want to truncate the history, you can set the `git_repo_depth`
    to false.
    ```toml
    git_repo_depth = false
    ```

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

### Edition

If you want to specify a specific edition for the temporary project, you can use the `--edition`
option:
```
cargo-temp --edition 2015
```

TODO: This shouldn't be in the readme but in the `--help` and error output
The available options are:
* `15` or `2015`
* `18` or `2018`
* `21` or `2021`
If the argument doesn't match these options, the default is the latest edition.

### Project name

If you want to provide a specific project name, you can use the `--name` option:
```
cargo-temp --name project
```

This name will be used as the suffix of the temporary project directory, like `tmp-wXyZ-project`.
If you decide to preserve the project, the directory will be renamed to match the project's name.

## Configuration


The configuration file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically following the [XDG system][xdg]
for both Linux and OSX and the [Known Folder system][knownfolder] on Windows.

| Setting | Default | Type | Description |
| --- | --- | --- | --- |
| `welcome_message` | true | bool | Welcome message explaining how to exit the project and how to preserve it. |
| `temporary_project_dir` | system cache directory | path | Path were the temporary projects are created. |
| `cargo_target_dir` | None | path | Cargo's target directory override. |
| `preserved_project_dir` | `temporary_project_dir` | path | Path to the directory where you want to preserve a saved project. |
| `prompt` | false | bool | Enable a prompt that ask a confirmation before deleting the project on exit. |
| `vcs` | "git" | String | Specify the VCS Cargo will use for your projects. |

### Editor

You can use `editor` to start an IDE instead of a shell
and `editor_args` to provide its arguments. These settings are unset by default.

* Example to run VS Code on Unix
    ```toml
    editor = "/usr/bin/code"
    editor_args = [ "--wait", "--new-window" ]
    ```

* Example to run VS Code on Windows
    ```toml
    editor = "C:\\Program Files\\Microsoft VS Code\\Code.exe"
    editor_args = [ "--wait", "--new-window" ]
    ```

### Subprocesses

You can spawn subprocess along your temporary shell like this:
```toml
[[subprocess]]
command = "alacritty -e cargo watch -x run"
foreground = false
```

The `command` field is a shell command like `echo Hello`.
The `foreground` field allows to run the program in foreground instead of
background.

#### Additional settings

* `working_dir` overrides the default working directory. The default is to use
  the temporary directory that has been created.
* `keep_on_exit` is used to keep the process alive after exiting the shell.
  The default is to kill the process when the shell exits. This setting doesn't
  work with foreground process.

##### Platform specific

Unix:
* `stdout` and `stderr` settings allows enabling or disabling outputs. With a
  background process, the default will be false, with a foreground process, the
  default will be true. The `stdin` setting doesn't exist since it's always
  disabled.

Windows:
* `inherit_handles` allows handles inheritance - If this parameter is true, each
  inheritable handle in the calling process is inherited by the new process. If
  the parameter is false, the handles are not inherited
  (see [CreateProcessW][CreateProcessW]).

[actions-badge]: https://github.com/yozhgoor/cargo-temp/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/yozhgoor/cargo-temp/actions
[crates-version-badge]: https://img.shields.io/crates/v/cargo-temp
[crates-url]: https://crates.io/crates/cargo-temp
[deps-badge]: https://deps.rs/repo/github/yozhgoor/cargo-temp/status.svg
[deps-url]: https://deps.rs/crate/cargo-temp
[licenses-badge]: https://img.shields.io/crates/l/cargo-temp
[rust-playground]: https://play.rust-lang.org
[comparison]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#comparison-requirements
[criterion]: https://docs.rs/criterion/latest/criterion
[xdg]: https://docs.rs/xdg/latest/xdg/
[knownfolder]: https://docs.rs/dirs-2/latest/dirs_2/
[ssh-issue]: https://github.com/rust-lang/cargo/issues/1851
[worktree]: https://git-scm.com/docs/git-worktree
[CreateProcessW]: https://docs.rs/CreateProcessW/latest/CreateProcessW/struct.Command.html#method.inherit_handles
