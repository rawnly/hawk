
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct CopyFlags {
    /// Watch for changes
    #[clap(short, long, value_parser, default_value_t = false)]
    pub watch: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct InitFlags {
    /// Use json instead of yaml
    #[clap(short, long, value_parser, default_value_t = false)]
    pub json: bool,

    /// Generate from local `pnpm-workspace` or package.json `workspaces` key.
    #[clap(short, long, value_parser, default_value_t = false)]
    pub read_env: bool,

    /// Choose the destination folder
    #[clap(long, value_parser)]
    pub target: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Delete generated files
    Clean,

    /// Initialize a repository.
    Init(InitFlags),

    /// Copy files to the `target` directory (specified in the config)
    Copy(CopyFlags),
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Specify the config file path
    #[clap(short, long, value_parser, value_hint = clap::ValueHint::FilePath)]
    pub config: Option<String>,

    #[clap(subcommand)]
    pub action: Option<Action>,
}
