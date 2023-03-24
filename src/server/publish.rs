//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::server::tracing::StelaeRootSpanBuilder;
use crate::stelae::archive::Archive;
use crate::stelae::stele::Stele;
use actix_web::{get, web, App, HttpRequest, HttpServer};
use std::{collections::HashMap, path::Path, path::PathBuf};
use tracing_actix_web::TracingLogger;
/// Global, read-only state
#[derive(Debug, Clone)]
struct AppState {
    /// Path to the Stelae archive
    archive: Archive,
}

/// Index path for testing purposes
// #[get("/t")]
async fn index() -> &'static str {
    "Welcome to Publish Server"
}

/// Index path for testing purposes
// #[get("/test")]
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

    let mut state = AppState {
        archive: Archive {
            path: archive_path,
            stelae: HashMap::new(),
        },
    };

    let root_stele = state.archive.get_root();
    //TODO: figure out how to access specific stele instead of root stele
    // let stele = state.archive.get_current_stele(Path::new(&raw_archive_path));
    if let Ok(root_stele) = root_stele {
        //load dependencies.json file from stelae root
        // let dependencies = root_stele.get_dependencies().unwrap();
        // for stele_name in dependencies.dependencies.keys() {
// 
        // }
    }

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
    //for stele in archive.stelae.values {
        // for repository in stele.repositories {
        //     for scope in scopes {
                
        //     }
        // }
    //}
    {
        let mut smc_hashmap = HashMap::new();
        smc_hashmap.insert("cityofsanmateo".to_owned(), "some value for SMC".to_owned());

        //for each scope:
            //
        cfg.service(
            web::scope("/us/ca/cities/san-mateo")
                .app_data(web::Data::new(smc_hashmap))
                .route("/{prefix:_reader/.*}", web::get().to(test))
                .route("/{pdfs:.*/.*pdf}", web::get().to(index))
                // .service(index)
                // .service(test),
        );
    }
    {
        let mut dc_hashmap = HashMap::new();
        dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

        cfg.service(
            web::scope("/us/dc")
                .app_data(web::Data::new(dc_hashmap))
                // .service(test),
        );
    }
}
