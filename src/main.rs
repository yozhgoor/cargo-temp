use anyhow::{ensure, Context, Result};
use std::io::Write;
use std::{env, fs, process};
use tempfile::Builder;

fn main() -> Result<()> {
    // Parse the command line input.
    let cli = cargo_temp::Cli::new();

    // Read configuration from disk or generate a default one.
    let config = cargo_temp::Config::get_or_create()?;
    let _ = fs::create_dir(&config.temporary_project_dir);

    // Create the temporary directory
    let tmp_dir = {
        let mut builder = Builder::new();

        if cli.worktree_branch.is_some() {
            builder.prefix("wk-");
        } else {
            builder.prefix("tmp-");
        }

        builder.tempdir_in(&config.temporary_project_dir)?
    };

    let project_name = cli.project_name.unwrap_or_else(|| {
        tmp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
    });

    // Generate the temporary project or temporary worktree
    if let Some(maybe_branch) = cli.worktree_branch.as_ref() {
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

        if cli.lib {
            command.arg("--lib");
        }

        ensure!(
            command.status().context("Could not start cargo")?.success(),
            "Cargo command failed"
        );

        // Add dependencies to Cargo.toml from arguments given by the user
        let mut toml = fs::OpenOptions::new()
            .append(true)
            .open(tmp_dir.path().join("Cargo.toml"))?;
        for dependency in cli.dependencies.iter() {
            match dependency {
                cargo_temp::Dependency::CrateIo(s, v) => match &v {
                    Some(version) => writeln!(toml, "{} = \"{}\"", s, version)?,
                    None => writeln!(toml, "{} = \"*\"", s)?,
                },
                cargo_temp::Dependency::Repository {
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
    }

    // Generate the `TO_DELETE` file
    let delete_file = tmp_dir.path().join("TO_DELETE");
    fs::write(
        &delete_file,
        "Delete this file if you want to preserve this project",
    )?;

    // Prepare a new shell or an editor if its set in the config file
    let mut shell_process = match config.editor {
        None => process::Command::new(cargo_temp::get_shell()),
        Some(ref editor) => {
            let mut ide_process = process::Command::new(editor);
            ide_process
                .args(config.editor_args.iter().flatten())
                .arg(tmp_dir.path());
            ide_process
        }
    };

    if env::var("CARGO_TARGET_DIR").is_err() {
        if let Some(path) = config.cargo_target_dir {
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

    if !delete_file.exists() {
        println!(
            "Project directory preserved at: {}",
            tmp_dir.into_path().display()
        );
    } else if cli.worktree_branch.is_some() {
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
