use crate::archive_testtools::{
    self, ArchiveType, DataRepositoryType, GitRepository, Jurisdiction,
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
            std::fs::remove_dir_all(&error_output_directory);
            std::fs::rename(td.path(), &error_output_directory)
                .expect("Failed to move temp directory");
            eprintln!(
                "{}", format!("Failed to remove '{error_output_directory:?}', please try to do that by hand. Original error: {err}")
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
    );
    let stele = initialize_stele(
        td.path().to_path_buf(),
        org_name,
        &[
            DataRepositoryType::Html("html".into()),
            DataRepositoryType::Rdf("rdf".into()),
            DataRepositoryType::Xml("xml".into()),
            DataRepositoryType::Xml("xml-codified".into()),
            DataRepositoryType::Pdf("pdf".into()),
        ],
    )
    .unwrap();
    // let law = make_repository("make-law-repo.sh", &path).unwrap();
    // let law_html = make_repository("make-law-html-repo.sh", &path).unwrap();
    anyhow::bail!("Something went wrong!");
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
    data_repositories: &[DataRepositoryType],
) -> Result<()> {
    init_data_repositories(&path, org_name, data_repositories)?;
    init_auth_repository(&path, org_name, data_repositories)?;
    Ok(())
}

pub fn init_auth_repository(
    path: &Path,
    org_name: &str,
    data_repositories: &[DataRepositoryType],
) -> Result<GitRepository> {
    let mut path = path.to_path_buf();
    path.push(format!("{org_name}/law"));
    std::fs::create_dir_all(&path).unwrap();

    let repo = GitRepository::init(&path).unwrap();
    add_repositories_json(&repo, &path)?;
    Ok(repo)
}

pub fn init_data_repositories(
    path: &Path,
    org_name: &str,
    data_repositories: &[DataRepositoryType],
) -> Result<()> {
    let mut data_git_repositories: Vec<GitRepository> = Vec::new();
    for data_type in data_repositories {
        let mut path = path.to_path_buf();
        path.push(format!("{}/law-{}", org_name, data_type.to_string()));
        std::fs::create_dir_all(&path).unwrap();
        let repo = GitRepository::init(&path).unwrap();
        match data_type {
            DataRepositoryType::Html(name) => {
                add_html(&repo, &path, name)?;
            }
            DataRepositoryType::Rdf(name) => {
                init_rdf_repository(&repo, &path, name)?;
            }
            DataRepositoryType::Xml(name) => {
                init_xml_repository(&repo, &path, name)?;
            }
            DataRepositoryType::Pdf(name) => {
                init_pdf_repository(&repo, &path, name)?;
            }
            DataRepositoryType::Other(name) => {
                init_other_repository(&repo, &path, name)?;
            }
        }
    }
    Ok(())
}

fn add_repositories_json(repo: &GitRepository, path: &Path) -> Result<()> {
    let mut path = path.to_path_buf();
    path.push("targets");
    let content = r#"{
        "law": {
            "custom": {
                "type": "data",
                "allow-unauthenticated-commits": true,
                "serve": "law",
                "routes": [
                    "law/{path:.*}"
                ],
                "serve-prefix": "law",
                "is-fallback": true
            }
        }
    }"#;

    repo.write_file(&path, "repositories.json", content)
        .unwrap();
    repo.commit("targets/repositories.json", "Add repositories.json")
        .unwrap();

    Ok(())
}
pub fn make_repository(script_name: &str, path: &Path) -> Result<()> {
    archive_testtools::execute_script(script_name, path.canonicalize()?)?;
    // TODO: return repository
    Ok(())
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
