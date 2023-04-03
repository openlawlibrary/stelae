//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::server::tracing::StelaeRootSpanBuilder;
use crate::stelae::archive::Archive;
use actix_web::{get, web, App, HttpRequest, HttpServer, Route, Scope};
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


async fn default() -> &'static str {
    "Default"
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

    let archive =
        Archive::parse(archive_path, PathBuf::from(raw_archive_path)).unwrap_or_else(|_| {
            tracing::error!("Unable to parse archive at '{raw_archive_path}'.");
            std::process::exit(1);
        });
    let mut state = AppState { archive };
    //TODO: root stele is a stele from which we began serving the archive
    // let root = state.archive.get_root().unwrap_or_else(|_| {
    //     tracing::error!("Unable to determine root Stele.");
    //     std::process::exit(1);
    // });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
            .app_data(web::Data::new(state.clone()))
            .configure(|cfg| init_routes(cfg, state.clone()))
    })
    .bind((bind, port))?
    .run()
    .await
}

/// Routes
fn init_routes(cfg: &mut web::ServiceConfig, state: AppState) {
    // let mut scopes: Vec<Scope> = vec![];
    // for stele in state.archive.stelae.values() {
    //     // dbg!(&stele.path);
    //     if let Some(repositories) = &stele.repositories {
    //         // dbg!(&repositories);
    //         for scope in repositories.scopes.iter().flat_map(|s| s.iter()) {
    //             // dbg!(&scope);
    //             // let scope = web::scope(scope.as_str());
    //             for repository in repositories.repositories.values() {
    //                 let custom = &repository.custom;
    //                 for route in custom.routes.iter().flat_map(|r| r.iter()) {
    //                     //ignore routes that start with underscore
    //                     if route.starts_with("~ _") {
    //                         continue;
    //                     }
    //                     let actix_route = format!("/{{prefix:{}}}", &route);
    //                     dbg!(&actix_route);
    //                     dbg!(&scope);
    //                     let actix_scope = web::scope(scope.as_str())
    //                         .route(actix_route.as_str(), web::get().to(default));
    //                     scopes.push(actix_scope);
    //                 }
    //             }
    //         }
    //     }
    // }
    // for scope in scopes {
    //     cfg.service(scope);
    // }

    {
        let mut smc_hashmap = HashMap::new();
        smc_hashmap.insert("cityofsanmateo".to_owned(), "some value for SMC".to_owned());
        let mut dc_hashmap = HashMap::new();
        dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

        cfg.service(
            web::scope("/us/ca/cities/san-mateo")
                .service(web::resource("/{prefix:_reader/.*}")
                // .route("/{prefix:_reader/.*}", web::get().to(test))
                .app_data(web::Data::new(smc_hashmap))
                .route(web::get().to(test)))
                // .route("/{pdfs:.*/.*pdf}", web::get().to(test))
                // .app_data(web::Data::new(dc_hashmap))
                .service(web::resource("/{pdfs:.*/.*pdf}").app_data(web::Data::new(dc_hashmap)).route(web::get().to(test))), // .service(index)
                                                                                                 // .service(test),
        );
    }
    // {
    //     let mut dc_hashmap = HashMap::new();
    //     dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

    //     cfg.service(
    //         web::scope("/us/dc").app_data(web::Data::new(dc_hashmap)), // .service(test),
    //     );
    // }
}
