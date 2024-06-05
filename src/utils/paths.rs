//! Utility functions for working with paths
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
/// On Windows removes the `\\?\\` prefix to UNC paths.
/// For other OS'es just turns the `Path` into a `PathBuf`
#[must_use]
pub fn fix_unc_path(absolute_path: &Path) -> PathBuf {
    if cfg!(windows) {
        let absolute_path_str = absolute_path.display().to_string();
        if absolute_path_str.starts_with(r"\\?") {
            return PathBuf::from(absolute_path_str.replace(r"\\?\", ""));
        }
    }
    absolute_path.to_path_buf()
}

#[allow(clippy::expect_used)]
#[must_use]
/// Remove leading and trailing `/`s from the `path` string.
pub fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}
