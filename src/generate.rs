#![cfg(feature = "generate")]

use anyhow::{ensure, Result};
use cargo_generate::{GenerateArgs, TemplatePath};
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
    pub template_path: TemplatePath,

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
    #[clap(long, short, requires("name"))]
    pub silent: bool,

    /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
    #[clap(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Specify the VCS used to initialize the generated template.
    #[clap(long, default_value = "git")]
    pub vcs: cargo_generate::Vcs,

    /// Populates a template variable `crate_type` with value `"lib"`
    #[clap(long, conflicts_with = "bin")]
    pub lib: bool,

    /// Populates a template variable `crate_type` with value `"bin"`
    #[clap(long, conflicts_with = "lib")]
    pub bin: bool,

    /// Use a different ssh identity
    #[clap(short = 'i', long = "identity", parse(from_os_str))]
    pub ssh_identity: Option<PathBuf>,

    /// Define a value for use during template expansion
    #[clap(long, short, number_of_values = 1)]
    pub define: Vec<String>,

    /// Will enforce a fresh git init on the generated project
    #[clap(long)]
    pub force_git_init: bool,
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
        cargo_generate::generate(GenerateArgs {
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
            // Ignore the `--init` flag since we are using a temporary directory.
            init: false,
            destination: Some(destination.to_path_buf()),
            force_git_init: self.force_git_init,
            // todo: add `allow_commands`.
            allow_commands: false,
        })
    }
}

fn clean_up(delete_file: &Path, project_dir: &Path, subprocesses: &mut [Child]) -> Result<()> {
    if !delete_file.exists() {
        println!("Project directory preserved at: {}", project_dir.display());
    } else {
        fs::remove_dir_all(project_dir)?
    }

    kill_subprocesses(subprocesses);

    Ok(())
}