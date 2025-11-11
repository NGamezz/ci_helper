use std::{io::Write, path::Path};

use git2::Repository;
use tempfile::tempfile;

use crate::packages::{
    args::PackagesArgs,
    error::SetupError,
    package::{Package, PackageRegistry, SourceInfo, SourceType},
};

pub async fn setup_git_package(source: &SourceInfo, package: &Package) -> Result<(), SetupError> {
    let target_dir = Path::new(&package.target_dir);
    debug_assert!(!target_dir.exists());

    tokio::fs::create_dir(target_dir).await?;

    if let Err(err) = Repository::clone(&source.source, target_dir) {
        eprintln!("Error Cloning Repository : {}", err.message());
    }

    Ok(())
}

pub async fn setup_http_package(source: &SourceInfo, package: &Package) -> Result<(), SetupError> {
    let path = Path::new(&package.target_dir);
    debug_assert!(!path.exists());

    let response = match reqwest::get(&source.source).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("{err:?}");
            return Err(SetupError::ReqwestError);
        }
    };

    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("{err:?}");
            return Err(SetupError::ReqwestError);
        }
    };

    let mut file = tempfile()?;
    file.write_all(&bytes)?;

    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(path)?;

    Ok(())
}

pub async fn setup(args: PackagesArgs) -> Result<(), SetupError> {
    let path = Path::new(&args.config_path);

    let registry = PackageRegistry::from_file(path).await?;

    let packages = registry.packages;
    println!("Processing : {} packages..", packages.len());

    #[cfg(target_os = "windows")]
    const PLATFORM: &str = "windows";
    #[cfg(target_os = "linux")]
    const PLATFORM: &str = "linux";

    let mut set = tokio::task::JoinSet::new();

    for (name, package) in packages {
        set.spawn(async move {
            let path = Path::new(&package.target_dir);

            if path.exists() {
                println!("Package Already Present : {path:?}, Skipping..");
                return Ok(());
            }

            println!("Installing : {name}..");
            let source = &package.platform[PLATFORM];

            match source.source_type {
                SourceType::Git => setup_git_package(source, &package).await,
                SourceType::Http => setup_http_package(source, &package).await,
            }
        });
    }

    let results = set.join_all().await;

    for setup_error in results.into_iter().map_while(|r| r.err()) {
        println!("{setup_error:?}");
    }

    Ok(())
}
