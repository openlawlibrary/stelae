pub mod db_data;

use crate::archive_testtools::{self, config::ArchiveType, utils};
use actix_http::Request;
use actix_service::Service;
use actix_web::{
    dev::ServiceResponse,
    rt::time,
    test::{self},
    Error,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Once;
use stelae::db::{self, DatabaseConnection};
use stelae::server::api::state::Global;
use tempfile::Builder;
static INIT: Once = Once::new();

use actix_http::body::MessageBody;

use stelae::server::app;
use stelae::stelae::archive::Archive;

pub const BASIC_MODULE_NAME: &str = "basic";

pub fn blob_to_string(blob: Vec<u8>) -> String {
    core::str::from_utf8(blob.as_slice()).unwrap().into()
}

// TODO: consider adding abort! test macro,
// which aborts the current test.
// then we can manually inspect the state of the test environment

// to manually inspect state of test environment at present,
// we use anyhow::bail!() which aborts the entire test suite.

#[derive(Debug, Clone)]
pub struct TestAppState {
    pub archive: Archive,
    pub db: DatabaseConnection,
}

impl Global for TestAppState {
    fn archive(&self) -> &Archive {
        &self.archive
    }
    fn db(&self) -> &db::DatabaseConnection {
        &self.db
    }
}

pub async fn initialize_app(
    archive_path: &Path,
) -> impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error> {
    let archive = Archive::parse(archive_path.to_path_buf(), archive_path, false).unwrap();
    let db = connect_test_db(archive_path).await;
    let state = TestAppState { archive, db };
    let app = app::init(&state).unwrap();
    test::init_service(app).await
}

/// Like `initialize_app`, but also returns a `DatabaseConnection`
pub async fn initialize_app_with_db(
    archive_path: &Path,
) -> (
    impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error>,
    DatabaseConnection,
) {
    let archive = Archive::parse(archive_path.to_path_buf(), archive_path, false).unwrap();
    let db = connect_test_db(archive_path).await;
    let db_handle = db.clone();
    let state = TestAppState { archive, db };
    let app = app::init(&state).unwrap();
    (test::init_service(app).await, db_handle)
}

pub async fn get_db(archive_path: &Path) -> DatabaseConnection {
    connect_test_db(archive_path).await
}

/// Connects to a real, file-backed `SQLite` database inside the archive's fixture
/// `TempDir` and runs migrations against it.
///
/// Uses `DELETE` journal mode rather than production's `WAL`. `WAL` persists as
/// `-wal`/`-shm` side files that are only checkpointed and removed once the *last*
/// connection to the database closes; if that close doesn't fully complete before the
/// test's `TempDir` is dropped, those files are left with open handles and
/// `TempDir::drop` silently fails to remove the directory -- leaving stale `.tmp*`
/// folders behind
async fn connect_test_db(archive_path: &Path) -> DatabaseConnection {
    let sqlite_db_path = &archive_path.join(PathBuf::from(".taf/db.sqlite3"));
    let db_url = format!("sqlite:///{}?mode=rwc", sqlite_db_path.to_string_lossy());
    let db = match DatabaseConnection::connect_with_journal_mode(&db_url, "DELETE").await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("error: could not connect to test database.");
            tracing::error!("Error: {:?}", err);
            panic!()
        }
    };
    match db.kind {
        db::DatabaseKind::Sqlite => {
            sqlx::migrate!("./migrations/sqlite")
                .run(&db.pool)
                .await
                .expect("failed to run migrations against test database");
        }
    }
    db
}

pub fn initialize_archive(archive_type: ArchiveType) -> Result<tempfile::TempDir> {
    match initialize_archive_without_bare(archive_type) {
        Ok(td) => {
            utils::make_all_git_repos_bare_recursive(&td)?;
            Ok(td)
        }
        Err(err) => Err(err),
    }
}

pub fn initialize_archive_without_bare(archive_type: ArchiveType) -> Result<tempfile::TempDir> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/");

    let td = Builder::new().tempdir_in(&path).unwrap();

    if let Err(err) = archive_testtools::initialize_archive_inner(archive_type, &td) {
        dbg!(&err);
        use std::mem::ManuallyDrop;
        let td = ManuallyDrop::new(td);
        // TODO: better error handling on testing failure
        let error_output_directory = path.clone().join(PathBuf::from("error_output_directory"));
        std::fs::remove_dir_all(&error_output_directory).unwrap();
        std::fs::rename(td.path(), &error_output_directory).expect("Failed to move temp directory");

        eprintln!("Failed to remove '{error_output_directory:?}', please try to remove directory by hand. Original error: {err}");
        return Err(err);
    }
    Ok(td)
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
