use anyhow::{bail, ensure, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tempfile::TempDir;

use crate::{
    config::{Config, Depth},
    dependency::{format_dependency, Dependency},
    Cli,
};

pub fn execute(cli: Cli, config: Config) -> Result<()> {
    let tmp_dir = generate_tmp_project(
        cli.clone(),
        &config.temporary_project_dir,
        config.git_repo_depth.as_ref(),
        config.vcs.as_deref(),
    )?;

    add_dependencies_to_project(tmp_dir.path(), &cli.dependencies)?;

    if let Some(maybe_bench_name) = &cli.bench {
        generate_benchmarking(tmp_dir.path(), maybe_bench_name.as_deref())?;
    }

    let delete_file = generate_delete_file(tmp_dir.path())?;

    let mut subprocesses = start_subprocesses(&config, tmp_dir.path());

    log::info!(
        "Temporary project created at: {}",
        &tmp_dir.path().display()
    );

    let res = start_shell(&config, tmp_dir.path());

    clean_up(
        &delete_file,
        tmp_dir,
        cli.worktree_branch.flatten().as_deref(),
        &mut subprocesses,
    )?;

    ensure!(res.is_ok(), "problem within the shell process");

    Ok(())
}

pub fn generate_tmp_project(
    cli: Cli,
    temporary_project_dir: &Path,
    git_repo_depth: Option<&Depth>,
    vcs: Option<&str>,
) -> Result<TempDir> {
    let tmp_dir = {
        let mut builder = tempfile::Builder::new();
        let mut suffix = String::new();

        let prefix = if cli.worktree_branch.is_some() {
            "wk-"
        } else {
            "tmp-"
        };

        if let Some(name) = cli.project_name.as_deref() {
            suffix = format!("-{}", name);
        };

        if !temporary_project_dir.exists() {
            fs::create_dir_all(temporary_project_dir)
                .context("cannot create temporary project's directory")?;
        }

        builder
            .prefix(&prefix)
            .suffix(&suffix)
            .tempdir_in(temporary_project_dir)?
    };

    let project_name = cli.project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
    });

    if let Some(maybe_branch) = cli.worktree_branch.as_ref() {
        let mut command = std::process::Command::new("git");
        command.args(["worktree", "add"]);

        match maybe_branch {
            Some(branch) => command.arg(tmp_dir.path()).arg(branch),
            None => command.arg("-d").arg(tmp_dir.path()),
        };

        ensure!(
            command.status().context("Could not start git")?.success(),
            "cannot create working tree"
        );
    } else if let Some(url) = &cli.git {
        let mut command = std::process::Command::new("git");
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
            "cannot clone repository"
        );
    } else {
        let mut command = std::process::Command::new("cargo");
        command
            .current_dir(&tmp_dir)
            .args(["init", "--name", project_name.as_str()]);

        if cli.lib {
            command.arg("--lib");
        }

        if let Some(arg) = vcs {
            command.args(["--vcs", arg]);
        }

        if let Some(num) = &cli.edition {
            match num {
                15 | 2015 => {
                    command.args(["--edition", "2015"]);
                }
                18 | 2018 => {
                    command.args(["--edition", "2018"]);
                }
                21 | 2021 => {
                    command.args(["--edition", "2021"]);
                }
                _ => log::error!("cannot find the {} edition, using the latest", num),
            }
        }

        ensure!(
            command.status().context("Could not start cargo")?.success(),
            "cargo command failed"
        );
    }

    Ok(tmp_dir)
}

pub fn add_dependencies_to_project(tmp_dir: &Path, dependencies: &[Dependency]) -> Result<()> {
    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.join("Cargo.toml"))?;

    for dependency in dependencies.iter() {
        writeln!(toml, "{}", format_dependency(dependency))?
    }

    Ok(())
}

fn generate_benchmarking(tmp_dir: &Path, maybe_name: Option<&str>) -> Result<()> {
    let name = if let Some(name) = maybe_name {
        name
    } else {
        "benchmark"
    };

    let mut toml = fs::OpenOptions::new()
        .append(true)
        .open(tmp_dir.join("Cargo.toml"))?;

    writeln!(toml, "{}", format_benchmarking(name))?;

    let bench_folder = tmp_dir.join("benches");
    fs::create_dir_all(&bench_folder)?;
    let mut bench_file = bench_folder.join(name);
    bench_file.set_extension("rs");

    fs::write(
        bench_file,
        "use criterion::{black_box, criterion_group, criterion_main, Criterion};\n\n\
        fn criterion_benchmark(_c: &mut Criterion) {\n\tprintln!(\"Hello, world!\");\n}\n\n\
        criterion_group!(\n\tbenches,\n\tcriterion_benchmark\n);\ncriterion_main!(benches);",
    )?;

    Ok(())
}

fn format_benchmarking(name: &str) -> String {
    format!(
        "
[dev-dependencies]
criterion = \"*\"

[profile.release]
debug = true

[[bench]]
name = \"{}\"
harness = false",
        name
    )
}

pub fn generate_delete_file(tmp_dir: &Path) -> Result<PathBuf> {
    let delete_file = tmp_dir.join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    Ok(delete_file)
}

pub fn start_shell(config: &Config, tmp_dir: &Path) -> Result<std::process::ExitStatus> {
    let mut shell_process = match config.editor {
        None => std::process::Command::new(get_shell()),
        Some(ref editor) => {
            let mut ide_process = std::process::Command::new(editor);
            ide_process
                .args(config.editor_args.iter().flatten())
                .arg(tmp_dir);
            ide_process
        }
    };

    if env::var("CARGO_TARGET_DIR").is_err() {
        if let Some(path) = &config.cargo_target_dir {
            env::set_var("CARGO_TARGET_DIR", path);
        }
    }

    let res = shell_process.current_dir(&tmp_dir).spawn();

    #[cfg(windows)]
    if config.editor.is_some() {
        unsafe {
            windows_sys::Win32::System::Console::FreeConsole();
        }
    }

    if let Ok(mut child) = res {
        child.wait().context("cannot wait shell process")
    } else {
        bail!("cannot spawn shell process")
    }
}

#[cfg(unix)]
type Child = std::process::Child;
#[cfg(windows)]
type Child = create_process_w::Child;

pub fn start_subprocesses(config: &Config, tmp_dir: &Path) -> Vec<Child> {
    config
        .subprocesses
        .iter()
        .filter_map(|x| x.spawn(tmp_dir))
        .collect::<Vec<Child>>()
}

pub fn clean_up(
    delete_file: &Path,
    tmp_dir: TempDir,
    worktree_branch: Option<&str>,
    subprocesses: &mut [Child],
) -> Result<()> {
    if !delete_file.exists() {
        log::info!(
            "Project directory preserved at: {}",
            tmp_dir.into_path().display()
        );
    } else if worktree_branch.is_some() {
        let mut command = std::process::Command::new("git");
        command
            .args(["worktree", "remove"])
            .arg(&tmp_dir.path())
            .arg("--force");
        ensure!(
            command.status().context("Could not start git")?.success(),
            "cannot remove working tree"
        );
    }

    kill_subprocesses(subprocesses)?;

    Ok(())
}

pub fn kill_subprocesses(subprocesses: &mut [Child]) -> Result<()> {
    #[cfg(unix)]
    {
        for subprocess in subprocesses.iter_mut() {
            {
                let now = std::time::Instant::now();

                unsafe {
                    libc::kill(
                        subprocess
                            .id()
                            .try_into()
                            .context("cannot get process id")?,
                        libc::SIGTERM,
                    );
                }

                while now.elapsed().as_secs() < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    if let Ok(Some(_)) = subprocess.try_wait() {
                        break;
                    }
                }
            }
        }
    }

    for subprocess in subprocesses.iter_mut() {
        match subprocess.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                let _ = subprocess.kill();
                let _ = subprocess.wait();
            }
        }
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
