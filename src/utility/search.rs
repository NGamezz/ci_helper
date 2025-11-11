use std::path::{Path, PathBuf};

use clap::{Parser, command};
use serde::Serialize;

#[derive(Parser, Clone, Debug, Serialize)]
#[command(version, about, long_about = None)]
/// options used for directory searches.
pub struct SearchOptions {
    /// the maximum amount of search depth
    #[arg(short = 'd', long = "depth", default_value_t = 4)]
    pub max_depth: usize,

    #[arg(short = 'p', long = "cwd", default_value_t = std::env::current_dir().unwrap().to_string_lossy().to_string())]
    /// the directory of the uproject file, without the filename. Ex : C:\\ProjectName
    pub project_directory: String,

    #[arg(short = None, long = "major", default_value_t = 5)]
    /// the unreal engine major version to target.
    pub ue_major_version: u16,

    #[arg(short = None, long = "minor", default_value_t = 6)]
    /// the unreal engine minor version to target.
    pub ue_minor_version: u16,

    #[arg(short = None, long = "patch", default_value_t = 1)]
    /// the unreal engine patch version to target.
    pub ue_patch_version: u16,

    #[arg(short = 's', long = "directory", num_args=1.., value_delimiter=' ', default_value = default_dir())]
    /// the directory to use as the UnrealEngine search root.
    pub search_dir: Vec<String>,
}

#[cfg(target_os = "windows")]
fn default_dir() -> String {
    let drives = unsafe {
        use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
        GetLogicalDrives()
    };

    ('A'..='Z')
        .enumerate()
        .filter_map(move |(i, c)| {
            let mask = 1u32 << i;

            if drives & mask != 0 {
                return Some(c);
            }

            None
        })
        .fold(String::new(), |mut res, input| {
            use std::fmt::Write;

            write!(&mut res, " {}:\\", &input).unwrap();
            res
        })
}

#[cfg(not(target_os = "windows"))]
fn default_dir() -> String {
    String::from("/")
}

pub fn file_with_extension(search_dir: impl AsRef<Path>, target_ext: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(search_dir).ok()?;

    entries.into_iter().find_map(|result| {
        let entry = result.ok()?;
        let path = entry.path();

        let extension = path.extension()?.to_str()?;

        if extension == target_ext {
            return Some(path);
        }

        None
    })
}
