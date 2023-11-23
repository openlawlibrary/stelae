use super::archive_testtools;
use actix_http::Request;
use actix_service::Service;
use actix_web::{
    dev::ServiceResponse,
    test::{self},
    Error,
};
use anyhow::Result;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Once;
use tempfile::{Builder, TempDir};

static INIT: Once = Once::new();

use actix_http::body::MessageBody;

use stelae::server::publish::{init_app, init_shared_app_state, AppState};
use stelae::stelae::archive::{self, Archive};

pub const BASIC_MODULE_NAME: &str = "basic";

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

pub fn initialize_archive(script_name: &str) -> Result<tempfile::TempDir> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/");

    let td = Builder::new().tempdir_in(path).unwrap();
    archive::init(
        td.path().to_owned(),
        "law".into(),
        "test".into(),
        None,
        false,
    );
    Ok(td)
}

/// Used to initialize the test environment for git micro-server.
pub fn initialize_git() {
    INIT.call_once(|| {
        let repo_path =
            get_test_archive_path(BASIC_MODULE_NAME).join(PathBuf::from("test/law-html"));
        let heads_path = repo_path.join(PathBuf::from("refs/heads"));
        create_dir_all(heads_path).unwrap();
        let tags_path = repo_path.join(PathBuf::from("refs/tags"));
        create_dir_all(tags_path).unwrap();
    });
}

pub fn get_test_archive_path(mod_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/");
    path.push(mod_name.to_owned() + "/archive");
    path
}
