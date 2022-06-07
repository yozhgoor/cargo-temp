#![cfg(feature = "generate")]

use anyhow::{ensure, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

use crate::{clean_up, generate_delete_file, start_shell, start_subprocesses, Config};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// List defined favorite templates from the config
    #[clap(
        long,
        conflicts_with = "git",
        conflicts_with = "subfolder",
        conflicts_with = "path",
        conflicts_with = "branch",
        conflicts_with = "name",
        conflicts_with = "force",
        conflicts_with = "template-values-file",
        conflicts_with = "silent",
        conflicts_with = "vcs",
        conflicts_with = "lib",
        conflicts_with = "bin",
        conflicts_with = "ssh-identity",
        conflicts_with = "define",
        conflicts_with = "init"
    )]
    pub list_favorites: bool,

    /// Generate a favorite template as defined in the config. In case the favorite is undefined,
    /// use in place of the `--git` option, otherwise specifies the subfolder
    #[clap(name = "favorite")]
    pub favorite: Option<String>,

    /// Specifies a subfolder within the template repository to be used as the actual template.
    #[clap(name = "subfolder")]
    pub subfolder: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[clap(name = "git", short, long, conflicts_with = "subfolder")]
    pub git: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[clap(
        short,
        long,
        conflicts_with = "git",
        conflicts_with = "favorite",
        conflicts_with = "subfolder"
    )]
    pub path: Option<PathBuf>,

    /// Branch to use when installing from git
    #[clap(short, long)]
    pub branch: Option<String>,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    /// to kebab-case unless `--force` is given.
    #[clap(long, short)]
    pub name: Option<String>,

    /// Don't convert the project name to kebab-case before creating the directory.
    /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
    #[clap(long, short)]
    pub force: bool,

    /// Enables more verbose output.
    #[clap(long, short)]
    pub verbose: bool,

    /// Pass template values through a file
    /// Values should be in the format `key=value`, one per line
    #[clap(long)]
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

        let args = self.convert(tmp_dir.path());

        cargo_generate::generate(args)?;

        let delete_file = generate_delete_file(tmp_dir.path())?;

        let mut subprocesses = start_subprocesses(&config, tmp_dir.path());

        let res = start_shell(&config, tmp_dir.path());

        clean_up(&delete_file, tmp_dir, None, &mut subprocesses)?;

        ensure!(res.is_ok(), "problem within the shell process");

        Ok(())
    }

    fn convert(self, destination_path: &Path) -> cargo_generate::Args {
        cargo_generate::Args {
            list_favorites: self.list_favorites,
            favorite: self.favorite,
            subfolder: self.subfolder,
            git: self.git,
            path: self.path,
            branch: self.branch,
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
            // Ignore `--init` since we create a temporary directory.
            init: false,
            // Force the destination path for the template to be the temporary directory.
            destination: Some(destination_path.to_path_buf()),
            force_git_init: self.force_git_init,
            // todo: add `allow_commands`.
            allow_commands: false,
        }
    }
}
