use crate::archive_testtools::{
    self, get_basic_test_data_repositories, ArchiveType, GitRepository, Jurisdiction,
    TestDataRepositoryContext, TestDataRepositoryType,
};
use actix_http::Request;
use actix_service::Service;
use actix_web::{
    dev::ServiceResponse,
    test::{self},
    Error,
};
use anyhow::Result;
use git2::{Commit, Repository};
use std::sync::Once;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tempfile::{Builder, TempDir};
static INIT: Once = Once::new();

use actix_http::body::MessageBody;

use stelae::stelae::archive::{self, Archive};
use stelae::{
    server::publish::{init_app, init_shared_app_state, AppState},
    stelae::types::repositories::Repositories,
};

pub const BASIC_MODULE_NAME: &str = "basic";

pub fn blob_to_string(blob: Vec<u8>) -> String {
    core::str::from_utf8(blob.as_slice()).unwrap().into()
}

// TODO: consider adding abort! test macro,
// which aborts the current test.
// then we can manually inspect the state of the test environment

// to manually inspect state of test environment at present,
// we use anyhow::bail!() which aborts the entire test suite, which is not ideal.

pub async fn initialize_app(
    archive_path: &Path,
) -> impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error> {
    let archive = Archive::parse(archive_path.to_path_buf(), archive_path, false).unwrap();
    let state = AppState { archive };
    let root = state.archive.get_root().unwrap();
    let shared_state = init_shared_app_state(root);
    let app = init_app(shared_state.clone(), state.clone());
    test::init_service(app).await
}

pub fn initialize_archive(archive_type: ArchiveType) -> Result<tempfile::TempDir> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/");

    let td = Builder::new().tempdir_in(&path).unwrap();

    match initialize_archive_inner(archive_type, &td) {
        Ok(_) => Ok(td),
        Err(err) => {
            dbg!(&err);
            let error_output_directory = path.clone().join(PathBuf::from("error_output_directory"));
            std::fs::remove_dir_all(&error_output_directory).unwrap();
            std::fs::rename(td.path(), &error_output_directory)
                .expect("Failed to move temp directory");
            eprintln!(
                "{}", format!("Failed to remove '{error_output_directory:?}', please try to remove directory by hand. Original error: {err}")
            );
            Err(err)
        }
    }
}

fn initialize_archive_inner(archive_type: ArchiveType, td: &TempDir) -> Result<()> {
    match archive_type {
        ArchiveType::Basic(Jurisdiction::Single) => initialize_archive_basic(td),
        ArchiveType::Basic(Jurisdiction::Multi) => initialize_archive_multijurisdiction(td),
        ArchiveType::Multihost(_) => initialize_archive_multihost(td),
    }
}

fn initialize_archive_basic(td: &TempDir) -> Result<()> {
    let org_name = "test_org";

    archive::init(
        td.path().to_owned(),
        "law".into(),
        org_name.into(),
        None,
        false,
    )
    .unwrap();
    let stele = initialize_stele(
        td.path().to_path_buf(),
        org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
    )
    .unwrap();
    // anyhow::bail!("Something went wrong!");
    Ok(())
}

fn initialize_archive_multijurisdiction(td: &TempDir) -> Result<()> {
    unimplemented!()
}

fn initialize_archive_multihost(td: &TempDir) -> Result<()> {
    unimplemented!()
}

pub fn initialize_stele(
    path: PathBuf,
    org_name: &str,
    data_repositories: &[TestDataRepositoryContext],
) -> Result<()> {
    let path = path.join(org_name);
    init_data_repositories(&path, data_repositories)?;
    init_auth_repository(&path, org_name, data_repositories)?;
    Ok(())
}

pub fn init_auth_repository(
    path: &Path,
    org_name: &str,
    data_repositories: &[TestDataRepositoryContext],
) -> Result<GitRepository> {
    let mut path = path.to_path_buf();
    path.push("law");
    std::fs::create_dir_all(&path).unwrap();

    let repo = GitRepository::init(&path).unwrap();

    path.push("targets");

    let content = r#"{
        "repositories": {
          "test_org/law-html": {
            "custom": {
              "type": "html",
              "serve": "historical",
              "location_regex": "/",
              "routes": [".*"]
            }
          }
        }
      }"#;

    repo.add_file(&path, "repositories.json", content).unwrap();
    repo.commit(Some("targets/repositories.json"), "Add repositories.json")
        .unwrap();
    Ok(repo)
}

pub fn init_data_repositories(
    path: &Path,
    data_repositories: &[TestDataRepositoryContext],
) -> Result<Vec<GitRepository>> {
    let mut data_git_repositories: Vec<GitRepository> = Vec::new();
    for data_repo in data_repositories {
        let mut path = path.to_path_buf();
        path.push(data_repo.name);
        std::fs::create_dir_all(&path).unwrap();
        let git_repo = GitRepository::init(&path).unwrap();
        init_data_repository(&git_repo, data_repo)?;
        data_git_repositories.push(git_repo);
    }
    Ok(data_git_repositories)
}

fn init_data_repository(
    git_repo: &GitRepository,
    data_repo: &TestDataRepositoryContext,
) -> Result<()> {
    for path in data_repo.paths.iter() {
        add_fixture_file_to_git_repo(git_repo, path)?;
    }
    git_repo.commit(None, "Add initial data").unwrap();
    Ok(())
}

fn add_fixture_file_to_git_repo(git_repo: &GitRepository, path: &str) -> Result<()> {
    let path_buf = PathBuf::from(path);
    let filename = path_buf.file_name().unwrap().to_str().unwrap();
    let static_file_path = get_static_file_path(filename);
    copy_file(&static_file_path, &git_repo.path.join(path)).unwrap();
    Ok(())
}

/// Returns a static file path for the given filename.
/// If the file does not exist, returns the default static file path.
fn get_static_file_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/static_files");
    let static_file_path = PathBuf::from(&path).join(filename);

    if static_file_path.exists() {
        return static_file_path;
    }

    let ext = Path::new(filename)
        .extension()
        .map_or("html", |ext| ext.to_str().map_or("", |ext_str| ext_str));
    let filename = get_default_static_filename(ext);

    PathBuf::from(path).join(filename)
}

/// Used to initialize the test environment for git micro-server.
pub fn initialize_git() {
    INIT.call_once(|| {
        let repo_path =
            get_test_archive_path(BASIC_MODULE_NAME).join(PathBuf::from("test/law-html"));
        let heads_path = repo_path.join(PathBuf::from("refs/heads"));
        std::fs::create_dir_all(heads_path).unwrap();
        let tags_path = repo_path.join(PathBuf::from("refs/tags"));
        std::fs::create_dir_all(tags_path).unwrap();
    });
}

pub fn get_test_archive_path(mod_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/");
    path.push(mod_name.to_owned() + "/archive");
    path
}
