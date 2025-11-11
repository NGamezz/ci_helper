use clap::Subcommand;
use serde::Serialize;

use crate::unreal_engine::{args::UnrealArgs, error::UnrealError, unreal_project::UnrealProject};

#[derive(Subcommand, Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnrealCommand {
    Build,
    BuildAndRun,
    Run,
}

pub fn process_unreal_command(args: UnrealArgs) -> Result<(), UnrealError> {
    let command = args.command;
    let options = args.search_options;

    let project = UnrealProject::try_from(options)?;

    match command {
        UnrealCommand::Build => project.build_project(),
        UnrealCommand::BuildAndRun => project.build_and_start(),
        UnrealCommand::Run => project.start_project(),
    }
}
