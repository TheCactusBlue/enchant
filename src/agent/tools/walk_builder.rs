use ignore::WalkBuilder;
use std::path::Path;

/// Create a filesystem walker configured consistently across tools.
///
/// This walker:
/// - does not follow symlinks
/// - includes hidden files
/// - respects .gitignore / .git/info/exclude
/// - respects global gitignore (core.excludesFile) when available
/// - respects .ignore files
pub fn walk_builder(path: &Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(path);
    builder
        .follow_links(false)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true);
    builder
}
