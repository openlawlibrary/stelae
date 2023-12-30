//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::stelae::archive::Archive;
use crate::stelae::types::repositories::Repository;
use crate::utils::archive::get_name_parts;
use crate::utils::git::Repo;
use crate::utils::http::get_contenttype;
use crate::{server::tracing::StelaeRootSpanBuilder, stelae::stele::Stele};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{guard, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Scope};
use git2::Repository as GitRepository;
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, path::PathBuf};
use tracing_actix_web::TracingLogger;

use actix_http::body::MessageBody;
use actix_service::ServiceFactory;
use std::sync::OnceLock;

/// Name of the header to guard current documents
static HEADER_NAME: OnceLock<String> = OnceLock::new();
/// Values of the header to guard current documents
static HEADER_VALUES: OnceLock<Vec<String>> = OnceLock::new();

/// Most-recent git commit
const HEAD_COMMIT: &str = "HEAD";

#[allow(clippy::expect_used)]
/// Remove leading and trailing `/`s from the `path` string.
fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}

/// Global, read-only state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Fully initialized Stelae archive
    pub archive: Archive,
}

/// Git repository to serve
struct RepoState {
    /// git2 repository pointing to the repo in the archive.
    repo: Repo,
    ///Latest or historical
    serve: String,
}

/// Shared, read-only app state
pub struct SharedState {
    /// Repository to fall back to if the current one is not found
    fallback: Option<RepoState>,
}

impl fmt::Debug for RepoState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo for {} in the archive at {}",
            self.repo.name,
            self.repo.path.display()
        )
    }
}

impl fmt::Debug for SharedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fb = &self.fallback;
        match *fb {
            Some(ref fallback) => write!(
                f,
                "Repo for {} in the archive at {}",
                fallback.repo.name,
                fallback.repo.path.display()
            ),
            None => write!(f, "No fallback repo"),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Clone for RepoState {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            serve: self.serve.clone(),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self {
            fallback: self.fallback.clone(),
        }
    }
}

#[allow(clippy::future_not_send)]
/// Serve current document
async fn serve(
    req: HttpRequest,
    shared: web::Data<SharedState>,
    data: web::Data<RepoState>,
) -> impl Responder {
    let prefix = req
        .match_info()
        .get("prefix")
        .unwrap_or_default()
        .to_owned();
    let tail = req.match_info().get("tail").unwrap_or_default().to_owned();
    let mut path = format!("{prefix}/{tail}");
    path = clean_path(&path);
    let blob = data.repo.get_bytes_at_path(HEAD_COMMIT, &path);
    let contenttype = get_contenttype(&path);
    if let Ok(content) = blob {
        HttpResponse::Ok().insert_header(contenttype).body(content)
    } else if let Some(ref fallback) = shared.fallback {
        let fallback_blob = fallback.repo.get_bytes_at_path(HEAD_COMMIT, &path);
        fallback_blob.map_or_else(
            |_| HttpResponse::BadRequest().into(),
            |content| HttpResponse::Ok().insert_header(contenttype).body(content),
        )
    } else {
        HttpResponse::BadRequest().into()
    }
}

/// Serve documents in a Stelae archive.
#[actix_web::main]
pub async fn serve_archive(
    raw_archive_path: &str,
    archive_path: PathBuf,
    port: u16,
    individual: bool,
) -> std::io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    let archive = Archive::parse(archive_path, &PathBuf::from(raw_archive_path), individual)
        .unwrap_or_else(|err| {
            tracing::error!("Unable to parse archive at '{raw_archive_path}'.");
            tracing::error!("Error: {:?}", err);
            std::process::exit(1);
        });
    let state = AppState { archive };

    HttpServer::new(move || {
        init_app(&state).unwrap_or_else(|err| {
            tracing::error!("Unable to initialize app.");
            tracing::error!("Error: {:?}", err);
            std::process::exit(1);
        })
    })
    .bind((bind, port))?
    .run()
    .await
}

/// Initialize the application
///
/// # Arguments
/// * `state` - The application state
/// # Errors
/// Will error if unable to initialize the application
pub fn init_app(
    state: &AppState,
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
    let config = state.archive.get_config()?;
    let stelae_guard = config
        .headers
        .and_then(|headers| headers.current_documents_guard);

    stelae_guard.map_or_else(
        || {
            tracing::info!("Initializing app");
            let root = state.archive.get_root()?;
            let shared_state = init_shared_app_state(root)?;
            Ok(App::new().service(
                web::scope("")
                    .app_data(web::Data::new(shared_state))
                    .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
                    .configure(|cfg| {
                        init_routes(cfg, state, root).unwrap_or_else(|_| {
                            tracing::error!(
                                "Failed to initialize routes for root Stele: {}",
                                root.get_qualified_name()
                            );
                            std::process::exit(1);
                        });
                    }),
            ))
        },
        |guard| {
            tracing::info!("Guarding current documents with header: {}", guard);
            HEADER_NAME.get_or_init(|| guard);
            HEADER_VALUES.get_or_init(|| {
                state
                    .archive
                    .stelae
                    .keys()
                    .map(ToString::to_string)
                    .collect()
            });

            let mut app = App::new();
            if let (Some(guard_name), Some(guard_values)) = (HEADER_NAME.get(), HEADER_VALUES.get())
            {
                for guard_value in guard_values {
                    let stele = state.archive.stelae.get(guard_value);
                    if let Some(guarded_stele) = stele {
                        let shared_state = init_shared_app_state(guarded_stele)?;
                        let mut stelae_scope = web::scope("");
                        stelae_scope = stelae_scope.guard(guard::Header(guard_name, guard_value));
                        app = app.service(
                            stelae_scope
                                .app_data(web::Data::new(shared_state))
                                .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
                                .configure(|cfg| {
                                    init_routes_single_stele(cfg, guarded_stele).unwrap_or_else(
                                        |_| {
                                            tracing::error!(
                                                "Failed to initialize routes for Stele: {}",
                                                guarded_stele.get_qualified_name()
                                            );
                                            std::process::exit(1);
                                        },
                                    );
                                }),
                        );
                    }
                }
            }
            Ok(app)
        },
    )
}

/// Init Actix routing for a single Stele
fn init_routes_single_stele(cfg: &mut web::ServiceConfig, stele: &Stele) -> anyhow::Result<()> {
    let mut root_scope: Scope = web::scope("");
    stele.repositories.as_ref().map_or_else(
        || {
            tracing::debug!("No data repositories found in Stele. Skipping initializing routes.");
            Ok::<(), anyhow::Error>(())
        },
        |repositories| {
            let sorted_repositories = repositories.get_sorted_repositories();
            for repository in &sorted_repositories {
                let custom = &repository.custom;
                let repo_state = init_repo_state(repository, stele)?;
                for route in custom.routes.iter().flat_map(|r| r.iter()) {
                    let actix_route = format!("/{{tail:{}}}", &route);
                    root_scope = root_scope.service(
                        web::resource(actix_route.as_str())
                            .route(web::get().to(serve))
                            .app_data(web::Data::new(repo_state.clone())),
                    );
                }
                if let &Some(ref underscore_scope) = &custom.scope {
                    let actix_underscore_scope = web::scope(underscore_scope.as_str()).service(
                        web::scope("").service(
                            web::resource("/{tail:.*}")
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        ),
                    );
                    cfg.service(actix_underscore_scope);
                }
            }
            cfg.service(root_scope);
            Ok(())
        },
    )
}

/// Initialize the data repository used in the Actix route
/// Each Actix route has its own data repository
///
/// # Errors
/// Will error if unable to initialize the data repository
fn init_repo_state(repo: &Repository, stele: &Stele) -> anyhow::Result<RepoState> {
    let name = &repo.name;
    let custom = &repo.custom;
    let mut repo_path = stele.archive_path.to_string_lossy().into_owned();
    repo_path = format!("{repo_path}/{name}");
    Ok(RepoState {
        repo: Repo {
            archive_path: stele.archive_path.to_string_lossy().to_string(),
            path: PathBuf::from(&repo_path),
            org: stele.auth_repo.org.clone(),
            name: name.clone(),
            repo: GitRepository::open(&repo_path)?,
        },
        serve: custom.serve.clone(),
    })
}

/// Routes
fn init_routes(cfg: &mut web::ServiceConfig, state: &AppState, root: &Stele) -> anyhow::Result<()> {
    let mut scopes: Vec<Scope> = vec![];
    let mut root_scope: Scope = web::scope("");

    // TODO: this has to get moved to a function
    for stele in state.archive.stelae.values() {
        if let &Some(ref repositories) = &stele.repositories {
            // Root Stele
            let sorted_repositories = repositories.get_sorted_repositories();
            if stele.get_qualified_name() == root.get_qualified_name() {
                for repository in &sorted_repositories {
                    let custom = &repository.custom;
                    let repo_state = init_repo_state(repository, stele)?;
                    for route in custom.routes.iter().flat_map(|r| r.iter()) {
                        let actix_route = format!("/{{tail:{}}}", &route);
                        root_scope = root_scope.service(
                            web::resource(actix_route.as_str())
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        );
                    }
                    if let &Some(ref underscore_scope) = &custom.scope {
                        let actix_underscore_scope = web::scope(underscore_scope.as_str()).service(
                            web::scope("").service(
                                web::resource("/{tail:.*}")
                                    .route(web::get().to(serve))
                                    .app_data(web::Data::new(repo_state.clone())),
                            ),
                        );
                        scopes.push(actix_underscore_scope);
                    }
                }
                continue;
            }
            //Child Stele
            for scope in repositories.scopes.iter().flat_map(|s| s.iter()) {
                let scope_str = format!("/{{prefix:{}}}", &scope.as_str());
                let mut actix_scope = web::scope(scope_str.as_str());
                for repository in &sorted_repositories {
                    let custom = &repository.custom;
                    let repo_state = init_repo_state(repository, stele)?;
                    for route in custom.routes.iter().flat_map(|r| r.iter()) {
                        if route.starts_with('_') {
                            // Ignore routes in child stele that start with underscore
                            continue;
                        }
                        let actix_route = format!("/{{tail:{}}}", &route);
                        actix_scope = actix_scope.service(
                            web::resource(actix_route.as_str())
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        );
                    }
                }
                scopes.push(actix_scope);
            }
        }
    }
    for scope in scopes {
        cfg.service(scope);
    }
    // Register root stele scope last
    cfg.service(root_scope);
    Ok(())
}

/// Initialize the shared application state
/// Currently shared application state consists of:
///     - fallback: used as a data repository to resolve data when no other url matches the request
/// # Returns
/// Returns a `SharedState` object
/// # Errors
/// Will error if unable to open the git repo for the fallback data repository
pub fn init_shared_app_state(stele: &Stele) -> anyhow::Result<SharedState> {
    let fallback = stele
        .get_fallback_repo()
        .map(|repo| {
            let (org, name) = get_name_parts(&repo.name)?;
            Ok::<RepoState, anyhow::Error>(RepoState {
                repo: Repo::new(&stele.archive_path, &org, &name)?,
                serve: repo.custom.serve.clone(),
            })
        })
        .transpose()?;
    Ok(SharedState { fallback })
}
