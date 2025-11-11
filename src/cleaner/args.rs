use clap::Args;
use serde::Serialize;

#[derive(Debug, Args, Serialize, Clone)]
pub struct CleanArgs {
    #[command(flatten)]
    pub ignore_settings: IgnoreOptions,
}

#[derive(clap::Args, Debug, Serialize, Clone)]
#[group(required = true, multiple = false)]
pub struct IgnoreOptions {
    #[arg(short = 'i', long)]
    pub ignore_path: String,
}
