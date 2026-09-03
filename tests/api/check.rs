use std::path::Path;

use anyhow::Result;
use stelae::server::errors::CliError;
use stelae::stelae::types::repositories::Repositories;
use stelae::stelae::types::targets_metadata::TargetsMetadata;
use stelae::utils::check;

use crate::archive_testtools::{
    self,
    config::{get_basic_test_data_repositories, ArchiveType, Jurisdiction},
    write_to_file, GitRepository,
};
use crate::common;

/// Names of the six data repositories created by
/// `get_basic_test_data_repositories`, used to patch in valid target files
/// for tests that need a clean baseline.
const BASIC_REPO_NAMES: [&str; 6] = [
    "law-html",
    "law-rdf",
    "law-xml",
    "law-xml-codified",
    "law-pdf",
    "law-other",
];

/// Make the root Stele of a basic archive (created via
/// `initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single))`)
/// fully valid: scopes set, and a valid target file (with all fields) for
/// every default data repository except those listed in `skip_target_files`.
fn make_root_valid(archive_path: &Path, org: &str, skip_target_files: &[&str]) -> Result<()> {
    let org_path = archive_path.join(org);
    let repo_path = org_path.join("law");

    archive_testtools::init_auth_repository(
        &org_path,
        org,
        &get_basic_test_data_repositories()?,
        Some(&vec!["us/example".to_string()]),
        None,
    )?;

    for name in BASIC_REPO_NAMES.iter().copied() {
        if skip_target_files.contains(&name) {
            continue;
        }
        archive_testtools::add_target_file(
            &repo_path,
            org,
            name,
            &TargetsMetadata {
                branch: "main".into(),
                commit: "abc123".into(),
                build_date: Some("2024-01-01".into()),
                codified_date: Some("2024-01-01".into()),
            },
        )?;
    }
    Ok(())
}

/// Initialize a Stele with an empty (but valid) `targets/repositories.json`
/// and no `targets/dependencies.json`. Used as a dependency target that
/// should recurse cleanly (0 errors, at most a "no scopes" warning).
fn init_minimal_valid_stele(archive_path: &Path, org: &str) -> Result<()> {
    let repo_path = archive_path.join(org).join("law");
    std::fs::create_dir_all(&repo_path)?;
    let repo = GitRepository::init(&repo_path)?;

    let repositories = Repositories::default();
    let content = serde_json::to_string_pretty(&repositories)?;
    repo.add_file(&repo_path.join("targets"), "repositories.json", &content)?;
    repo.commit(Some("targets/repositories.json"), "Add repositories.json")?;
    Ok(())
}

/// Initialize a Stele's auth repo with no `targets/repositories.json` at all.
fn init_stele_without_repositories_json(archive_path: &Path, org: &str) -> Result<()> {
    let repo_path = archive_path.join(org).join("law");
    std::fs::create_dir_all(&repo_path)?;
    GitRepository::init(&repo_path)?;
    Ok(())
}

// ---------------------------------------------------------------------
// Success
// ---------------------------------------------------------------------

#[test]
fn test_check_when_archive_valid_expect_success() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "unexpected warnings: {:?}",
        report.warnings
    );

    // Also confirm the CLI-facing entrypoint maps a clean report to Ok(()).
    let result = check::run(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    );
    assert!(result.is_ok(), "expected Ok(()), got {result:?}");
}

// ---------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------

#[test]
fn test_check_when_repositories_json_missing_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();

    init_stele_without_repositories_json(archive_path.path(), "ghost_org").unwrap();
    archive_testtools::add_dependencies(archive_path.path(), "test_org", vec!["ghost_org"], None)
        .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert_eq!(report.errors[0].stele, "ghost_org/law");
    assert!(report.errors[0]
        .message
        .contains("required file is missing"));

    // Also confirm run() maps a failed check to CliError::CheckFailed.
    let result = check::run(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    );
    assert!(
        matches!(result, Err(CliError::CheckFailed)),
        "got {result:?}"
    );
}

#[test]
fn test_check_when_repositories_json_malformed_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    write_to_file(
        &auth_repo_path,
        "{ not valid json".to_string(),
        "repositories.json".to_string(),
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(
        report.errors[0].message.contains("failed to parse archive"),
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn test_check_when_type_missing_except_docs_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    GitRepository::init(&archive_path.path().join("test_org/law-docs")).unwrap();

    let content = r#"
    {
        "repositories": {
            "test_org/law-html": {
                "custom": { "serve": "latest", "routes": [".*"] }
            },
            "test_org/law-docs": {
                "custom": { "serve": "latest", "routes": [".*"] }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    let metadata = TargetsMetadata {
        branch: "main".into(),
        commit: "abc123".into(),
        build_date: None,
        codified_date: None,
    };
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-html", &metadata).unwrap();
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-docs", &metadata).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0]
        .message
        .contains("missing required field 'type'"));
    assert!(report.errors[0].message.contains("law-html"));
}

#[test]
fn test_check_when_serve_prefix_and_routes_missing_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "repositories": {
            "test_org/law-html": {
                "custom": { "type": "html", "serve": "latest" }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();
    archive_testtools::add_target_file(
        &auth_repo_path,
        "test_org",
        "law-html",
        &TargetsMetadata {
            branch: "main".into(),
            commit: "abc123".into(),
            build_date: None,
            codified_date: None,
        },
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0]
        .message
        .contains("must have either 'serve-prefix' or 'routes'"));
}

#[test]
fn test_check_when_serve_value_invalid_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "repositories": {
            "test_org/law-html": {
                "custom": { "type": "html", "serve": "sometimes", "routes": [".*"] }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();
    archive_testtools::add_target_file(
        &auth_repo_path,
        "test_org",
        "law-html",
        &TargetsMetadata {
            branch: "main".into(),
            commit: "abc123".into(),
            build_date: None,
            codified_date: None,
        },
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0].message.contains("invalid 'serve' value"));
}

#[test]
fn test_check_when_serve_prefix_duplicated_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "repositories": {
            "test_org/law-xml": {
                "custom": { "type": "xml", "serve": "latest", "serve-prefix": "_xml" }
            },
            "test_org/law-xml-codified": {
                "custom": { "type": "xml", "serve": "latest", "serve-prefix": "_xml" }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    let metadata = TargetsMetadata {
        branch: "main".into(),
        commit: "abc123".into(),
        build_date: None,
        codified_date: None,
    };
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-xml", &metadata).unwrap();
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-xml-codified", &metadata)
        .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0]
        .message
        .contains("duplicate 'serve-prefix'"));
    assert!(report.errors[0].message.contains("law-xml"));
    assert!(report.errors[0].message.contains("law-xml-codified"));
}

#[test]
fn test_check_when_multiple_fallbacks_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "repositories": {
            "test_org/law-pdf": {
                "custom": { "type": "pdf", "serve": "latest", "routes": [".*\\.pdf"], "is_fallback": true }
            },
            "test_org/law-other": {
                "custom": { "type": "other", "serve": "latest", "routes": [".*"], "is_fallback": true }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    let metadata = TargetsMetadata {
        branch: "main".into(),
        commit: "abc123".into(),
        build_date: None,
        codified_date: None,
    };
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-pdf", &metadata).unwrap();
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "law-other", &metadata)
        .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0]
        .message
        .contains("multiple repositories marked"));
}

#[test]
fn test_check_when_data_repository_missing_or_not_git_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    // "not-a-repo" exists on disk but isn't a git repository.
    std::fs::create_dir_all(archive_path.path().join("test_org/not-a-repo")).unwrap();

    let content = r#"
    {
        "repositories": {
            "test_org/ghost-repo": {
                "custom": { "type": "html", "serve": "latest", "routes": [".*"] }
            },
            "test_org/not-a-repo": {
                "custom": { "type": "html", "serve": "latest", "routes": [".*"] }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    // Target files exist for both, so only the data-repository checks fire.
    let metadata = TargetsMetadata {
        branch: "main".into(),
        commit: "abc123".into(),
        build_date: None,
        codified_date: None,
    };
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "ghost-repo", &metadata)
        .unwrap();
    archive_testtools::add_target_file(&auth_repo_path, "test_org", "not-a-repo", &metadata)
        .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 2, "errors: {:?}", report.errors);
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("does not exist at")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("is not a valid git repository")));
}

#[test]
fn test_check_when_dependencies_json_malformed_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    write_to_file(
        &auth_repo_path,
        "{ not valid json".to_string(),
        "dependencies.json".to_string(),
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0].message.contains("failed to parse"));
}

#[test]
fn test_check_when_dependencies_business_rules_violated_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    init_minimal_valid_stele(archive_path.path(), "ghost_org").unwrap();
    init_minimal_valid_stele(archive_path.path(), "another_org").unwrap();

    let content = r#"
    {
        "dependencies": {
            "test_org/law": { "out-of-band-authentication": "abc123", "branch": "main" },
            "ghost_org/law": { "out-of-band-authentication": "", "branch": "main" },
            "another_org/law": { "out-of-band-authentication": "abc123", "branch": "" }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "dependencies.json".to_string()).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 3, "errors: {:?}", report.errors);
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("lists itself")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("out-of-band-authentication")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("non-empty 'branch'")));
}

#[test]
fn test_check_when_dependency_directory_missing_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "dependencies": {
            "ghost_org/law": { "out-of-band-authentication": "abc123", "branch": "main" }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "dependencies.json".to_string()).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert!(report.errors[0]
        .message
        .contains("does not exist on the filesystem"));
}

#[test]
fn test_check_when_nested_dependency_invalid_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();

    init_minimal_valid_stele(archive_path.path(), "dependent_org").unwrap();
    archive_testtools::add_dependencies(
        archive_path.path(),
        "test_org",
        vec!["dependent_org"],
        None,
    )
    .unwrap();

    let dependent_auth_repo_path = archive_path.path().join("dependent_org/law");
    write_to_file(
        &dependent_auth_repo_path,
        "{ not valid json".to_string(),
        "repositories.json".to_string(),
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert_eq!(
        report.errors[0].stele, "None",
        "errors: {:?}",
        report.errors
    );
    assert!(
        report.errors[0].message.contains("failed to parse"),
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn test_check_when_target_file_missing_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &["law-rdf"]).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert_eq!(report.errors[0].file, "targets/test_org/law-rdf");
    assert!(report.errors[0]
        .message
        .contains("target file does not exist"));
}

#[test]
fn test_check_when_target_file_malformed_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    write_to_file(
        &auth_repo_path,
        "{ not valid json".to_string(),
        "test_org/law-rdf".to_string(),
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 1, "errors: {:?}", report.errors);
    assert_eq!(report.errors[0].file, "targets/test_org/law-rdf");
    assert!(report.errors[0]
        .message
        .contains("target file can't be parsed"));
}

#[test]
fn test_check_when_target_file_missing_required_fields_expect_error() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &[]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = serde_json::to_string_pretty(&TargetsMetadata {
        branch: String::new(),
        commit: String::new(),
        build_date: Some("2024-01-01".into()),
        codified_date: Some("2024-01-01".into()),
    })
    .unwrap();
    write_to_file(&auth_repo_path, content, "test_org/law-rdf".to_string()).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.errors.len(), 2, "errors: {:?}", report.errors);
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("missing 'branch'")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.message.contains("missing 'commit'")));
}

#[test]
fn test_check_when_scopes_missing_expect_warning() {
    // Plain basic fixture: scopes are never set, and no target files are
    // written -- but missing target files are *errors*, not warnings, so
    // they don't interfere with asserting on warnings here.
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(report.warnings.len(), 1, "warnings: {:?}", report.warnings);
    assert!(report.warnings[0].message.contains("scopes"));
}

#[test]
fn test_check_when_target_file_missing_optional_dates_expect_warnings() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    make_root_valid(archive_path.path(), "test_org", &["law-rdf"]).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    archive_testtools::add_target_file(
        &auth_repo_path,
        "test_org",
        "law-rdf",
        &TargetsMetadata {
            branch: "main".into(),
            commit: "abc123".into(),
            build_date: None,
            codified_date: None,
        },
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert_eq!(report.warnings.len(), 2, "warnings: {:?}", report.warnings);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.message.contains("build-date")));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.message.contains("codified-date")));
}

#[test]
fn test_check_when_docs_repo_missing_optional_dates_expect_no_warning() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    GitRepository::init(&archive_path.path().join("test_org/law-docs")).unwrap();

    let content = r#"
    {
        "scopes": ["us/example"],
        "repositories": {
            "test_org/law-docs": {
                "custom": { "serve": "latest", "routes": [".*"] }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    archive_testtools::add_target_file(
        &auth_repo_path,
        "test_org",
        "law-docs",
        &TargetsMetadata {
            branch: "main".into(),
            commit: "abc123".into(),
            build_date: None,
            codified_date: None,
        },
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "unexpected warnings: {:?}",
        report.warnings
    );
}

#[test]
fn test_check_when_xml_repo_missing_codified_date_expect_only_build_date_warning() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let auth_repo_path = archive_path.path().join("test_org/law");

    let content = r#"
    {
        "scopes": ["us/example"],
        "repositories": {
            "test_org/law-xml": {
                "custom": { "type": "xml", "serve": "latest", "serve-prefix": "_xml" }
            }
        }
    }
    "#
    .to_string();
    write_to_file(&auth_repo_path, content, "repositories.json".to_string()).unwrap();

    archive_testtools::add_target_file(
        &auth_repo_path,
        "test_org",
        "law-xml",
        &TargetsMetadata {
            branch: "main".into(),
            commit: "abc123".into(),
            build_date: None,
            codified_date: None,
        },
    )
    .unwrap();

    let report = check::check(
        archive_path.path().to_str().unwrap(),
        archive_path.path().to_path_buf(),
    )
    .unwrap();

    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert_eq!(report.warnings.len(), 1, "warnings: {:?}", report.warnings);
    assert!(report.warnings[0].message.contains("build-date"));
}
