use crate::config::{Config, Depth};
use crate::Dependency;
use anyhow::{ensure, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, process};
use tempfile::TempDir;

pub fn generate_tmp_project(
    worktree_branch: Option<Option<String>>,
    project_name: Option<String>,
    lib: bool,
    git: Option<String>,
    temporary_project_dir: PathBuf,
    git_repo_depth: Option<Depth>,
    vcs: Option<String>,
) -> Result<TempDir> {
    let tmp_dir = {
        let mut builder = tempfile::Builder::new();

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
    } else if let Some(url) = git {
        let mut command = process::Command::new("git");
        command.arg("clone").arg(url).arg(&tmp_dir.as_ref());

        match git_repo_depth {
            Some(Depth::Active(false)) => {}
            None | Some(Depth::Active(true)) => {
                command.arg("--depth").arg("1");
            }
            Some(Depth::Level(level)) => {
                command.arg("--depth").arg(level.to_string());
            }
        };

        ensure!(
            command.status().context("Could not start git")?.success(),
            "Cannot clone repository"
        );
    } else {
        let mut command = process::Command::new("cargo");
        command
            .current_dir(&tmp_dir)
            .args(["init", "--name", project_name.as_str()]);

        if lib {
            command.arg("--lib");
        }

        if let Some(arg) = vcs {
            command.args(["--vcs", arg.as_str()]);
        }

        ensure!(
            command.status().context("Could not start cargo")?.success(),
            "Cargo command failed"
        );
    }

    Ok(tmp_dir)
}

pub fn add_dependencies_to_project(tmp_dir: &Path, dependencies: &[Dependency]) -> Result<()> {
    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.join("Cargo.toml"))?;
    for dependency in dependencies.iter() {
        match dependency {
            Dependency::CrateIo {
                name: n,
                version: v,
                features: f,
            } => match (&v, &f) {
                (Some(version), None) => writeln!(toml, "{} = \"{}\"", n, version)?,
                (None, Some(features)) => todo!(),
                (Some(version), Some(features)) => todo!(),
                (None, None) => writeln!(toml, "{} = \"*\"", n)?,
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

pub fn generate_delete_file(tmp_dir: &Path) -> Result<PathBuf> {
    let delete_file = tmp_dir.join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    Ok(delete_file)
}

pub fn start_shell(config: &Config, tmp_dir: &Path) -> Result<()> {
    let mut shell_process = match config.editor {
        None => process::Command::new(get_shell()),
        Some(ref editor) => {
            let mut ide_process = process::Command::new(editor);
            ide_process
                .args(config.editor_args.iter().flatten())
                .arg(tmp_dir);
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

pub fn clean_up(
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
