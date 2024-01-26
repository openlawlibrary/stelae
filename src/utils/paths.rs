//! Utility functions for working with paths
use std::path::{Path, PathBuf};
/// On Windows removes the `\\?\\` prefix to UNC paths.
/// For other OS'es just turns the `Path` into a `PathBuf`
#[must_use]
pub fn fix_unc_path(absolute_path: &Path) -> PathBuf {
    if cfg!(windows) {
        let absolute_path_str = absolute_path.display().to_string();
        if absolute_path_str.starts_with(r#"\\?"#) {
            return PathBuf::from(absolute_path_str.replace(r#"\\?\"#, ""));
        }
    }
    absolute_path.to_path_buf()
}
