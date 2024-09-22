use crate::dependency::{parse_dependency, Dependency};

/// This tool allow you to create a new Rust temporary project in a temporary
/// directory.
///
/// The dependencies can be provided in arguments (e.g.`cargo-temp anyhow
/// tokio`). When the shell is exited, the temporary directory is deleted unless
/// you removed the file `TO_DELETE`.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct Cli {
    /// Dependencies to add to `Cargo.toml`.
    ///
    /// The default version used is `*` but this can be replaced using `=`.
    /// E.g. `cargo-temp anyhow=1.0.13`
    #[arg(value_parser = parse_dependency)]
    pub dependencies: Vec<Dependency>,

    /// Create a library instead of a binary.
    #[arg(long, short = 'l')]
    pub lib: bool,

    /// Name of the temporary crate.
    #[arg(long = "name", short = 'n')]
    pub project_name: Option<String>,

    /// Create a temporary Git working tree based on the repository in the
    /// current directory.
    #[arg(long = "worktree", short = 'w')]
    pub worktree_branch: Option<Option<String>>,

    /// Create a temporary clone of a Git repository.
    #[arg(long, short = 'g')]
    pub git: Option<String>,

    /// Add a `benches` to the temporary project.
    ///
    /// You can choose the name of the benchmark file name as argument.
    /// The default is `benchmark.rs`
    #[arg(long, short = 'b')]
    pub bench: Option<Option<String>>,

    /// Select the Rust's edition of the temporary project.
    ///
    /// Available options are:
    /// * 15 | 2015 => edition 2015,
    /// * 18 | 2018 => edition 2018,
    /// * 21 | 2021 => edition 2021,
    ///
    /// If the argument doesn't match any of the options, the default is the latest edition
    #[arg(long, short = 'e')]
    pub edition: Option<u32>,

    #[cfg(feature = "generate")]
    #[command(subcommand)]
    pub subcommand: Option<generate::Subcommand>,
}

#[cfg(feature = "generate")]
pub mod generate {
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    #[derive(Clone, Debug, clap::Subcommand)]
    pub enum Subcommand {
        /// Generate a temporary project from a template using `cargo-generate`.
        Generate(Args),
    }

    #[derive(Clone, Debug, clap::Args)]
    pub struct Args {
        #[command(flatten)]
        pub template_path: cargo_generate::TemplatePath,
        /// List defined favorite templates from the config
        #[arg(
        long,
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
        #[arg(long, short)]
        pub name: Option<String>,
        /// Don't convert the project name to kebab-case before creating the directory.
        /// Note that cargo generate won't overwrite an existing directory, even if `--force` is given.
        #[arg(long, short)]
        pub force: bool,
        /// Enables more verbose output.
        #[arg(long, short)]
        pub verbose: bool,
        /// Pass template values through a file
        /// Values should be in the format `key=value`, one per line
        #[arg(long)]
        pub template_values_file: Option<String>,
        /// If silent mode is set all variables will be
        /// extracted from the template_values_file.
        /// If a value is missing the project generation will fail
        #[arg(long, short, requires("name"))]
        pub silent: bool,
        /// Use specific configuration file. Defaults to $CARGO_HOME/cargo-generate or $HOME/.cargo/cargo-generate
        #[arg(short, long)]
        pub config: Option<PathBuf>,
        /// Specify the VCS used to initialize the generated template.
        #[arg(long)]
        pub vcs: Option<cargo_generate::Vcs>,
        /// Populates template variable `crate_type` with value `"lib"`
        #[arg(long, conflicts_with = "bin")]
        pub lib: bool,
        /// Populates a template variable `crate_type` with value `"bin"`
        #[arg(long, conflicts_with = "lib")]
        pub bin: bool,
        /// Use a different ssh identity
        #[arg(short = 'i', long = "identity")]
        pub ssh_identity: Option<PathBuf>,
        /// Define a value for use during template expansion
        #[arg(long, short, number_of_values = 1)]
        pub define: Vec<String>,
        /// Use a different gitconfig file, if omitted the usual $HOME/.gitconfig will be used.
        #[arg(long = "gitconfig")]
        pub gitconfig: Option<PathBuf>,
        /// Will enforce a fresh git init on the generated project
        #[arg(long)]
        pub force_git_init: bool,
        /// Allow the template to overwrite existing files in the destination.
        #[arg(short, long)]
        pub overwrite: bool,
        /// Skip downloading git submodules (if there are any)
        #[arg(short, long)]
        pub skip_submodules: bool,
    }

    impl Args {
        pub fn generate(self, destination: &Path) -> Result<PathBuf> {
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
                gitconfig: self.gitconfig,
                define: self.define,
                force_git_init: self.force_git_init,
                overwrite: self.overwrite,
                skip_submodules: self.skip_submodules,
                // Not available for the users.
                destination: Some(destination.to_path_buf()),
                init: false,
                allow_commands: false,
                other_args: None,
            })
        }
    }
}
