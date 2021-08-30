# cargo-temp

A CLI tool that allows you to create a new rust project in a temporary directory
with already installed dependencies.

![Cargo-temp demo](t-rec.gif)

## Install

Requires Rust 1.51.

```
cargo install cargo-temp
```

## Usage

Create a new temporary project:

* With no additional dependencies:
    ```
    cargo-temp
    ```

* With multiple dependencies:
    ```
    cargo-temp rand tokio
    ```

* When specifying a version:
    ```
    cargo-temp anyhow=1.0`
    ```

Using the [cargo's comparison requirements][comparison]:

* Exact version:
    ```
    cargo-temp anyhow==1.0.13
    ```

* Maximal version:
    ```
    cargo-temp anyhow=<1.0.2
    ```

### Repositories

You can add repositories to your `Cargo.toml`.

Examples:

* HTTP:
    ```
    cargo-temp anyhow=https://github.com/dtolnay/anyhow.git
    ```

* SSH
    ```
    cargo-temp anyhow=ssh://git@github.com/dtolnay/anyhow.git
    ```

To choose a branch or a revision:

* Branch:
    ```
    cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#branch=master
    ```

* Revision:
    ```
    cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#rev=7e0f77a38
    ```

Without a branch or a revision, cargo will use the default branch of the
repository.

## Features

### The TO_DELETE file

If you change your mind and decide to keep the project you can just delete the
`TO_DELETE` file and the directory will not be deleted when the shell or the
editor exits.

### Git Working Tree

You can create a git worktree from the current repository using:

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

### Temporary Git Clone

If you want to create a temporary project from a Git repository, you can use the `--git` option with the repository's URL:

```
cargo-temp --git <url>
```

Cargo-temp truncates the history to the last commit by default. You can change this behavior in the config file:

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

`git_repo_depth = true` is the same as the default behavior.

## Settings

The config file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically.
We use the [XDG system][xdg] for both Linux and OSX
and the [Known Folder system][knownfolder] on Windows.

### Temporary project directory

The path where the temporary projects are created.
Set on the cache directory by default.

```toml
temporary_project_dir = "/home/name/.cache/cargo-temp/"
```

### Cargo target directory

Cargo's target directory override.
This setting is unset by default and will be ignored if the `CARGO_TARGET_DIR`
environment variable is already set.

```toml
temporary_project_dir = "/home/name/repos/tmp"
```

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

### Use a VCS

By default, cargo-temp will use the default cargo VCS for your projects (which
is normally git), you can change that in the config file with the `vcs` option.

```toml
vcs = "pijul"
```

The possible values are

* pijul
* fossil
* hg
* git
* none

The `--vcs` value will be passed as is to cargo.

[comparison]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#comparison-requirements
[xdg]: https://docs.rs/xdg/2.2.0/xdg/
[knownfolder]: https://docs.rs/dirs-2/3.0.1/dirs_2/
