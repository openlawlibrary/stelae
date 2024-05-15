//! Serve documents in a Stelae archive.
#![allow(
    clippy::exit,
    clippy::unused_async,
    clippy::infinite_loop,
    clippy::module_name_repetitions
)]
use crate::db;
use crate::server::api::state::App as AppState;
use crate::stelae::archive::Archive;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{App, Error, HttpServer};

use std::{io, path::PathBuf, process};

use actix_http::body::MessageBody;
use actix_service::ServiceFactory;

use super::api::state::Global;
use crate::server::api::routes;

/// Serve documents in a Stelae archive.
#[actix_web::main]
pub async fn serve_archive(
    raw_archive_path: &str,
    archive_path: PathBuf,
    port: u16,
    individual: bool,
) -> io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    let db = match db::init::connect(&archive_path).await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!(
                "error: could not connect to database. Confirm that DATABASE_URL env var is set correctly."
            );
            tracing::error!("Error: {:?}", err);
            process::exit(1);
        }
    };

    let archive = Archive::parse(archive_path, &PathBuf::from(raw_archive_path), individual)
        .unwrap_or_else(|err| {
            tracing::error!("Unable to parse archive at '{raw_archive_path}'.");
            tracing::error!("Error: {:?}", err);
            process::exit(1);
        });
    let state = AppState { archive, db };

    HttpServer::new(move || {
        init_app(&state).unwrap_or_else(|err| {
            tracing::error!("Unable to initialize app.");
            tracing::error!("Error: {:?}", err);
            process::exit(1);
        })
    })
    .bind((bind, port))?
    .run()
    .await
}

/// Initialize the application and all possible routing at start-up time.
///
/// # Arguments
/// * `state` - The application state
/// # Errors
/// Will error if unable to initialize the application
pub fn init_app<T: Global + Clone + 'static>(
    state: &T,
) -> anyhow::Result<
    App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    >,
> {
    let app = routes::register_app(App::new(), state)?;
    Ok(app)
}
