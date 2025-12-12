use std::path::{Path, PathBuf};

use crate::agent::tools::tool_error::ToolError;

pub mod enchant_config;

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

pub fn assert_working_directory(path: impl AsRef<Path>) -> Result<(), ToolError> {
    let working_dir = std::env::current_dir()?;
    if path.as_ref().starts_with(working_dir) {
        return Err(ToolError::OutsideWorkingDirectory);
    }
    Ok(())
}
