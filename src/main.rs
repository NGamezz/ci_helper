use clap::{Parser, Subcommand, command};
use serde::Serialize;
use thiserror::Error;

use crate::{
    cleaner::{args::CleanArgs, error::CleanError},
    packages::{args::PackagesArgs, error::SetupError},
    unreal_engine::{args::UnrealArgs, error::UnrealError},
};

mod cleaner;
mod packages;
mod unreal_engine;
mod utility;

#[derive(Error, Debug)]
enum Error {
    #[error("Unreal Error : {0}")]
    Unreal(#[from] UnrealError),

    #[error("Clean Error : {0}")]
    Clean(#[from] CleanError),

    #[error("Setup Error : {0}")]
    Setup(#[from] SetupError),
}

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
    Setup(PackagesArgs),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = CliArgs::parse();

    match args.command {
        Command::Unreal(args) => unreal_engine::command::process_unreal_command(args)?,
        Command::Clean(args) => cleaner::command::process_clean_command(args).await?,
        Command::Setup(args) => packages::command::setup(args).await?,
    };

    Ok(())
}
