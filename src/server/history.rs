//! Handlers for serving historical documents.
#![allow(clippy::future_not_send)]
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::stelae::archive::Archive;

use super::publish::{AppState, GlobalState};

/// Request for the versions endpoint.
#[derive(Deserialize, Debug)]
pub struct VersionRequest {
    /// Publication name.
    pub publication: Option<String>,
    /// Date to compare.
    pub date: Option<String>,
    /// Date to compare against.
    pub compare_date: Option<String>,
    /// Path to document/collection.
    pub path: Option<String>,
}

/// Handler for the versions endpoint.
pub async fn versions(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let stele = match get_stele_from_request(&req, data.archive()) {
        Ok(stele) => stele,
        Err(err) => return HttpResponse::BadRequest().body(format!("Error: {err}")),
    };
    HttpResponse::Ok().body(format!("Hello world! - {stele}"))
}

/// Extracts the stele from the request.
/// If the `X-Stelae` header is present, it will return the value of the header.
/// Otherwise, it will return the root stele.
fn get_stele_from_request(req: &HttpRequest, archive: &Archive) -> anyhow::Result<String> {
    let req_headers = req.headers();
    let stele = archive.get_root()?.get_qualified_name();

    req_headers.get("X-Stelae").map_or_else(
        || Ok(stele),
        |value| {
            value.to_str().map_or_else(
                |_| anyhow::bail!("Invalid X-Stelae header value"),
                |str| Ok(str.to_owned()),
            )
        },
    )
}
