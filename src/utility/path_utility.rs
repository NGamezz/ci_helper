use std::path::{Path, PathBuf};

pub fn remove_extension(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().with_extension("")
}

pub fn filename_as_string(path: impl AsRef<Path>) -> Option<String> {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_owned())
}
