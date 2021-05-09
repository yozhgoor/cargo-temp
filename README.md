# cargo-temp

A CLI tool that allow you to create a new rust project in a temporary directory with
already installed dependencies.

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

## Features

If you change your mind and decide to keep the project you can just delete the `TO_DELETE` file and the directory
will not be deleted when the shell exits.

## Settings

The config file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically

* `temporary_project_dir`: path where the temporary projects are created (cache directory by default).
* `cargo_target_dir`: cargo's target directory override (unset by default).
   This setting is ignored if `CARGO_TARGET_DIR` is already set.
