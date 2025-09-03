use clap::{Args, Parser, Subcommand, command};
use serde::Serialize;

use crate::{
    unreal_engine::{error::UnrealError, unreal_project::UnrealProject},
    utility::search::SearchOptions,
};

mod unreal_engine;
mod utility;

#[derive(clap::Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
/// Main program options.
struct CliArgs {
    /// the command it will process.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
enum Command {
    Unreal(UnrealArgs),
    Clean(CleanArgs),
}

#[derive(Debug, Args, Serialize, Clone)]
struct CleanArgs {}

#[derive(Debug, Args, Serialize, Clone)]
#[command(version, about, long_about = None)]
struct UnrealArgs {
    #[command(subcommand)]
    command: UnrealCommand,

    #[command(flatten)]
    search_options: SearchOptions,
}

#[derive(Subcommand, Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum UnrealCommand {
    Build,
    BuildAndRun,
    Run,
}

fn process_unreal_command(args: UnrealArgs) -> Result<(), UnrealError> {
    let command = args.command;
    let options = args.search_options;

    let project = UnrealProject::from_cwd(options)?;

    match command {
        UnrealCommand::Build => project.build_project(),
        UnrealCommand::BuildAndRun => project.build_and_start(),
        UnrealCommand::Run => project.start_project(),
    }
}

fn process_clean_command(_: CleanArgs) -> Result<(), UnrealError> {
    println!("Clean isn't implemented yet.");
    Ok(())
}

fn main() -> Result<(), UnrealError> {
    let args = CliArgs::parse();

    match args.command {
        Command::Unreal(args) => process_unreal_command(args),
        Command::Clean(args) => process_clean_command(args),
    }
}
