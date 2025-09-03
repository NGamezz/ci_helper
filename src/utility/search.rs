use std::{
    io,
    path::{Path, PathBuf},
};

use clap::{Parser, command};
use jwalk::WalkDir;
use serde::Serialize;

#[derive(Parser, Clone, Debug, Serialize)]
#[command(version, about, long_about = None)]
/// options used for directory searches.
pub struct SearchOptions {
    /// the maximum amount of search depth
    #[arg(short = 'd', long = "depth", default_value_t = 4)]
    pub max_depth: usize,

    /// the unreal engine version to target.
    #[arg(short = 'v', long = "version", default_value_t = 5.6)]
    pub ue_version: f32,

    #[arg(short = 't', long = "directory", default_value = "default_dir()")]
    /// the directory to use as the search root.
    pub search_dir: Vec<String>,
}

#[cfg(target_os = "windows")]
fn default_dir() -> Vec<String> {
    let drives = unsafe {
        use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
        GetLogicalDrives()
    };

    ('A'..='Z')
        .enumerate()
        .filter_map(|(i, c)| {
            let mask = 1u32 << i;

            if drives & mask != 0 {
                return Some(format!("{c}:/"));
            }

            None
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn default_dir() -> Vec<String> {
    vec![String::from("/")]
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_depth: 6,
            ue_version: 5.6,
            search_dir: default_dir(),
        }
    }
}

pub fn get_ue_dir(options: SearchOptions) -> Option<PathBuf> {
    let ue_string = format!("UE_{}", options.ue_version);
    let ue_pattern = ue_string.as_str();

    let search_dirs = options.search_dir;
    println!("Search Directories : {search_dirs:?}");

    search_dirs.into_iter().find_map(|dir| {
        WalkDir::new(dir)
            .max_depth(options.max_depth)
            .into_iter()
            .find_map(|result| {
                let entry = result.ok()?;
                let name = entry.file_name().to_str()?;

                match name == ue_pattern {
                    true => Some(entry.path()),
                    false => None,
                }
            })
    })
}

pub fn file_with_extension(search_dir: impl AsRef<Path>, target_ext: &str) -> io::Result<PathBuf> {
    let entries = std::fs::read_dir(search_dir)?;

    entries
        .into_iter()
        .find_map(|result| {
            let entry = result.ok()?;
            let path = entry.path();

            let extension = path.extension()?.to_str()?;

            if extension == target_ext {
                return Some(path);
            }

            None
        })
        .ok_or(io::Error::other("File not Found."))
}

pub mod test {
    use super::*;

    #[test]
    pub fn get_ue_dir_test() {
        let options = SearchOptions::default();

        assert!(get_ue_dir(options).is_some());
    }
}
