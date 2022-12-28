#![cfg(feature = "generate")]

use anyhow::{ensure, Result};
use clap::Args;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{generate_delete_file, kill_subprocesses, start_shell, start_subprocesses, Config};

#[derive(Clone, Debug, Args)]
pub struct GenerateArgs {
    #[clap(flatten)]
    pub template_path: cargo_generate::TemplatePath,
    /// List defined favorite templates from the config
    #[arg(
        long,
        action,
        conflicts_with_all(&[
            "git", "path", "subfolder", "branch",
            "name",
            "force",
            "silent",
            "vcs",
            "lib",
            "bin",
            "define",
            "template_values_file",
            "ssh_identity",
            "test",
        ])
    )]
    pub list_favorites: bool,
    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[arg(long, short, value_parser)]
    pub name: Option<String>,
    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[arg(long, short, action)]
    pub force: bool,
    /// Enables more verbose output.
    #[arg(long, short, action)]
    pub verbose: bool,
    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[arg(long, value_parser)]
    pub template_values_file: Option<String>,
    /// If silent mode is set all variables will be
    /// extracted from the template_values_file.
    /// If a value is missing the project generation will fail
    #[arg(long, short, requires("name"), action)]
    pub silent: bool,
    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[arg(short, long, value_parser)]
    pub config: Option<PathBuf>,
    /// Specify the VCS used to initialize the generated template.
    #[arg(long, value_parser)]
    pub vcs: Option<cargo_generate::Vcs>,
    /// Populates template variable `crate_type` with value `"lib"`
    #[arg(long, conflicts_with = "bin", action)]
    pub lib: bool,
    /// Populates a template variable `crate_type` with value `"bin"`
    #[arg(long, conflicts_with = "lib", action)]
    pub bin: bool,
    /// Use a different ssh identity
    #[arg(short = 'i', long = "identity", value_parser)]
    pub ssh_identity: Option<PathBuf>,
    /// Define a value for use during template expansion
    #[arg(long, short, number_of_values = 1, value_parser)]
    pub define: Vec<String>,
    /// Will enforce a fresh git init on the generated project
    #[arg(long, action)]
    pub force_git_init: bool,
    /// Allow the template to overwrite existing files in the destination.
    #[arg(short, long, action)]
    pub overwrite: bool,
}

impl GenerateArgs {
    pub fn generate(self, config: Config) -> Result<()> {
        let project_dir = self.cargo_generate(&config.temporary_project_dir)?;

        let delete_file = generate_delete_file(&project_dir)?;

        let mut subprocesses = start_subprocesses(&config, &project_dir);

        log::info!("Temporary project created at: {}", &project_dir.display());

        let res = start_shell(&config, &project_dir);

        clean_up(&delete_file, &project_dir, &mut subprocesses)?;

        ensure!(res.is_ok(), "problem within the shell process");

        Ok(())
    }

    fn cargo_generate(self, destination: &Path) -> Result<PathBuf> {
        cargo_generate::generate(cargo_generate::GenerateArgs {
            template_path: self.template_path,
            list_favorites: self.list_favorites,
            name: self.name,
            force: self.force,
            verbose: self.verbose,
            template_values_file: self.template_values_file,
            silent: self.silent,
            config: self.config,
            vcs: self.vcs,
            lib: self.lib,
            bin: self.bin,
            ssh_identity: self.ssh_identity,
            define: self.define,
            force_git_init: self.force_git_init,
            overwrite: self.overwrite,
            // Not available for the users.
            destination: Some(destination.to_path_buf()),
            init: false,
            allow_commands: false,
            other_args: None,
        })
    }
}

#[cfg(unix)]
type Child = std::process::Child;
#[cfg(windows)]
type Child = create_process_w::Child;

fn clean_up(delete_file: &Path, project_dir: &Path, subprocesses: &mut [Child]) -> Result<()> {
    if !delete_file.exists() {
        log::info!("Project directory preserved at: {}", project_dir.display());
    } else {
        fs::remove_dir_all(project_dir)?
    }

    kill_subprocesses(subprocesses)
}
