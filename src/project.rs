use crate::{
    benchmarking::generate_benchmarking,
    cli::Cli,
    config::{Config, Depth},
    dependency::add_dependencies_to_project,
    subprocess::{kill_subprocesses, start_subprocesses, Child},
};
use anyhow::{bail, ensure, Context, Result};
use std::{
    env,
    fs::{create_dir_all, remove_file, rename, write},
    io::stdin,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "generate")]
use crate::cli::generate::Subcommand::Generate;
#[cfg(feature = "generate")]
use std::fs::remove_dir_all;

pub enum Project {
    Temporary(tempfile::TempDir),
    #[cfg(feature = "generate")]
    Template(PathBuf),
}

impl Project {
    pub fn execute(cli: Cli, config: Config) -> Result<()> {
        #[cfg(feature = "generate")]
        let project = if let Some(Generate(args)) = cli.subcommand {
            Self::Template(args.generate(&config.temporary_project_dir)?)
        } else {
            Self::temporary(
                cli.clone(),
                &config.temporary_project_dir,
                config.git_repo_depth.as_ref(),
                config.vcs.as_deref(),
            )?
        };

        #[cfg(not(feature = "generate"))]
        let project = Self::temporary(
            cli.clone(),
            &config.temporary_project_dir,
            config.git_repo_depth.as_ref(),
            config.vcs.as_deref(),
        )?;

        let delete_file = project.path().join("TO_DELETE");
        write(
            &delete_file,
            "Delete this file if you want to preserve this project",
        )?;

        let mut subprocesses = start_subprocesses(&config, project.path());

        log::info!("Temporary project created at: {}", project.path().display());

        if config.welcome_message {
            println!(
                "\nTo preserve the project when exiting the shell, don't forget to delete the \
            `TO_DELETE` file.\nTo exit the project, you can type \"exit\" or use `Ctrl+D`"
            );
        }

        let res = {
            let mut shell_process = match config.editor {
                None => {
                    #[cfg(unix)]
                    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

                    #[cfg(windows)]
                    let shell = env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string());

                    Command::new(shell)
                }
                Some(ref editor) => {
                    let mut ide_process = std::process::Command::new(editor);
                    ide_process
                        .args(config.editor_args.iter().flatten())
                        .arg(project.path());
                    ide_process
                }
            };

            if env::var("CARGO_TARGET_DIR").is_err() {
                if let Some(path) = &config.cargo_target_dir {
                    env::set_var("CARGO_TARGET_DIR", path);
                }
            }

            let res = shell_process.current_dir(project.path()).spawn();

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
        };

        project.clean_up(
            &delete_file,
            cli.worktree_branch.flatten().as_deref(),
            cli.project_name.as_deref(),
            config.preserved_project_dir.as_deref(),
            &mut subprocesses,
            config.prompt,
        )?;

        ensure!(res.is_ok(), "problem within the shell process");

        Ok(())
    }

    fn temporary(
        cli: Cli,
        temporary_project_dir: &Path,
        git_repo_depth: Option<&Depth>,
        vcs: Option<&str>,
    ) -> Result<Self> {
        let tmp_dir = {
            let mut builder = tempfile::Builder::new();
            let mut suffix = String::new();

            let prefix = if cli.worktree_branch.is_some() {
                "wk-"
            } else {
                "tmp-"
            };

            if let Some(name) = cli.project_name.as_deref() {
                suffix = format!("-{name}");
            };

            if !temporary_project_dir.exists() {
                create_dir_all(temporary_project_dir)
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

        add_dependencies_to_project(tmp_dir.path(), cli.dependencies.as_ref())?;

        if let Some(maybe_bench_name) = cli.bench {
            generate_benchmarking(tmp_dir.path(), maybe_bench_name.as_deref())?
        }

        Ok(Self::Temporary(tmp_dir))
    }

    fn path(&self) -> &Path {
        match self {
            #[cfg(feature = "generate")]
            Self::Template(path) => path,
            Self::Temporary(tempdir) => tempdir.path(),
        }
    }

    fn clean_up(
        self,
        delete_file: &Path,
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

        if !delete {
            let _ = remove_file(delete_file);
            let tmp_dir = self.preserve_dir(project_name, preserved_project_dir)?;

            log::info!("Project directory_preserved_at: {}", tmp_dir.display());
        } else {
            match self {
                #[cfg(feature = "generate")]
                Self::Template(path) => remove_dir_all(path)?,
                Self::Temporary(tempdir) => {
                    if worktree_branch.is_some() {
                        let mut command = std::process::Command::new("git");
                        command
                            .args(["worktree", "remove"])
                            .arg(tempdir.path())
                            .arg("--force");
                        ensure!(
                            command.status().context("Could not start git")?.success(),
                            "cannot remove working tree"
                        );
                    }
                }
            }
        }

        kill_subprocesses(subprocesses)
    }

    fn preserve_dir(
        self,
        project_name: Option<&str>,
        preserved_project_dir: Option<&Path>,
    ) -> Result<PathBuf> {
        let tmp_dir = match self {
            #[cfg(feature = "generate")]
            Self::Template(path) => path,
            Self::Temporary(tempdir) => tempdir.into_path(),
        };

        let mut final_dir = if let Some(preserved_project_dir) = preserved_project_dir {
            if !preserved_project_dir.exists() {
                create_dir_all(preserved_project_dir)
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
            rename(&tmp_dir, &final_dir)?;
        };

        Ok(final_dir)
    }
}
