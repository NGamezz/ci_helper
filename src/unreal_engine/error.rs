use std::{io, process::ExitStatus};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnrealError {
    #[error("Failed to find Unreal Engine Path.")]
    EngineNotFound,

    #[error("Failed to find Unreal Engine Project.")]
    ProjectNotFound,

    #[error("Failed exit status : {status}")]
    FailedExitStatus { status: ExitStatus },

    #[error("{error}")]
    IoError {
        #[from]
        error: io::Error,
    },
}
