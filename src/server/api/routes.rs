//! A central place to register App routes.
#![expect(
    clippy::exit,
    reason = "We exit with 1 error code on any application errors"
)]
use std::{process, sync::OnceLock};

use crate::server::api::state;
use crate::stelae::{stele::Stele, types::repositories::Repositories};
use actix_service::ServiceFactory;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
<<<<<<< HEAD
    guard, web, App, Error, Scope, Responder, route,
    HttpResponse
=======
    guard, web, App, Error, Scope, get, Responder,
<<<<<<< HEAD
    HttpResponse, HttpRequest
>>>>>>> ac49acd (Merging git and serve)
=======
    HttpResponse
>>>>>>> 6268aa5 (Updated path for stelae git and removed redundant code)
};
use serde::Deserialize;

use crate::utils::http::get_contenttype;
use crate::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};
use super::{serve::serve, state::Global, versions::versions};
use crate::utils::paths::clean_path;
use git2::{self, ErrorCode};
use super::state::App as AppState;

use super::super::errors::HTTPError;

/// Name of the header to guard current documents
static HEADER_NAME: OnceLock<String> = OnceLock::new();
/// Values of the header to guard current documents
static HEADER_VALUES: OnceLock<Vec<String>> = OnceLock::new();

/// Central place to register all the App routing.
///
/// Registers all routes for the given Archive
/// Static routes should be registered first, followed by dynamic routes.
///
/// # Errors
/// Errors if unable to register dynamic routes (e.g. if git repository cannot be opened)
#[tracing::instrument(skip(app, state))]
pub fn register_app<
    T: Global + Clone + 'static,
    U: MessageBody,
    V: ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<U>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
>(
    mut app: App<V>,
    state: &T,
) -> anyhow::Result<App<V>> {
    app = app
        .service(
            web::scope("/_api").service(
                web::scope("/versions")
                    .service(
                        web::resource("/_publication/{publication}/_compare/{date}/{compare_date}")
                            .to(versions),
                    )
                    .service(
                        web::resource(
                            "/_publication/{publication}/_compare/{date}/{compare_date}/{path:.*}",
                        )
                        .to(versions),
                    )
                    .service(web::resource("/_publication/{publication}/_date/{date}").to(versions))
                    .service(web::resource("/_publication/{publication}").to(versions))
                    .service(web::resource("/_publication/{publication}/{path:.*}").to(versions))
                    .service(web::resource("/_compare/{date}/{compare_date}").to(versions))
                    .service(
                        web::resource("/_compare/{date}/{compare_date}/{path:.*}").to(versions),
                    )
                    .service(web::resource("/_date/{date}").to(versions))
                    .service(web::resource("/_date/{date}/{path:.*}").to(versions))
                    .service(web::resource("/{path:.*}").to(versions))
                    .service(web::resource("").to(versions)),
            ),

        )
        .app_data(web::Data::new(state.clone()));

        app = app
        .service(
            web::scope("/_git").service(
                get_blob
            )
        )
        .app_data(web::Data::new(state.clone()));

    app = register_dynamic_routes(app, state)?;
    Ok(app)
}

/// Initialize all dynamic routes for the given Archive.
///
/// Dynamic routes are determined at runtime by looking at the stele's `dependencies.json` and `repositories.json` files
/// in the authentication (e.g. law) repository.
///
/// # Errors
/// Errors if unable to register dynamic routes (e.g. if git repository cannot be opened)
fn register_dynamic_routes<
    T: MessageBody,
    U: ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<T>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
>(
    mut app: App<U>,
    state: &impl Global,
) -> anyhow::Result<App<U>> {
    let config = state.archive().get_config()?;
    let stelae_guard = config
        .headers
        .and_then(|headers| headers.current_documents_guard);

    if let Some(guard) = stelae_guard {
        app = initialize_guarded_dynamic_routes(guard, app, state)?;
    } else {
        app = initialize_dynamic_routes(app, state)?;
    };
    Ok(app)
}

/// Initialize all guarded dynamic routes for the given Archive.
/// Routes are guarded by a header value specified in the config.toml file.
///
/// # Errors
/// Errors if unable to register dynamic routes (e.g. if git repository cannot be opened)
fn initialize_guarded_dynamic_routes<
    T: MessageBody,
    U: ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<T>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
>(
    guard: String,
    mut app: App<U>,
    state: &impl Global,
) -> anyhow::Result<App<U>> {
    tracing::info!(
        "Initializing guarded current documents with header: {}",
        guard
    );
    HEADER_NAME.get_or_init(|| guard);
    HEADER_VALUES.get_or_init(|| {
        state
            .archive()
            .stelae
            .keys()
            .map(ToString::to_string)
            .collect()
    });

    if let (Some(guard_name), Some(guard_values)) = (HEADER_NAME.get(), HEADER_VALUES.get()) {
        for guard_value in guard_values {
            let stele = state.archive().stelae.get(guard_value);
            if let Some(guarded_stele) = stele {
                let shared_state = state::init_shared(guarded_stele)?;
                let mut stelae_scope = web::scope("");
                stelae_scope = stelae_scope.guard(guard::Header(guard_name, guard_value));
                app = app.service(
                    stelae_scope
                        .app_data(web::Data::new(shared_state))
                        .configure(|cfg| {
                            register_root_routes(cfg, guarded_stele).unwrap_or_else(|_| {
                                tracing::error!(
                                    "Failed to initialize routes for Stele: {}",
                                    guarded_stele.get_qualified_name()
                                );
                                process::exit(1);
                            });
                        }),
                );
            }
        }
    } else {
        let err_msg = "Failed to initialize guarded routes. Header name or values not found.";
        tracing::error!(err_msg);
        anyhow::bail!(err_msg);
    }
    Ok(app)
}

/// Initialize all dynamic routes for the given Archive.
///
/// # Errors
/// Errors if unable to register dynamic routes (e.g. if git repository cannot be opened)
fn initialize_dynamic_routes<
    T: MessageBody,
    U: ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<T>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
>(
    mut app: App<U>,
    state: &impl Global,
) -> anyhow::Result<App<U>> {
    tracing::info!("Initializing app");
    let root = state.archive().get_root()?;
    let shared_state = state::init_shared(root)?;
    app = app.service(
        web::scope("")
            .app_data(web::Data::new(shared_state))
            .configure(|cfg| {
                register_routes(cfg, state).unwrap_or_else(|_| {
                    tracing::error!(
                        // TODO: error handling
                        "Failed to initialize routes for root Stele: {}",
                        root.get_qualified_name()
                    );
                    process::exit(1);
                });
            }),
    );
    Ok(app)
}

/// Registers all dynamic routes for the given Archive
/// Each current document routes consists of two dynamic segments: `{prefix}/{tail}`.
/// prefix: the first part of the request uri, used to determine which dependent Stele to serve.
/// tail: the remaining glob pattern path of the request uri.
/// # Arguments
/// * `cfg` - The Actix `ServiceConfig`
/// * `state` - The application state
/// # Errors
/// Will error if unable to register routes (e.g. if git repository cannot be opened)
#[expect(
    clippy::iter_over_hash_type,
    reason = "List of repositories that are registered as routes are always sorted, even with iterating over hash type"
)]
fn register_routes<T: Global>(cfg: &mut web::ServiceConfig, state: &T) -> anyhow::Result<()> {
    for stele in state.archive().stelae.values() {
        if let Some(repositories) = stele.repositories.as_ref() {
            if stele.is_root() {
                continue;
            }
            register_dependent_routes(cfg, stele, repositories)?;
        }
    }
    let root = state.archive().get_root()?;
    register_root_routes(cfg, root)?;
    Ok(())
}

/// Register routes for the root Stele
/// Root Stele is the Stele specified in config.toml
/// # Arguments
/// * `cfg` - The Actix `ServiceConfig`
/// * `stele` - The root Stele
/// # Errors
/// Will error if unable to register routes (e.g. if git repository cannot be opened)
fn register_root_routes(cfg: &mut web::ServiceConfig, stele: &Stele) -> anyhow::Result<()> {
    let mut root_scope: Scope = web::scope("");
    if let Some(repositories) = stele.repositories.as_ref() {
        let sorted_repositories = repositories.get_sorted();
        for repository in sorted_repositories {
            let custom = &repository.custom;
            let repo_state = state::init_repo(repository, stele)?;
            for route in custom.routes.iter().flat_map(|routes| routes.iter()) {
                let actix_route = format!("/{{tail:{}}}", &route);
                root_scope = root_scope.service(
                    web::resource(actix_route.as_str())
                        .route(web::get().to(serve))
                        .route(web::head().to(serve))
                        .app_data(web::Data::new(repo_state.clone())),
                );
            }
            if let Some(underscore_scope) = custom.scope.as_ref() {
                let actix_underscore_scope = web::scope(underscore_scope.as_str()).service(
                    web::scope("").service(
                        web::resource("/{tail:.*}")
                            .route(web::get().to(serve))
                            .route(web::head().to(serve))
                            .app_data(web::Data::new(repo_state.clone())),
                    ),
                );
                cfg.service(actix_underscore_scope);
            }
        }
        cfg.service(root_scope);
    }
    Ok(())
}

/// Register routes for dependent Stele
/// Dependent Stele are all Steles' specified in the root Stele's `dependencies.json` config file.
/// # Arguments
/// * `cfg` - The Actix `ServiceConfig`
/// * `stele` - The root Stele
/// * `repositories` - Data repositories of the dependent Stele
/// # Errors
/// Will error if unable to register routes (e.g. if git repository cannot be opened)
fn register_dependent_routes(
    cfg: &mut web::ServiceConfig,
    stele: &Stele,
    repositories: &Repositories,
) -> anyhow::Result<()> {
    let sorted_repositories = repositories.get_sorted();
    for scope in repositories.scopes.iter().flat_map(|scopes| scopes.iter()) {
        let scope_str = format!("/{{prefix:{}}}", &scope.as_str());
        let mut actix_scope = web::scope(scope_str.as_str());
        for repository in &sorted_repositories {
            let custom = &repository.custom;
            let repo_state = state::init_repo(repository, stele)?;
            for route in custom.routes.iter().flat_map(|routes| routes.iter()) {
                if route.starts_with('_') {
                    // Ignore routes in dependent Stele that start with underscore
                    // These routes are handled by the root Stele.
                    continue;
                }
                let actix_route = format!("/{{tail:{}}}", &route);
                actix_scope = actix_scope.service(
                    web::resource(actix_route.as_str())
                        .route(web::get().to(serve))
                        .route(web::head().to(serve))
                        .app_data(web::Data::new(repo_state.clone())),
                );
            }
        }
        cfg.service(actix_scope);
    }
    Ok(())
}

/// Structure for 
#[derive(Debug, Deserialize)]
struct Info {
    /// commit of the repo
    commitish: String,
    /// path of the file
    remainder: Option<String>,
}

/// Return the content in the stelae archive in the `{namespace}/{name}`
/// repo at the `commitish` commit at the `remainder` path.
/// Return 404 if any are not found or there are any errors.
<<<<<<< HEAD
<<<<<<< HEAD
#[route(
    "/{namespace}/{name}/{commitish}{remainder:/+([^{}]*?)?/*}",
    method = "GET",
    method = "HEAD"
)]
=======
#[get("/{namespace}/{name}/ref_{commitish:.*}_/{remainder}")]//:/+([^{}]*?)?/*}")]
>>>>>>> ac49acd (Merging git and serve)
#[tracing::instrument(name = "Retrieving a Git blob", skip(path, data))]
=======
#[get("/{namespace}/{name}")]//:/+([^{}]*?)?/*}")]
#[tracing::instrument(name = "Retrieving a Git blob", skip(path, data, info))]
>>>>>>> 6268aa5 (Updated path for stelae git and removed redundant code)
async fn get_blob(
    path: web::Path<(String, String)>,
    info: web::Query<Info>,
    data: web::Data<AppState>
) -> impl Responder {
    let (namespace, name/* , commitish, remainder*/) = path.into_inner();
    let info_struct: Info = info.into_inner();
    let commitish = info_struct.commitish;
    let remainder = info_struct.remainder.unwrap_or_else(|| "".to_string());
    let archive_path = &data.archive_path;
    let blob = Repo::find_blob(archive_path, &namespace, &name, &remainder, &commitish);
    let blob_path = clean_path(&remainder);
    let contenttype = get_contenttype(&blob_path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(error) => blob_error_response(&error, &namespace, &name),
    }
}

/// A centralised place to match potentially unsafe internal errors to safe user-facing error responses
#[allow(clippy::wildcard_enum_match_arm)]
#[tracing::instrument(name = "Error with Git blob request", skip(error, namespace, name))]
fn blob_error_response(error: &anyhow::Error, namespace: &str, name: &str) -> HttpResponse {
    tracing::error!("{error}",);
    if let Some(git_error) = error.downcast_ref::<git2::Error>() {
        return match git_error.code() {
            // TODO: check this is the right error
            ErrorCode::NotFound => {
                HttpResponse::NotFound().body(format!("repo {namespace}/{name} does not exist"))
            }
            _ => HttpResponse::InternalServerError().body("Unexpected Git error"),
        };
    }
    match error {
        // TODO: Obviously it's better to use custom `Error` types
        _ if error.to_string() == GIT_REQUEST_NOT_FOUND => {
            HttpResponse::NotFound().body(HTTPError::NotFound.to_string())
        }
        _ => HttpResponse::InternalServerError().body(HTTPError::InternalServerError.to_string()),
    }
}