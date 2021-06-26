# cargo-temp

A CLI tool that allows you to create a new rust project in a temporary directory
with already installed dependencies.

![Cargo-temp demo](t-rec.gif)

## Install

Requires Rust 1.51.

`cargo install cargo-temp`

## Usage

Create a new temporary project:

* With no additional dependencies:
    `$ cargo-temp`

* With multiple dependencies:
    `$ cargo-temp rand tokio`

* With a dependency that have a fixed version:
    `$ cargo-temp anyhow==1.0.13`

### Repositories

You can add repositories to you Cargo.toml.

Examples:

* HTTP
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git`

* SSH
    `$ cargo-temp anyhow=ssh://git@github.com/dtolnay/anyhow.git`

This will add the repository on the default branch by default. You can choose
another branch or a revision:

* Branch
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#branch=master`

* Revision
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#rev=7e0f77a38`

## Features

If you change your mind and decide to keep the project you can just delete the
`TO_DELETE` file and the directory will not be deleted when the shell exits.

## Settings

The config file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically.
We use the [XDG system](https://docs.rs/xdg/2.2.0/xdg/) for both Linux and OSX
and the [Know Folder system](https://docs.rs/dirs-2/3.0.1/dirs_2/) on Windows.

### `temporary_project_dir`

The path where the temporary projects are created.
Set on the cache directory by default.

`temporary_project_dir = "/home/name/.cache/cargo-temp/"`

### `cargo_target_dir`

Cargo's target directory override.
This setting is unset by default and will be ignored if the `CARGO_TARGET_DIR`
environment variable is already set.

`temporary_project_dir = "home/name/repos/tmp"`

### editor (unset by default)

You can use `editor` and `editor_args` as respectively the path to an IDE to
start instead of a shell and arguments for it (unset by default).

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
