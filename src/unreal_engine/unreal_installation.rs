use std::path::PathBuf;

use jwalk::WalkDir;
use serde::{Deserialize, Serialize};

use crate::{
    unreal_engine::error::UnrealError,
    utility::search::{self, SearchOptions},
};

#[derive(Serialize, Deserialize)]
pub struct UnrealInstallation {
    pub version: f32,
    pub exe_path: String,
    pub base_path: String,
}

impl UnrealInstallation {
    pub fn find_installation(
        version: f32,
        search_options: SearchOptions,
    ) -> Result<UnrealInstallation, UnrealError> {
        let base_path = search::get_ue_dir(search_options).ok_or(UnrealError::EngineNotFound)?;

        let base_path_string = base_path.to_string_lossy().to_string();

        let installation = UnrealInstallation {
            exe_path: format!("{base_path_string}\\Engine\\Binaries\\Win64\\UnrealEditor.exe"),
            base_path: base_path_string,
            version,
        };

        Ok(installation)
    }

    pub fn find_file(&self, target_name: impl AsRef<str>) -> Option<PathBuf> {
        WalkDir::new(&self.base_path).into_iter().find_map(|entry| {
            let entry = entry.ok()?;

            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let name = path.file_name()?;

            if name.eq(target_name.as_ref()) {
                return Some(path);
            }

            None
        })
    }
}
