//! Utility functions for working with paths
use std::path::{Path, PathBuf};
/// On windows removes the `\\?\\` prefix to UNC paths.
/// For other OS'es just turns the `Path` into a `PathBuf`
#[must_use]
pub fn fix_unc_path(res: &Path) -> PathBuf {
    if cfg!(windows) {
        let res_str = res.display().to_string();
        if res_str.starts_with(r#"\\?"#) {
            PathBuf::from(res_str.replace(r#"\\?\"#, ""))
        } else {
            res.to_path_buf()
        }
    } else {
        res.to_path_buf()
    }
}
