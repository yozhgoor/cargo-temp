use anyhow::{bail, ensure, Context, Result};
use std::io::{stdin, Write};
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

    log::info!("Temporary project created at: {}", tmp_dir.path().display());

    if config.welcome_message {
        println!(
            "\nTo preserve the project when exiting the shell, don't forget to delete the \
            `TO_DELETE` file.\nTo exit the project, you can type \"exit\" or use `Ctrl+D`"
        );
    }

    let res = start_shell(&config, tmp_dir.path());

    clean_up(
        &delete_file,
        tmp_dir,
        cli.worktree_branch.flatten().as_deref(),
        cli.project_name.as_deref(),
        config.preserved_project_dir.as_deref(),
        &mut subprocesses,
        config.prompt,
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
        command.arg("clone").arg(url).arg(tmp_dir.as_ref());

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

    let res = shell_process.current_dir(tmp_dir).spawn();

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
    project_name: Option<&str>,
    preserved_project_dir: Option<&Path>,
    subprocesses: &mut [Child],
    prompt: bool,
) -> Result<()> {
    let delete = if !delete_file.exists() {
        false
    } else if prompt {
        println!("Are you sure you want to delete this project? (Y/n)");

        let mut input = String::new();

        loop {
            match stdin().read_line(&mut input) {
                Ok(_n) => match input.trim() {
                    "" | "Yes" | "yes" | "Y" | "y" => {
                        break true;
                    }
                    "No" | "no" | "N" | "n" => {
                        break false;
                    }
                    _ => println!("hmm, `{}` doesn't look like `yes` or `no`", input.trim()),
                },
                Err(err) => {
                    log::error!("failed to read input: {}", err);
                }
            }

            input.clear()
        }
    } else {
        true
    };

    if delete {
        if worktree_branch.is_some() {
            let mut command = std::process::Command::new("git");
            command
                .args(["worktree", "remove"])
                .arg(tmp_dir.path())
                .arg("--force");
            ensure!(
                command.status().context("Could not start git")?.success(),
                "cannot remove working tree"
            );
        }
    } else {
        let _ = fs::remove_file(delete_file);
        let tmp_dir = preserve_dir(tmp_dir, project_name, preserved_project_dir)?;

        log::info!("Project directory preserved at: {}", tmp_dir.display());
    }

    kill_subprocesses(subprocesses)?;

    Ok(())
}

// preserve dir by renaming to appropriate name if project_name is set
// and moves the project dir to preserved_project_dir as defined by the user
// it returns the dir where the project is preserved
pub fn preserve_dir(
    tmp_dir: TempDir,
    project_name: Option<&str>,
    preserved_project_dir: Option<&Path>,
) -> Result<PathBuf> {
    let tmp_dir = tmp_dir.into_path();

    let mut final_dir = if let Some(preserved_project_dir) = preserved_project_dir {
        if !preserved_project_dir.exists() {
            fs::create_dir_all(preserved_project_dir)
                .context("cannot create preserve project's directory")?;
        }

        preserved_project_dir.join(
            tmp_dir
                .file_name()
                .context("cannot create preserve project's directory")?,
        )
    } else {
        tmp_dir.clone()
    };

    if let Some(name) = project_name {
        final_dir = final_dir.with_file_name(name);
    }

    if final_dir != tmp_dir {
        fs::rename(&tmp_dir, &final_dir)?;
    };

    Ok(final_dir)
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
