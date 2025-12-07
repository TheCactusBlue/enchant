use std::path::PathBuf;

pub fn format_path<P: Into<PathBuf>>(path: P) -> PathBuf {
    let path: PathBuf = path.into();
    let working_dir = if let Ok(working_dir) = std::env::current_dir() {
        working_dir
    } else {
        return path;
    };

    if path.starts_with(&working_dir) {
        return path
            .strip_prefix(&working_dir)
            .unwrap_or(&path)
            .to_path_buf();
    }
    return path;
}
