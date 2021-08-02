# cargo-temp

A CLI tool that allows you to create a new rust project in a temporary directory
with already installed dependencies.

![Cargo-temp demo](t-rec.gif)

## Install

Requires Rust 1.51.

`cargo install cargo-temp`

## Usage

Create a new temporary project:

### With no additional dependencies:

```bash
cargo-temp
```

### With multiple dependencies:

```bash
cargo-temp rand tokio
```

### When specifying a version:

```bash
cargo-temp anyhow=1.0`
```

Using the [cargo's comparison requirements][comparison]:

* Exact version:
  ```bash
  cargo-temp anyhow==1.0.13
  ```

* Maximal version:
  ```bash
  cargo-temp anyhow=<1.0.2
  ```

### Repositories

You can add repositories to your *Cargo.toml*.

Examples:

* HTTP:
  ```bash
  cargo-temp anyhow=https://github.com/dtolnay/anyhow.git
  ```

* SSH
  ```bash
  cargo-temp anyhow=ssh://git@github.com/dtolnay/anyhow.git
  ```

To choose a branch or a revision:

* Branch:
  ```bash
  cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#branch=master
  ```

* Revision:
  ```bash
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

```bash
cargo-temp --worktree
```
<!-- git worktree -d <temp_dir>] -->

This will create a new working tree at the current HEAD.
You can specify a branch like this:

```bash
cargo-temp --worktree <branch>
```
<!-- git worktree <temp_dir> <branch>-->

When exiting the shell (or your editor) the working tree will be cleaned up.
Equivalent to `git worktree prune`

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

[comparison]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#comparison-requirements
[xdg]: https://docs.rs/xdg/2.2.0/xdg/
[knownfolder]: https://docs.rs/dirs-2/3.0.1/dirs_2/
