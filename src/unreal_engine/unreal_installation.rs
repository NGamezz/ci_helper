use std::path::{Path, PathBuf};

use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};

use crate::{unreal_engine::error::UnrealError, utility::search::SearchOptions};

#[derive(Serialize, Deserialize, Debug)]
pub struct UnrealInstallation {
    pub exe_path: String,
    pub base_path: String,
    pub version: UnrealVersion,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UnrealVersion {
    pub major_version: u16,
    pub minor_version: u16,
    pub patch_version: u16,
}

impl TryFrom<SearchOptions> for UnrealInstallation {
    type Error = UnrealError;

    fn try_from(value: SearchOptions) -> Result<Self, Self::Error> {
        Self::get_installation(value).ok_or(UnrealError::EngineNotFound)
    }
}

impl UnrealInstallation {
    fn get_installation(search_options: SearchOptions) -> Option<UnrealInstallation> {
        let (tx, rx) = std::sync::mpsc::channel::<UnrealInstallation>();
        let num_cpus = num_cpus::get();

        let found = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        for root in search_options.search_dir {
            if !Path::new(&root).exists() {
                continue;
            }

            let tx = tx.clone();
            let cl = found.clone();

            let walker = WalkBuilder::new(&root)
                .max_depth(Some(search_options.max_depth))
                .threads(num_cpus)
                .follow_links(false)
                .build_parallel();

            walker.run(move || {
                let tx = tx.clone();
                let found = cl.clone();

                Box::new(move |result: Result<ignore::DirEntry, ignore::Error>| {
                    if found.load(std::sync::atomic::Ordering::Relaxed) {
                        return ignore::WalkState::Quit;
                    }

                    let entry = match result {
                        Ok(entry) => entry,
                        Err(_) => return ignore::WalkState::Continue,
                    };

                    let path = entry.path();

                    if !path.is_dir() {
                        return ignore::WalkState::Continue;
                    }

                    let name = match path.file_name().and_then(|n| n.to_str()) {
                        Some(name) => name,
                        None => return ignore::WalkState::Continue,
                    };

                    if name != "Engine" {
                        return ignore::WalkState::Continue;
                    }

                    let installation = Self::from_path(
                        path,
                        search_options.ue_major_version,
                        search_options.ue_minor_version,
                    );

                    if let Some(installation) = installation {
                        found.store(true, std::sync::atomic::Ordering::Relaxed);
                        let _ = tx.send(installation);
                        return ignore::WalkState::Quit;
                    }

                    ignore::WalkState::Skip
                })
            });

            if found.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }

        drop(tx);
        rx.recv().ok()
    }

    // Helper function to validate and create UnrealInstallation
    fn from_path(
        engine_path: &std::path::Path,
        target_major: u16,
        target_minor: u16,
    ) -> Option<UnrealInstallation> {
        let base_path = engine_path.parent()?.to_path_buf();
        let build_version_path = engine_path.join("Build").join("Build.version");

        if !build_version_path.exists() {
            return None;
        }

        let exe_path = Self::find_executable(engine_path)?;

        let file = std::fs::File::open(&build_version_path).ok()?;
        let version: UnrealVersion = serde_json::from_reader(file).ok()?;

        if version.major_version == target_major && version.minor_version == target_minor {
            Some(UnrealInstallation {
                base_path: base_path.to_string_lossy().to_string(),
                exe_path: exe_path.to_string_lossy().to_string(),
                version,
            })
        } else {
            None
        }
    }

    fn find_executable(engine_path: &std::path::Path) -> Option<std::path::PathBuf> {
        let binaries_path = engine_path.join("Binaries");

        let candidates = [
            binaries_path.join("Win64").join("UnrealEditor.exe"),
            binaries_path.join("Mac").join("UnrealEditor"),
            binaries_path.join("Linux").join("UnrealEditor"),
        ];

        candidates.into_iter().find(|path| path.exists())
    }

    pub fn find_file(&self, target_name: impl AsRef<str>) -> Option<PathBuf> {
        Walk::new(&self.base_path).into_iter().find_map(|entry| {
            let entry = entry.ok()?;

            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let name = path.file_name()?;

            if name.eq(target_name.as_ref()) {
                return Some(path.to_path_buf());
            }

            None
        })
    }
}
