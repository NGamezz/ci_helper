use std::{collections::HashMap, path::Path};

use serde::Deserialize;

use crate::packages::error::PackageError;

#[derive(Debug, Deserialize)]
pub enum SourceType {
    Git,
    Http,
}

#[derive(Debug, Deserialize)]
pub struct SourceInfo {
    pub source_type: SourceType,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    // pub source: Source,
    pub target_dir: String,
    pub platform: HashMap<String, SourceInfo>,
}

#[derive(Debug, Deserialize)]
pub struct PackageRegistry {
    pub packages: HashMap<String, Package>,
}

impl PackageRegistry {
    pub async fn from_file(path: impl AsRef<Path>) -> Result<PackageRegistry, PackageError> {
        let path = path.as_ref();

        let contents = tokio::fs::read(path).await?;
        let package: Self = toml::from_slice(&contents)?;

        Ok(package)
    }
}
