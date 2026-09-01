//! Check the structural and semantic validity of a Stelae archive.
//!
//! Mirrors `nginx -t`: walks the archive the same way `serve`/`update` do,
//! but instead of starting a server or writing to the database, it
//! validates every stele's required files and reports *all* problems it
//! finds rather than stopping at the first one.

#![expect(
    clippy::iter_over_hash_type,
    reason = "List of repositories that are registered as routes are always sorted, even with iterating over hash type"
)]

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use git2::Repository as GitRepository;
use serde::Deserialize;

use crate::server::errors::CliError;
use crate::stelae::archive::Archive;
use crate::stelae::stele::Stele;
use crate::stelae::types::dependencies::Dependencies;
use crate::stelae::types::repositories::{Repositories, Repository};
use crate::utils::archive::get_name_parts;

/// A single problem found while checking the archive that causes `check`
/// to fail (exit non-zero).
#[derive(Debug)]
pub struct Error {
    /// Qualified name (`org/name`) of the Stele the error was found in.
    pub stele: String,
    /// File the error relates to, relative to the Stele's auth repo (e.g. `targets/repositories.json`).
    pub file: String,
    /// Human-readable description of the problem.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.stele, self.file, self.message)
    }
}

/// A non-fatal problem found while checking the archive. Doesn't affect the
/// exit code, but is surfaced to the user alongside any errors.
#[derive(Debug)]
pub struct Warning {
    /// Qualified name (`org/name`) of the Stele the warning was found in.
    pub stele: String,
    /// File the warning relates to, relative to the Stele's auth repo.
    pub file: String,
    /// Human-readable description of the concern.
    pub message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.stele, self.file, self.message)
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
    fn error(&mut self, stele: &str, file: &str, message: impl Into<String>) {
        self.errors.push(Error {
            stele: stele.to_owned(),
            file: file.to_owned(),
            message: message.into(),
        });
    }

    /// Record a non-fatal concern.
    fn warning(&mut self, stele: &str, file: &str, message: impl Into<String>) {
        self.warnings.push(Warning {
            stele: stele.to_owned(),
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
/// `src/stelae/types/info.rs`.
#[derive(Debug, Deserialize)]
struct Info {
    /// Namespace of the Stele.
    #[expect(
        dead_code,
        reason = "field presence is what we validate; value isn't consumed here"
    )]
    namespace: String,
    /// Name of the Stele.
    #[expect(
        dead_code,
        reason = "field presence is what we validate; value isn't consumed here"
    )]
    name: String,
}

/// Run `check` as the CLI entrypoint: logs a report mirroring `nginx -t`,
/// and maps the result to a CLI exit code.
///
/// # Errors
/// Returns `CliError::ArchiveParseError` if the archive can't be parsed.
/// Returns `CliError::CheckFailed` if the archive parses but fails validation.
pub fn run(raw_archive_path: &str, archive_path: PathBuf) -> Result<(), CliError> {
    let report = check(raw_archive_path, archive_path)?;

    for warning in &report.warnings {
        tracing::warn!("stelae check: warning: {warning}");
    }

    if report.errors.is_empty() {
        tracing::info!("stelae check: '{raw_archive_path}' is syntactically valid.");
        tracing::info!("stelae check: archive test is successful.");
        return Ok(());
    }

    tracing::error!(
        "stelae check: '{raw_archive_path}' failed validation with {} error(s):",
        report.errors.len()
    );
    for error in &report.errors {
        tracing::error!("  - {error}");
    }

    Err(CliError::CheckFailed)
}

/// Check the validity of a Stelae archive's configuration without starting a server.
///
/// Parses the archive to resolve the root Stele, then recursively checks the
/// root and every Stele it depends on (per `targets/dependencies.json`),
/// mirroring the traversal `Archive::parse`/`traverse_children` use for
/// `serve`/`update`. Unlike that traversal, `check` never skips a problem
/// silently: every issue found across every Stele is collected and returned
/// together, similar to `nginx -t`.
///
/// # Errors
/// Returns `CliError::ArchiveParseError` if the archive itself can't be
/// parsed (e.g. no root Stele found).
pub fn check(raw_archive_path: &str, archive_path: PathBuf) -> Result<Report, CliError> {
    let mut report = Report::default();
    tracing::info!("Checking Stelae archive at '{raw_archive_path}'.");

    let Ok(archive) = Archive::parse(archive_path, &PathBuf::from(raw_archive_path), false) else {
        report.error("None", raw_archive_path, "failed to parse archive");
        return Ok(report);
    };

    let Ok(root) = archive.get_root() else {
        report.error(
            "None",
            raw_archive_path,
            "could not determine root Stele for archive",
        );
        return Ok(report);
    };

    let mut visited = vec![root.get_qualified_name()];
    check_stele(&archive, root, &mut visited, &mut report);

    Ok(report)
}

/// Recursively check a Stele and all of its dependencies, accumulating
/// every problem found into `report` rather than stopping at the first one.
fn check_stele(archive: &Archive, stele: &Stele, visited: &mut Vec<String>, report: &mut Report) {
    let qualified_name = stele.get_qualified_name();
    tracing::info!("Checking Stele '{qualified_name}'.");

    check_repositories_json(stele, report);
    check_info_json(stele, report);
    // mirrors.json is not yet a stable/implemented format archive-wide, so
    // we don't validate its contents yet. See check_mirrors_json below.

    let Some(dependencies) = check_dependencies_json(stele, report) else {
        return;
    };

    for qualified_dep_name in dependencies.sorted_dependencies_names() {
        if visited.contains(&qualified_dep_name) {
            continue;
        }
        visited.push(qualified_dep_name.clone());

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

        let child = match Stele::new(
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

        check_stele(archive, &child, visited, report);
    }
}

/// Check `targets/dependencies.json`: that it parses into `Dependencies`,
/// and that each entry is internally consistent (non-empty `branch` and
/// `out-of-band-authentication`, no self-reference).
///
/// Returns `Some(dependencies)` so the caller can recurse, or `None` if the
/// file is absent (not required -- a leaf Stele may have none) or
/// unparseable (already recorded as an error).
fn check_dependencies_json(stele: &Stele, report: &mut Report) -> Option<Dependencies> {
    const FILE: &str = "targets/dependencies.json";
    let qualified_name = stele.get_qualified_name();

    let dependencies = match stele.get_dependencies() {
        Ok(Some(dependencies)) => dependencies,
        Ok(None) => return None,
        Err(err) => {
            report.error(&qualified_name, FILE, format!("failed to parse: {err}"));
            return None;
        }
    };

    check_dependencies_consistency(&qualified_name, &dependencies, report);

    Some(dependencies)
}

/// Validate business rules on `dependencies.json`: every entry must have a
/// non-empty `branch` and `out-of-band-authentication`, and a Stele
/// shouldn't list itself as a dependency.
fn check_dependencies_consistency(
    qualified_name: &str,
    dependencies: &Dependencies,
    report: &mut Report,
) {
    const FILE: &str = "targets/dependencies.json";
    for (name, dependency) in &dependencies.dependencies {
        if name == qualified_name {
            report.error(
                qualified_name,
                FILE,
                format!("Stele lists itself ('{name}') as a dependency"),
            );
        }

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

/// Check `targets/repositories.json`: that it exists, parses into
/// `Repositories`, and that its entries are internally consistent
/// (required `type`, required `serve-prefix`/`routes`, no duplicate
/// `serve-prefix`, at most one `is_fallback: true`). Also checks that each
/// referenced data repository exists on disk and has a valid target file.
fn check_repositories_json(stele: &Stele, report: &mut Report) {
    const FILE: &str = "targets/repositories.json";
    let qualified_name = stele.get_qualified_name();

    let Ok(blob) = stele.auth_repo.get_bytes_at_path("HEAD", FILE) else {
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
        report.warning(&qualified_name, FILE, "no 'scopes' defined for this Stele");
    }

    check_repositories_consistency(&qualified_name, &repositories, report);

    for repository in repositories.repositories.values() {
        check_data_repository_exists(stele, repository, report);
        check_target_file(stele, repository, report);
    }
}

/// Validate business rules across all repositories in one Stele's
/// `repositories.json`: required `type` (except repositories whose name
/// ends in `docs`), required `serve-prefix`/`routes`, no duplicate
/// `serve-prefix`, at most one fallback. Scoped per-Stele, since any Stele
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
        let is_docs_repo = repository.get_name().ends_with("docs");
        if repository.custom.repository_type.is_none() && !is_docs_repo {
            report.error(
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
            report.error(
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
fn check_data_repository_exists(stele: &Stele, repository: &Repository, report: &mut Report) {
    const FILE: &str = "targets/repositories.json";
    let qualified_name = stele.get_qualified_name();
    let org = repository.get_org();
    let name = repository.get_name();
    let path = stele.archive_path.join(&org).join(&name);

    if fs::metadata(&path).is_err() {
        report.error(
            &qualified_name,
            FILE,
            format!(
                "data repository '{org}/{name}' does not exist at '{}'",
                path.display()
            ),
        );
        return;
    }

    if GitRepository::open(&path).is_err() {
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

/// Check that a data repository referenced in `repositories.json` has a
/// corresponding target file at `targets/<stele_org>/<data_repo_name>`,
/// that it parses into `TargetsMetadata`.
/// It shows an error if `branch` or `commit` are missing,
/// and warnning if it's missing `build-date` or `codified-date`.
fn check_target_file(stele: &Stele, repository: &Repository, report: &mut Report) {
    let qualified_name = stele.get_qualified_name();
    let filename = repository.get_name();
    let file = format!("targets/{}/{filename}", stele.auth_repo.org);
    let is_docs_repo = repository.get_name().ends_with("docs");
    let is_xml_or_static = matches!(
        repository.get_type().as_deref(),
        Some("xml" | "static-assets")
    );

    let Ok(metadata_option) = stele.get_targets_metadata_at_commit_and_filename("HEAD", &filename)
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
///
/// This file's location isn't consistent across existing archives -- the
/// openlawlibrary/law archive itself has been seen using
/// `targets/<org>/protected/info.json` instead. Absence is therefore not
/// treated as an error; only presence with bad content is.
fn check_info_json(stele: &Stele, report: &mut Report) {
    const FILE: &str = "targets/protected/info.json";
    let qualified_name = stele.get_qualified_name();

    let Ok(blob) = stele.auth_repo.get_bytes_at_path("HEAD", FILE) else {
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
