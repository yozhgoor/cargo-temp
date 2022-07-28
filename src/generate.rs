#![cfg(feature = "generate")]

use anyhow::{ensure, Result};
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Child,
};

use crate::{generate_delete_file, kill_subprocesses, start_shell, start_subprocesses, Config};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(flatten)]
    pub template_path: cargo_generate::TemplatePath,
    /// List defined favorite templates from the config
    #[clap(
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
            "template-values-file",
            "ssh-identity",
            "test",
        ])
    )]
    pub list_favorites: bool,
    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[clap(long, short, value_parser)]
    pub name: Option<String>,
    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[clap(long, short, action)]
    pub force: bool,
    /// Enables more verbose output.
    #[clap(long, short, action)]
    pub verbose: bool,
    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[clap(long, value_parser)]
    pub template_values_file: Option<String>,
    /// If silent mode is set all variables will be
    /// extracted from the template_values_file.
    /// If a value is missing the project generation will fail
    #[clap(long, short, requires("name"), action)]
    pub silent: bool,
    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[clap(short, long, value_parser)]
    pub config: Option<PathBuf>,
    /// Specify the VCS used to initialize the generated template.
    #[clap(long, value_parser)]
    pub vcs: Option<cargo_generate::Vcs>,
    /// Populates template variable `crate_type` with value `"lib"`
    #[clap(long, conflicts_with = "bin", action)]
    pub lib: bool,
    /// Populates a template variable `crate_type` with value `"bin"`
    #[clap(long, conflicts_with = "lib", action)]
    pub bin: bool,
    /// Use a different ssh identity
    #[clap(short = 'i', long = "identity", value_parser)]
    pub ssh_identity: Option<PathBuf>,
    /// Define a value for use during template expansion
    #[clap(long, short, number_of_values = 1, value_parser)]
    pub define: Vec<String>,
    /// Generate the template directly at the given path.
    #[clap(long, value_parser)]
    pub destination: Option<PathBuf>,
    /// Will enforce a fresh git init on the generated project
    #[clap(long, action)]
    pub force_git_init: bool,
    /// Allow the template to overwrite existing files in the destination.
    #[clap(short, long, action)]
    pub overwrite: bool,
    /*

    Arguments from cargo-generate that will not be used by cargo-temp:

    /// Generate the template directly into the current dir. No subfolder will be created and no vcs is initialized.
    #[clap(long, action)]
    pub init: bool,
    /// Allows running system commands without being prompted.
    /// Warning: Setting this flag will enable the template to run arbitrary system commands without user confirmation.
    /// Use at your own risk and be sure to review the template code beforehand.
    #[clap(short, long, action)]
    pub allow_commands: bool,
    /// All args after "--" on the command line.
    #[clap(skip)]
    pub other_args: Option<Vec<String>>,
    */
}

impl Args {
    pub fn generate(self, config: Config) -> Result<()> {
        let project_dir = self.cargo_generate(&config.temporary_project_dir)?;

        let delete_file = generate_delete_file(&project_dir)?;

        let mut subprocesses = start_subprocesses(&config, &project_dir);

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
            destination: Some(destination.to_path_buf()),
            force_git_init: self.force_git_init,
            overwrite: self.overwrite,
            // Not used by cargo-temp.
            init: false,
            allow_commands: false,
            other_args: None,
        })
    }
}

fn clean_up(delete_file: &Path, project_dir: &Path, subprocesses: &mut [Child]) -> Result<()> {
    if !delete_file.exists() {
        println!("Project directory preserved at: {}", project_dir.display());
    } else {
        fs::remove_dir_all(project_dir)?
    }

    kill_subprocesses(subprocesses)?;

    Ok(())
}
