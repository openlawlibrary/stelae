//! Headers used in the Stelae server.

/// Provides the relative file path (starting from the git repository)
/// of the corresponding git blob.
///
/// This header is included in responses from the `git` microserver and the `_stelae` endpoint,
/// indicating the location of the file within the repository.
///
/// Example:
///
/// For a URL request: `a/b/c/1`
/// the header will be set as:
///
/// `X-File-Path: a/b/c/1/index.html`
pub const HTTP_X_FILE_PATH: &str = "X-File-Path";
