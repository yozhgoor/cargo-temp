# cargo-temp

A CLI tool for linux (currently) that allow you to create a new rust project in a temporary directory with already installed dependencies.

![Cargo-temp demo](t-rec.gif)

Only *nix OS are supported for now because a shell is ran while the project is being edited by the user.
It would be nice to have this working on Windows.
If you know how to this, please open an issue to tell us (or a PR.)

## Usage

Create a new temporary project:

* With no additional dependencies:
    `$ cargo-temp`

* With multiple dependencies:
    `$ cargo-temp rand tokio`

* With a dependency that have a fixed version:
    `$ cargo-temp anyhow==1.0.13`

## Features

If after a bit of play with you project you want to keep your work, just delete the `TO_DELETE` file.
When you run `cargo-temp` for the first time he will create `cargo-temp/config.toml` in your config directory.

This file will contain one line a field `temporary-project-dir` and the value of this field is the path where `cargo-temp` will create your temporary project.
The default path is your cache directory, but you can override it by simply change the value of the field.

We provide one another optional field `cargo_target_dir`, that allow you to override the path where the target directory of you temporary project will be created.
