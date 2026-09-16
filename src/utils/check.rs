//! Check the structural and semantic validity of a Fonds archive.
//!
//! Mirrors `nginx -t`: walks the archive the same way `serve`/`update` do,
//! but instead of starting a server or writing to the database, it
//! validates every fonds's required files and reports *all* problems it
//! finds rather than stopping at the first one.

#![expect(
    clippy::iter_over_hash_type,
    reason = "List of repositories that are registered as routes are always sorted, even with iterating over hash type"
)]

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::fs;
use std::path::PathBuf;

use git2::Repository as GitRepository;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::fonds::archive::Archive;
use crate::fonds::fonds::Fonds;
use crate::fonds::types::dependencies::Dependencies;
use crate::fonds::types::repositories::{Repositories, Repository};
use crate::server::errors::CliError;
use crate::utils::archive::get_name_parts;

/// A single problem found while checking the archive that causes `check`
/// to fail (exit non-zero).
#[derive(Debug)]
pub struct Error {
    /// Qualified name (`org/name`) of the Fonds the error was found in.
    pub fonds: String,
    /// File the error relates to, relative to the Fonds's auth repo (e.g. `targets/repositories.json`).
    pub file: String,
    /// Human-readable description of the problem.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.fonds, self.file, self.message)
    }
}

/// A non-fatal problem found while checking the archive. Doesn't affect the
/// exit code, but is surfaced to the user alongside any errors.
#[derive(Debug)]
pub struct Warning {
    /// Qualified name (`org/name`) of the Fonds the warning was found in.
    pub fonds: String,
    /// File the warning relates to, relative to the Fonds's auth repo.
    pub file: String,
    /// Human-readable description of the concern.
    pub message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.fonds, self.file, self.message)
    }
}

/// Accumulates every error and warning found while checking an archive.
#[derive(Debug, Default)]
pub struct Report {
    /// Problems that will cause `check` to exit non-zero.
    pub errors: Vec<Error>,
    /// Non-fatal concerns surfaced alongside any errors.
    pub warnings: Vec<Warning>,
}

impl Report {
    /// Record a fatal problem.
    fn error(&mut self, fonds: &str, file: &str, message: impl Into<String>) {
        self.errors.push(Error {
            fonds: fonds.to_owned(),
            file: file.to_owned(),
            message: message.into(),
        });
    }

    /// Record a non-fatal concern.
    fn warning(&mut self, fonds: &str, file: &str, message: impl Into<String>) {
        self.warnings.push(Warning {
            fonds: fonds.to_owned(),
            file: file.to_owned(),
            message: message.into(),
        });
    }
}

/// Info object, deserialized from `targets/protected/info.json`.
///
/// NOTE: this isn't yet defined as a shared type elsewhere in the codebase
/// (unlike `Repositories`/`Dependencies`), so it's kept local to `check`
/// for now. If it becomes load-bearing elsewhere, move it to
/// `src/fonds/types/info.rs`.
#[derive(Debug, Deserialize)]
struct Info {
    /// Namespace of the Fonds.
    #[expect(
        dead_code,
        reason = "field presence is what we validate; value isn't consumed here"
    )]
    namespace: String,
    /// Name of the Fonds.
    #[expect(
        dead_code,
        reason = "field presence is what we validate; value isn't consumed here"
    )]
    name: String,
}

/// Run `check` as the CLI entrypoint: logs a report,
/// and maps the result to a CLI exit code.
///
/// # Errors
/// Returns `CliError::CheckFailed` if the archive parses but fails validation.
pub fn run(raw_archive_path: &str, archive_path: PathBuf) -> Result<(), CliError> {
    let report = check(raw_archive_path, archive_path)?;

    for warning in &report.warnings {
        tracing::warn!("taf-server check: warning: {warning}");
    }

    if report.errors.is_empty() {
        tracing::info!("taf-server check: '{raw_archive_path}' is syntactically valid.");
        tracing::info!("taf-server check: archive test is successful.");
        return Ok(());
    }

    tracing::error!(
        "taf-server check: '{raw_archive_path}' failed validation with {} error(s):",
        report.errors.len()
    );
    for error in &report.errors {
        tracing::error!("  - {error}");
    }

    Err(CliError::CheckFailed)
}

/// Check the validity of a Fonds archive's configuration without starting a server.
///
/// Parses the archive to resolve the root Fonds, then recursively checks the
/// root and every Fonds it depends on (per `targets/dependencies.json`),
/// mirroring the traversal `Archive::parse`/`traverse_children` use for
/// `serve`/`update`. Unlike that traversal, `check` never skips a problem
/// silently: every issue found across every Fonds is collected and returned
/// together
///
/// # Errors
/// Returns `CliError::ArchiveParseError` if the archive itself can't be
/// parsed (e.g. no root Fonds found).
pub fn check(raw_archive_path: &str, archive_path: PathBuf) -> Result<Report, CliError> {
    let mut report = Report::default();
    tracing::info!("Checking Fonds archive at '{raw_archive_path}'.");

    let Ok(archive) = Archive::parse(archive_path, &PathBuf::from(raw_archive_path), false) else {
        report.error("None", raw_archive_path, "failed to parse archive");
        return Ok(report);
    };

    let Ok(root) = archive.get_root() else {
        report.error(
            "None",
            raw_archive_path,
            "could not determine root Fonds for archive",
        );
        return Ok(report);
    };

    let mut visited: Vec<String> = vec![];
    let mut depth: Vec<String> = vec![];
    check_fonds(&archive, root, &mut visited, &mut depth, &mut report);

    Ok(report)
}

/// Recursively check a Fonds and all of its dependencies, accumulating
/// every problem found into `report` rather than stopping at the first one.
fn check_fonds(
    archive: &Archive,
    fonds: &Fonds,
    visited: &mut Vec<String>,
    depth: &mut Vec<String>,
    report: &mut Report,
) {
    let qualified_name = fonds.get_qualified_name();
    tracing::info!("Checking Fonds '{qualified_name}'.");
    if depth.contains(&qualified_name) {
        report.error(
            depth.first().map_or("Fonds", |string| string.as_str()),
            "targets/dependencies.json",
            format!(
                "{qualified_name} repeated in a cycle:\n{} > {qualified_name}",
                depth.join(" > ")
            ),
        );
        return;
    }
    depth.push(qualified_name.clone());
    let is_visited = visited.contains(&qualified_name);

    if !is_visited {
        check_repositories_json(fonds, report);
        check_info_json(fonds, report);
        // mirrors.json is not yet a stable/implemented format archive-wide, so
        // we don't validate its contents yet. See check_mirrors_json below.
    }

    let Some(dependencies) = check_dependencies_json(fonds, is_visited, report) else {
        visited.push(qualified_name);
        depth.pop();
        return;
    };

    for qualified_dep_name in dependencies.sorted_dependencies_names() {
        let Ok((org, name)) = get_name_parts(&qualified_dep_name) else {
            report.error(
                &qualified_name,
                "targets/dependencies.json",
                format!("dependency '{qualified_dep_name}' is not in '<org>/<name>' format"),
            );
            continue;
        };

        let child_path = archive.path.join(&org).join(&name);
        if fs::metadata(&child_path).is_err() {
            report.error(
                &qualified_name,
                "targets/dependencies.json",
                format!(
                    "dependency '{qualified_dep_name}' does not exist on the filesystem at '{}'",
                    child_path.display()
                ),
            );
            continue;
        }

        let child = match Fonds::new(
            &archive.path,
            Some(name),
            Some(org.clone()),
            Some(archive.path.join(&org)),
            false,
        ) {
            Ok(child) => child,
            Err(err) => {
                report.error(
                    &qualified_name,
                    "targets/dependencies.json",
                    format!("failed to load dependency '{qualified_dep_name}': {err}"),
                );
                continue;
            }
        };

        check_fonds(archive, &child, visited, depth, report);
    }
    visited.push(qualified_name);
    depth.pop();
}

/// Check `targets/dependencies.json`: that it parses into `Dependencies`,
/// and that each entry is internally consistent (non-empty `branch` and
/// `out-of-band-authentication`, no self-reference).
///
/// Returns `Some(dependencies)` so the caller can recurse, or `None` if the
/// file is absent (not required -- a leaf Fonds may have none) or
/// unparseable (already recorded as an error).
fn check_dependencies_json(
    fonds: &Fonds,
    is_visited: bool,
    report: &mut Report,
) -> Option<Dependencies> {
    const FILE: &str = "targets/dependencies.json";
    let qualified_name = fonds.get_qualified_name();

    let Ok(blob) = fonds.auth_repo.get_bytes_at_path("HEAD", FILE) else {
        return None;
    };

    let Ok(raw) = String::from_utf8(blob.content) else {
        report.error(&qualified_name, FILE, "file is not valid UTF-8");
        return None;
    };

    let dependencies = match fonds.get_dependencies() {
        Ok(Some(dependencies)) => dependencies,
        Ok(None) => return None,
        Err(err) => {
            report.error(&qualified_name, FILE, format!("failed to parse: {err}"));
            return None;
        }
    };

    if !is_visited {
        check_dependencies_consistency(&qualified_name, &raw, &dependencies, report);
    }

    Some(dependencies)
}

/// Validate business rules on `dependencies.json`: every entry must have a
/// non-empty `branch` and `out-of-band-authentication`, and a Fonds
/// shouldn't list itself as a dependency.
fn check_dependencies_consistency(
    qualified_name: &str,
    raw: &str,
    dependencies: &Dependencies,
    report: &mut Report,
) {
    const FILE: &str = "targets/dependencies.json";

    match find_duplicate_dependency_names(raw) {
        Ok(Some(duplicates)) => report.error(
            qualified_name,
            FILE,
            format!("duplicate dependencies found: {}", duplicates.join(", ")),
        ),
        Ok(None) => {}
        Err(err) => report.error(
            qualified_name,
            FILE,
            format!("failed to check for duplicate dependencies: {err}"),
        ),
    }

    for (name, dependency) in &dependencies.dependencies {
        if dependency.branch.is_empty() {
            report.error(
                qualified_name,
                FILE,
                format!("dependency '{name}' is missing a non-empty 'branch'"),
            );
        }

        if dependency.out_of_band_authentication.is_empty() {
            report.error(
                qualified_name,
                FILE,
                format!("dependency '{name}' is missing a non-empty 'out-of-band-authentication'"),
            );
        }
    }
}

/// Re-parse the raw `dependencies.json` text to find any repeated key under "dependencies".
/// Deserializing into `(String, Value)` pairs preserves every occurrence in source order.
fn find_duplicate_dependency_names(raw: &str) -> serde_json::Result<Option<Vec<String>>> {
    struct Pairs(Vec<(String, serde_json::Value)>);

    #[expect(
        clippy::missing_trait_methods,
        reason = "Use serde default trait implementations"
    )]
    impl<'de> Deserialize<'de> for Pairs {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct PairsVisitor;

            #[expect(
                clippy::missing_trait_methods,
                reason = "Use serde default trait implementations"
            )]
            #[expect(
                clippy::absolute_paths,
                reason = "use of std::fmt::Result and core::result::Result"
            )]
            impl<'de> Visitor<'de> for PairsVisitor {
                type Value = Vec<(String, serde_json::Value)>;

                fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a JSON object")
                }

                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut pairs = Vec::new();
                    while let Some(pair) = map.next_entry()? {
                        pairs.push(pair);
                    }
                    Ok(pairs)
                }
            }
            deserializer.deserialize_map(PairsVisitor).map(Pairs)
        }
    }

    #[derive(Deserialize)]
    struct Document {
        dependencies: Pairs,
    }

    let document: Document = serde_json::from_str(raw)?;
    let mut names: Vec<String> = document
        .dependencies
        .0
        .into_iter()
        .map(|(name, _value)| name)
        .collect();
    Ok(find_duplicates_sorted(&mut names))
}

/// Given a list of strings, sort it in place and return any values that
/// appear more than once (each listed only once), or `None` if there are
/// no duplicates.
fn find_duplicates_sorted(names: &mut [String]) -> Option<Vec<String>> {
    names.sort_unstable();

    let mut duplicates = Vec::new();
    let mut last_duplicate: Option<&String> = None;

    for pair in names.windows(2) {
        let (Some(first), Some(second)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        if first == second && last_duplicate != Some(first) {
            duplicates.push(first.clone());
            last_duplicate = Some(first);
        }
    }

    if duplicates.is_empty() {
        None
    } else {
        Some(duplicates)
    }
}

/// Check `targets/repositories.json`: that it exists, parses into
/// `Repositories`, and that its entries are internally consistent
/// (required `type`, required `serve-prefix`/`routes`, no duplicate
/// `serve-prefix`, at most one `is_fallback: true`). Also checks that each
/// referenced data repository exists on disk and has a valid target file.
fn check_repositories_json(fonds: &Fonds, report: &mut Report) {
    const FILE: &str = "targets/repositories.json";
    let qualified_name = fonds.get_qualified_name();

    let Ok(blob) = fonds.auth_repo.get_bytes_at_path("HEAD", FILE) else {
        report.error(&qualified_name, FILE, "required file is missing");
        return;
    };

    let Ok(raw) = String::from_utf8(blob.content) else {
        report.error(&qualified_name, FILE, "file is not valid UTF-8");
        return;
    };

    let repositories: Repositories = match serde_json::from_str(&raw) {
        Ok(repositories) => repositories,
        Err(err) => {
            report.error(&qualified_name, FILE, format!("invalid JSON: {err}"));
            return;
        }
    };

    if repositories.scopes.as_ref().is_none_or(Vec::is_empty) {
        report.warning(&qualified_name, FILE, "no 'scopes' defined");
    }

    check_repositories_consistency(&qualified_name, &repositories, report);

    for repository in repositories.repositories.values() {
        check_data_repository_exists(fonds, repository, report);
        check_target_file(fonds, repository, report);
    }
}

/// Validate business rules across all repositories in one Fonds's
/// `repositories.json`: required `type` (except repositories whose name
/// ends in `docs`), required `serve-prefix`/`routes`, no duplicate
/// `serve-prefix`, at most one fallback. Scoped per-Fonds, since any Fonds
/// can be served standalone.
fn check_repositories_consistency(
    qualified_name: &str,
    repositories: &Repositories,
    report: &mut Report,
) {
    const FILE: &str = "targets/repositories.json";

    let mut prefixes: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut fallbacks: Vec<&str> = Vec::new();

    for (name, repository) in &repositories.repositories {
        if repository.custom.repository_type.is_none() {
            // IMPORTANT: change this to error when type is in every partner repository.json (law-docs)
            report.warning(
                qualified_name,
                FILE,
                format!("'{name}' is missing required field 'type'"),
            );
        }

        let has_prefix = repository
            .custom
            .scope
            .as_deref()
            .is_some_and(|prefix| !prefix.is_empty());
        let has_routes = repository
            .custom
            .routes
            .as_ref()
            .is_some_and(|routes| !routes.is_empty());
        if !has_prefix && !has_routes {
            report.warning(
                qualified_name,
                FILE,
                format!("'{name}' must have either 'serve-prefix' or 'routes'"),
            );
        }

        if !matches!(repository.custom.serve.as_str(), "latest" | "historical") {
            report.error(
                qualified_name,
                FILE,
                format!(
                    "'{name}' has invalid 'serve' value '{}': must be 'latest' or 'historical'",
                    repository.custom.serve
                ),
            );
        }

        if let Some(prefix) = repository.custom.scope.as_deref() {
            prefixes.entry(prefix).or_default().push(name);
        }

        if repository.custom.is_fallback.unwrap_or(false) {
            fallbacks.push(name);
        }
    }

    for (prefix, names) in prefixes {
        if names.len() > 1 {
            report.error(
                qualified_name,
                FILE,
                format!(
                    "duplicate 'serve-prefix' \"{prefix}\" used by: {}",
                    names.join(", ")
                ),
            );
        }
    }

    if fallbacks.len() > 1 {
        report.error(
            qualified_name,
            FILE,
            format!(
                "multiple repositories marked 'is_fallback: true': {}",
                fallbacks.join(", ")
            ),
        );
    }
}

/// Check that a data repository referenced in `repositories.json` actually
/// exists on disk and is a valid git repository.
fn check_data_repository_exists(fonds: &Fonds, repository: &Repository, report: &mut Report) {
    const FILE: &str = "targets/repositories.json";
    let qualified_name = fonds.get_qualified_name();
    let org = repository.get_org();
    let name = repository.get_name();
    let path = fonds.archive_path.join(&org).join(&name);

    if fs::metadata(&path).is_err() {
        if let Some(true) = repository.custom.archived {
            report.warning(
                &qualified_name,
                FILE,
                format!(
                    "data repository '{org}/{name}' does not exist at '{}'",
                    path.display()
                ),
            );
        } else {
            report.error(
                &qualified_name,
                FILE,
                format!(
                    "data repository '{org}/{name}' does not exist at '{}'",
                    path.display()
                ),
            );
        }
        return;
    }

    if GitRepository::open(&path).is_err() {
        if let Some(true) = repository.custom.archived {
            report.warning(
                &qualified_name,
                FILE,
                format!(
                    "'{org}/{name}' at '{}' is not a valid git repository",
                    path.display()
                ),
            );
        } else {
            report.error(
                &qualified_name,
                FILE,
                format!(
                    "'{org}/{name}' at '{}' is not a valid git repository",
                    path.display()
                ),
            );
        }
    }
}

/// Check that a data repository referenced in `repositories.json` has a
/// corresponding target file at `targets/<fonds_org>/<data_repo_name>`,
/// that it parses into `TargetsMetadata`.
/// It shows an error if `branch` or `commit` are missing,
/// and warnning if it's missing `build-date` or `codified-date`.
fn check_target_file(fonds: &Fonds, repository: &Repository, report: &mut Report) {
    let qualified_name = fonds.get_qualified_name();
    let filename = repository.get_name();
    let file = format!("targets/{}/{filename}", fonds.auth_repo.org);
    let is_docs_repo = repository.get_name().ends_with("docs");
    let is_xml_or_static = matches!(
        repository.get_type().as_deref(),
        Some("xml" | "static-assets")
    );

    let Ok(metadata_option) = fonds.get_targets_metadata_at_commit_and_filename("HEAD", &filename)
    else {
        report.error(&qualified_name, &file, "target file can't be parsed");
        return;
    };

    let Some(metadata) = metadata_option else {
        report.error(&qualified_name, &file, "target file does not exist");
        return;
    };

    // Required field validation
    if metadata.branch.is_empty() {
        report.error(&qualified_name, &file, "target file is missing 'branch'");
    }
    if metadata.commit.is_empty() {
        report.error(&qualified_name, &file, "target file is missing 'commit'");
    }

    // Optional fields – warnings only
    // serve historical only
    if !is_docs_repo && metadata.build_date.is_none() {
        report.warning(
            &qualified_name,
            &file,
            "target file is missing 'build-date'",
        );
    }
    if !is_docs_repo && !is_xml_or_static && metadata.codified_date.is_none() {
        report.warning(
            &qualified_name,
            &file,
            "target file is missing 'codified-date'",
        );
    }
}

/// Check `targets/protected/info.json`, if present.
fn check_info_json(fonds: &Fonds, report: &mut Report) {
    const FILE: &str = "targets/protected/info.json";
    let qualified_name = fonds.get_qualified_name();

    let Ok(blob) = fonds.auth_repo.get_bytes_at_path("HEAD", FILE) else {
        report.error(&qualified_name, FILE, "required file is missing");
        return;
    };

    let Ok(raw) = String::from_utf8(blob.content) else {
        report.error(&qualified_name, FILE, "file is not valid UTF-8");
        return;
    };

    if let Err(err) = serde_json::from_str::<Info>(&raw) {
        report.error(&qualified_name, FILE, format!("invalid JSON: {err}"));
    }
}
