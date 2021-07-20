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
* When specifying a version:
    `$ cargo-temp anyhow=1.0`
    * Using the [cargo's comparison requirements][comparison]:
        `$ cargo-temp anyhow==1.0.13`

### Repositories

You can add repositories to your `Cargo.toml`.

Examples:

* HTTP
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git`

* SSH
    `$ cargo-temp anyhow=ssh://git@github.com/dtolnay/anyhow.git`

This will add the repository on the last commit of the main branch by default. 
You can choose another branch or a revision:

* Branch
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#branch=master`

* Revision
    `$ cargo-temp anyhow=https://github.com/dtolnay/anyhow.git#rev=7e0f77a38`

## Features

If you change your mind and decide to keep the project you can just delete the
`TO_DELETE` file and the directory will not be deleted when the shell or the
editor exits.

## Settings

The config file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically.
We use the [XDG system][xdg] for both Linux and OSX
and the [Known Folder system][knownfolder] on Windows.

### Temporary project directory

The path where the temporary projects are created.
Set on the cache directory by default.

`temporary_project_dir = "/home/name/.cache/cargo-temp/"`

### Cargo target directory

Cargo's target directory override.
This setting is unset by default and will be ignored if the `CARGO_TARGET_DIR`
environment variable is already set.

`temporary_project_dir = "/home/name/repos/tmp"`

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
