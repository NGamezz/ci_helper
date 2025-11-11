use clap::Args;
use serde::Serialize;

#[derive(Args, Serialize, Clone, Debug)]
pub struct PackagesArgs {
    #[arg(short = 'c', long)]
    /// the file that stores package information.
    pub config_path: String,
}
