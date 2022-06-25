#![cfg(feature = "generate")]

use anyhow::{ensure, Result};
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::Parser;
use std::path::PathBuf;

use crate::{clean_up, generate_delete_file, start_shell, start_subprocesses, Config};

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
        let tmp_dir = tempfile::Builder::new()
            .prefix("tmp-")
            .tempdir_in(&config.temporary_project_dir)?;

        self.cargo_generate()?;

        let delete_file = generate_delete_file(tmp_dir.path())?;

        let mut subprocesses = start_subprocesses(&config, tmp_dir.path());

        let res = start_shell(&config, tmp_dir.path());

        clean_up(&delete_file, tmp_dir, None, &mut subprocesses)?;

        ensure!(res.is_ok(), "problem within the shell process");

        Ok(())
    }

    fn cargo_generate(self) -> Result<()> {
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
            // Use `init` to init the template as the created temporary directory.
            init: true,
            // Ignore `--destination` since we create a temporary directory.
            destination: None,
            force_git_init: self.force_git_init,
            // todo: add `allow_commands`.
            allow_commands: false,
        })
    }
}
