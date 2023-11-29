use super::archive_testtools;
use actix_http::Request;
use actix_service::Service;
use actix_web::{
    dev::ServiceResponse,
    test::{self},
    Error,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tempfile::{Builder, TempDir};

static INIT: Once = Once::new();

use actix_http::body::MessageBody;

use stelae::server::publish::{init_app, init_shared_app_state, AppState};
use stelae::stelae::archive::{self, Archive};

pub const BASIC_MODULE_NAME: &str = "basic";

pub enum ArchiveType {
    Basic,
    Multijurisdiction,
    Multihost,
}

pub async fn initialize_app(
) -> impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error> {
    initialize_git();
    let archive = Archive::parse(
        get_test_archive_path(BASIC_MODULE_NAME),
        &get_test_archive_path(BASIC_MODULE_NAME),
        false,
    )
    .unwrap();
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

    //todo:

    //let law-html = execute_script('make-law-html', td.path(), script_name);
    //let law-rdf = execute_script('make-law-rdf', td.path(), script_name);
    //let law-xml = execute_script('make-law-xml', td.path(), script_name);
}

fn initialize_archive_inner(archive_type: ArchiveType, td: &TempDir) -> Result<()> {
    match archive_type {
        ArchiveType::Basic => initialize_archive_basic(td),
        ArchiveType::Multijurisdiction => initialize_archive_multijurisdiction(td),
        ArchiveType::Multihost => initialize_archive_multihost(td),
    }
}

fn initialize_archive_basic(td: &TempDir) -> Result<()> {
    let mut path = td.path().to_owned();
    path.push("test");
    std::fs::create_dir_all(&path).unwrap();

    archive::init(
        td.path().to_owned(),
        "law".into(),
        "test".into(),
        None,
        false,
    );
    let law_repo = make_repository("make-law-repo.sh", &path).unwrap();
    // anyhow::bail!("Something happened!");
    Ok(())
}

fn initialize_archive_multijurisdiction(td: &TempDir) -> Result<()> {
    unimplemented!()
}

fn initialize_archive_multihost(td: &TempDir) -> Result<()> {
    unimplemented!()
}

pub fn make_repository(script_name: &str, path: &Path) -> Result<()> {
    archive_testtools::execute_script(script_name, path.canonicalize()?)?;
    // TODO: return repository
    // panic!("Something happened!");
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
