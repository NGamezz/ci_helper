use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Package Error : {0}")]
    Package(#[from] PackageError),

    #[error("Io Error : {0}")]
    Io(#[from] std::io::Error),

    #[error("Reqwest Error")]
    ReqwestError,

    #[error("Zip Error : {0}")]
    ZipError(#[from] zip::result::ZipError),
}

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Io Error : {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to Parse the Package.")]
    Parsing(#[from] toml::de::Error),
}
