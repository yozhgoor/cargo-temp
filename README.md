# cargo-temp

A CLI tool for linux (currently) that allow you to create a new rust project in a temporary directory with
already installed dependencies.

![Cargo-temp demo](t-rec.gif)

Only \*nix OS are supported for now because a shell is ran while the project is being edited by the user.
It would be nice to have this working on Windows.
If you know how to achieve this, please open an issue to tell us (or a PR.)

## Usage

Create a new temporary project:

* With no additional dependencies:
    `$ cargo-temp`

* With multiple dependencies:
    `$ cargo-temp rand tokio`

* With a dependency that have a fixed version:
    `$ cargo-temp anyhow==1.0.13`

To set the name of the project use `-n <name>` or `--name <name>`.
The default is the name of the temporary directory.
Ex: `$ cargo-temp seed --name seed-playground`

## Features

If you change your mind and decide to keep the project you can just delete the `TO_DELETE` file and the directory
will not be deleted when the shell exits.

## Settings

The config file is located at `{CONFIG_DIR}/cargo-temp/config.toml`.
When you run `cargo-temp` for the first time it will be created automatically

* `temporary_project_dir`: path where the temporary projects are created (cache directory by default).
* `cargo_target_dir`: cargo's target directory override (unset by default).
   This setting is ignored if `CARGO_TARGET_DIR` is already set.
