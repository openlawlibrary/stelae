//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::db;
use crate::server::tracing::StelaeRootSpanBuilder;
use actix_web::{get, web, App, HttpRequest, HttpServer};
use entity::sea_orm::DatabaseConnection;
use std::{collections::HashMap, path::PathBuf};
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
#[get("/t")]
async fn index() -> &'static str {
    "Welcome to Publish Server"
}

/// Index path for testing purposes
#[get("/test")]
async fn test(req: HttpRequest, data: web::Data<HashMap<String, String>>) -> String {
    format!(
        "{}, {}",
        req.path().to_owned(),
        data.get("cityofsanmateo")
            .unwrap_or(&("no value").to_owned())
    )
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
    {
        let mut smc_hashmap = HashMap::new();
        smc_hashmap.insert("cityofsanmateo".to_owned(), "some value for SMC".to_owned());

        cfg.service(
            web::scope("/us/ca/cities/san-mateo")
                .app_data(web::Data::new(smc_hashmap))
                .service(index)
                .service(test),
        );
    }
    {
        let mut dc_hashmap = HashMap::new();
        dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

        cfg.service(
            web::scope("/us/dc")
                .app_data(web::Data::new(dc_hashmap))
                .service(test),
        );
    }
}
