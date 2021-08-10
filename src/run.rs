use crate::config::Config;
use crate::{Cli, Dependency};
use anyhow::{ensure, Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process};
use tempfile::{Builder, TempDir};

pub fn start(cli: Cli, config: &Config) -> Result<()> {
    let tmp_dir = generate_tmp_project(
        cli.worktree_branch.clone(),
        cli.project_name,
        cli.lib,
        config.temporary_project_dir.clone(),
    )?;

    add_dependencies_to_toml(&tmp_dir, cli.dependencies)?;

    let delete_file = generate_delete_file(&tmp_dir)?;

    start_shell(config, &tmp_dir)?;

    exit_shell(delete_file, tmp_dir, cli.worktree_branch)?;

    Ok(())
}

fn generate_tmp_project(
    worktree_branch: Option<Option<String>>,
    project_name: Option<String>,
    lib: bool,
    temporary_project_dir: String,
) -> Result<TempDir> {
    let tmp_dir = {
        let mut builder = Builder::new();

        if worktree_branch.is_some() {
            builder.prefix("wk-");
        } else {
            builder.prefix("tmp-");
        }

        builder.tempdir_in(temporary_project_dir)?
    };

    let project_name = project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
    });

    if let Some(maybe_branch) = worktree_branch.as_ref() {
        let mut command = process::Command::new("git");
        command.args(["worktree", "add"]);

        match maybe_branch {
            Some(branch) => command.arg(tmp_dir.path()).arg(branch),
            None => command.arg("-d").arg(tmp_dir.path()),
        };

        ensure!(
            command.status().context("Could not start git")?.success(),
            "Cannot create working tree"
        );
    } else {
        let mut command = process::Command::new("cargo");
        command
            .current_dir(&tmp_dir)
            .args(["init", "--name", project_name.as_str()]);

        if lib {
            command.arg("--lib");
        }

        ensure!(
            command.status().context("Could not start cargo")?.success(),
            "Cargo command failed"
        );
    }

    Ok(tmp_dir)
}

fn add_dependencies_to_toml(tmp_dir: &TempDir, dependencies: Vec<Dependency>) -> Result<()> {
    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.path().join("Cargo.toml"))?;
    for dependency in dependencies.iter() {
        match dependency {
            Dependency::CrateIo(s, v) => match &v {
                Some(version) => writeln!(toml, "{} = \"{}\"", s, version)?,
                None => writeln!(toml, "{} = \"*\"", s)?,
            },
            Dependency::Repository {
                name,
                url,
                branch,
                rev,
            } => {
                write!(toml, "{name} = {{ git = {url:?}", name = name, url = url)?;
                if let Some(branch) = branch {
                    write!(toml, ", branch = {:?}", branch)?;
                }
                if let Some(rev) = rev {
                    write!(toml, ", rev = {:?}", rev)?;
                }
                writeln!(toml, " }}")?;
            }
        }
    }

    Ok(())
}

fn generate_delete_file(tmp_dir: &TempDir) -> Result<PathBuf> {
    let delete_file = tmp_dir.path().join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    Ok(delete_file)
}

fn start_shell(config: &Config, tmp_dir: &TempDir) -> Result<()> {
    let mut shell_process = match config.editor {
        None => process::Command::new(get_shell()),
        Some(ref editor) => {
            let mut ide_process = process::Command::new(editor);
            ide_process
                .args(config.editor_args.iter().flatten())
                .arg(tmp_dir.path());
            ide_process
        }
    };

    if env::var("CARGO_TARGET_DIR").is_err() {
        if let Some(path) = &config.cargo_target_dir {
            shell_process.env("CARGO_TARGET_DIR", path);
        }
    }

    shell_process
        .current_dir(&tmp_dir)
        .status()
        .context("Cannot start shell")?;

    #[cfg(windows)]
    if config.editor.is_some() {
        unsafe {
            cargo_temp_bindings::Windows::Win32::SystemServices::FreeConsole();
        }
    }

    Ok(())
}

pub fn exit_shell(
    delete_file: PathBuf,
    tmp_dir: TempDir,
    worktree_branch: Option<Option<String>>,
) -> Result<()> {
    if !delete_file.exists() {
        println!(
            "Project directory preserved at: {}",
            tmp_dir.into_path().display()
        );
    } else if worktree_branch.is_some() {
        let mut command = process::Command::new("git");
        command
            .args(["worktree", "remove"])
            .arg(&tmp_dir.path())
            .arg("--force");
        ensure!(
            command.status().context("Could not start git")?.success(),
            "Cannot remove working tree"
        );
    }

    Ok(())
}

pub fn get_shell() -> String {
    #[cfg(unix)]
    {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }

    #[cfg(windows)]
    {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string())
    }
}
