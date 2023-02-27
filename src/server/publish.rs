//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
use crate::db;
use crate::server::tracing::StelaeRootSpanBuilder;
use actix_web::{get, web, App, HttpServer};
use entity::sea_orm::DatabaseConnection;
use std::path::PathBuf;
use tracing_actix_web::TracingLogger;
/// Global, read-only state
#[derive(Debug, Clone)]
struct AppState {
    /// Path to the Stelae archive
    archive_path: PathBuf, //TODO: this should be an Archive struct
    /// Database connection
    connection: DatabaseConnection,
}

/// Index path for testing purposes
#[get("/")]
async fn index() -> &'static str {
    "Welcome to Publish Server"
}

/// Serve documents in a Stelae archive.
#[actix_web::main]
pub async fn serve_archive(
    raw_archive_path: &str,
    archive_path: PathBuf,
    port: u16,
) -> std::io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    let Ok(connection) = db::init::connect(&archive_path).await else {
        tracing::error!(
            "error: could not connect to database. Confirm that local `SQLite` database exists in `.stelae` directory in `{}`",
            &raw_archive_path
        );
        std::process::exit(1);
    };
    let state = AppState {
        archive_path,
        connection,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
            .app_data(web::Data::new(state.clone()))
            .configure(init)
    })
    .bind((bind, port))?
    .run()
    .await
}

/// Routes
fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
