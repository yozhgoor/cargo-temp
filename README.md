# cargo-temp

[![actions status][actions-badge]][actions-url]
[![crate version][crates-version-badge]][crates-url]
[![dependencies status][deps-badge]][deps-url]
![licenses][licenses-badge]

[actions-badge]: https://github.com/yozhgoor/cargo-temp/workflows/main/badge.svg
[actions-url]: https://github.com/yozhgoor/cargo-temp/actions
[crates-version-badge]: https://img.shields.io/crates/v/cargo-temp
[crates-url]: https://crates.io/crates/cargo-temp
[deps-badge]: https://deps.rs/crate/cargo-temp/0.2.10/status.svg
[deps-url]: https://deps.rs/crate/cargo-temp
[licenses-badge]: https://img.shields.io/crates/l/cargo-temp

A CLI tool that allows you to create a new rust project in a temporary directory
with already installed dependencies.

![Cargo-temp demo](t-rec.gif)

## Install

Requires Rust 1.51.

```
cargo install cargo-temp
```

If you want to use the `generate` feature, Rust 1.61 is required.

```
cargo install --all-features cargo-temp
```

Or

```
cargo install --features generate cargo-temp
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
    cargo-temp anyhow=1.0
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
    cargo-temp anyhow=https://github.com/dtolnay/anyhow
    ```

* SSH
    ```
    cargo-temp anyhow=ssh://git@github.com/dtolnay/anyhow.git
    ```

If you have some problems to add a dependency over SSH, please refer to this:
[Support SSH Git URLs](ssh-issue). If it doesn't help, please fill an issue.

* Without package name
    ```
    cargo-temp https://github.com/dtolnay/anyhow.git
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

### Dependencies features

You can add features to a dependency with `+`.

Examples:

* A dependency with feature
    ```
    cargo-temp serde+derive
    ```

* A dependency with version and feature
    ```
    cargo-temp serde=1.0+derive
    ```

* A repository with branch and feature
    ```
    cargo-temp serde=https://github.com/serde-rs/serde#branch=master+derive
    ```

* Without specifying package name
    ```
    cargo-temp https://github.com/tokio-rs/tokio#branch=compat+io_std
    ```

If you want to add multiple features you can do it with `+`, like this:

```
cargo-temp serde=1.0+derive+alloc
```

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

If you want to create a temporary project from a Git repository, you can use the `--git` option with
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

`git_repo_depth = true` is the same as the default behavior.

### Benchmarking

If you want to create a temporary project with benchmarking using [`criterion-rs`][criterion], you
can use the `--bench` option with an optional name for the benchmark file:

```
cargo-temp --bench my_benchmark
```

The resulting directory layout will look like this:

```
tmp-id/
├── benches
│   └── my_benchmark.rs
├── Cargo.toml
├── src
│   └── main.rs
└── TO_DELETE
```

This will also adds these lines to the `Cargo.toml` of the project:

```toml
[dev-dependencies]
criterion = "*"

[profile.release]
debug = true

[[bench]]
name = "my_benchmark"
harness = false
```

And finally, the benchmark file contains some imports and a `Hello, world!` example:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(_c: &mut Criterion) {
    println!("Hello, world!");
}

criterion_group!(
    benches,
    criterion_benchmark
);
criterion_main!(benches);
```

### Edition

If you want to specify a specific edition for the temporary project, you can use the `--edition`
option:

```
cargo-temp --edition 2015
```

The available options are:

* 15 or 2015
* 18 or 2018
* 21 or 2021

If the argument doesn't match these options, the default is the latest edition.

### Generate

Generate a temporary project from a template using [`cargo-generate`][cargo-generate]
(current used version is v0.16)

For example, to create a temporary project using
[this template][cargo-generate-example-template]:

```
cargo temp generate --git https://github.com/ashleygwilliams/wasm-pack-template
```

Almost all the features of the [`cargo-generate`][cargo-generate] CLI are provided as it,
expections are:

* `--init` - This flag is used by cargo-temp to init the template as the temporary directory.
* `--destination` - Since we already use `--init`, this feature cannot be used with cargo-temp.
* `--allow-commands` - Needs further investigation to avoid conflicts with cargo-temp's subprocess.

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

If the directory doesn't exist, it will be created with all of its parent components if
they are missing.

### Cargo target directory

Cargo's target directory override.
This setting is unset by default and will be ignored if the `CARGO_TARGET_DIR`
environment variable is already set.

```toml
cargo_target_dir = "/home/name/repos/tmp"
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

### Enable a prompt that ask a confirmation before deleting the project

cargo-temp will automatically delete the temporary project if the flag file `TO_DELETE` exists
in the project when exiting the shell. If you prefer to enable a prompt that ask you if you want to
delete the project, you can add this to your `config.toml`:

```toml
prompt = true
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

#### Examples

* Unix:
    ```toml
    [[subprocess]]
    command = "cargo run"
    foreground = true

    [[subprocess]]
    command = "firefox"
    foreground = false
    ```
* Windows
    ```toml
    [[subprocess]]
    command = "cargo.exe run"
    foreground = true

    [[subprocess]]
    command = "firefox.exe"
    foreground = false
    ```

[comparison]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#comparison-requirements
[criterion]: https://docs.rs/criterion/latest/criterion
[cargo-generate]: https://github.com/cargo-generate/cargo-generate
[cargo-generate-example-template]: https://github.com/ashleygwilliams/wasm-pack-template
[xdg]: https://docs.rs/xdg/latest/xdg/
[knownfolder]: https://docs.rs/dirs-2/latest/dirs_2/
[ssh-issue]: https://github.com/rust-lang/cargo/issues/1851
[CreateProcessW]: https://docs.rs/CreateProcessW/latest/CreateProcessW/struct.Command.html#method.inherit_handles
