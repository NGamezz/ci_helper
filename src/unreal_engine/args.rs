use clap::Args;
use serde::Serialize;

use crate::{unreal_engine::command::UnrealCommand, utility::search::SearchOptions};

#[derive(Debug, Args, Serialize, Clone)]
#[command(version, about, long_about = None)]
pub struct UnrealArgs {
    #[command(subcommand)]
    pub command: UnrealCommand,

    #[command(flatten)]
    pub search_options: SearchOptions,
}
