use crate::server::tracing::StelaeRootSpanBuilder;
use actix_web::{get, web, App, HttpServer};
use entity::sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};
use std::env;
use std::path::PathBuf;
use tracing_actix_web::TracingLogger;

#[derive(Debug, Clone)]
struct AppState {
    archive_path: PathBuf, //TODO: this should be an Archive struct
    connection: DatabaseConnection,
}

#[get("/")]
async fn index() -> &'static str {
    "Welcome to Publish Server"
}

/// Serve documents in a Stelae archive.
#[actix_web::main]
pub async fn serve_archive(
    raw_library_path: &str,
    library_path: PathBuf,
    port: u16,
) -> std::io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_library_path}' on http://{bind}:{port}.",);

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in as an env variable");
    let connection = Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    let state = AppState {
        archive_path: library_path,
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

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
