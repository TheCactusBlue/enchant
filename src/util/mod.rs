use std::path::{Path, PathBuf};

pub fn format_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let working_dir = if let Ok(working_dir) = std::env::current_dir() {
        working_dir
    } else {
        return path.to_path_buf();
    };

    if path.starts_with(&working_dir) {
        return path
            .strip_prefix(&working_dir)
            .unwrap_or(&path)
            .to_path_buf();
    }
    return path.to_path_buf();
}
