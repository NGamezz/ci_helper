use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanError {
    #[error("Ignore file not found.")]
    IgnoreFileNotFound,

    #[error("{error}")]
    IoError {
        #[from]
        error: io::Error,
    },
}
