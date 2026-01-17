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

/// Controls client- and proxy-side caching behavior for responses served by the application.
///
/// The `Cache-Control` header may be included in client requests to influence how intermediary
/// caches (such as NGINX) and origin services handle cached responses.
///
/// When present in a request, this header can be used to bypass upstream caches and force
/// revalidation or refetching of content from the origin service.
///
/// Example:
///
/// To require cached content to be revalidated before use, the client may send:
///
/// `Cache-Control: must-revalidate`
///
/// This ensures that intermediaries do not serve stale cached responses without first
/// confirming freshness with the origin.
///
/// Common directives:
/// - `no-cache`: Forces caches to revalidate with the origin before serving a response
/// - `no-store`: Prevents caches from storing the response
/// - `max-age=0`: Indicates that cached responses are immediately stale
/// - `must-revalidate`: Requires strict revalidation once a response becomes stale
///
/// # Notes
///
/// - In this system, the presence of a `Cache-Control` header may cause upstream caches
///   (such as NGINX proxy or uWSGI caches) to be bypassed entirely.
/// - Actual cache behavior depends on both request directives and server-side cache
///   configuration.
pub const HTTP_CACHE_CONTROL: &str = "Cache-Control";
