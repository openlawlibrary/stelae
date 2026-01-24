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

/// Provides an HTTP `ETag` value derived from the Git blob hash of the requested file.
///
/// This header is included in responses from the `git` microserver and the `_archive` endpoint,
/// allowing clients and intermediaries to efficiently validate cached content.
///
/// The `ETag` is generated from the blob’s object ID, ensuring that the value changes whenever
/// the file contents change, while remaining stable across identical content in different
/// commits.
///
/// Example:
///
/// For a request resolving to a Git blob with hash:
///
/// `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`
///
/// the header will be set as:
///
/// `ETag: "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"`
pub const HTTP_E_TAG: &str = "ETAG";

/// Checks if a given `ETag` matches any of the values in an `If-None-Match` header.
///
/// This function splits the `If-None-Match` header by commas (to support multiple `ETags`),
/// trims whitespace, and compares each value to the provided `etag`.
///
/// # Arguments
///
/// * `header` - The value of the `If-None-Match` HTTP request header.
///   May contain one or more comma-separated `ETags`.
/// * `etag` - The server's current `ETag` for the resource.
///
/// # Returns
///
/// `true` if the provided `etag` matches any of the values in `header`.
/// `false` otherwise.
///
/// # Example
///
/// ```rust
/// use stelae::server::headers::matches_if_none_match;
/// let etag = "\"abc123\"";
/// assert!(matches_if_none_match("\"abc123\"", etag));
/// assert!(matches_if_none_match("\"xyz\", \"abc123\"", etag));
/// assert!(!matches_if_none_match("\"xyz\"", etag));
/// assert!(!matches_if_none_match("", etag));
/// ```
#[must_use]
pub fn matches_if_none_match(header: &str, etag: &str) -> bool {
    header.split(',').any(|tag| tag.trim() == etag)
}
