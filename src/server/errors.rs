#![allow(
    // derive_more doesn't respect these lints
    clippy::pattern_type_mismatch,
    clippy::use_self
)]

//! Stelae-specific errors

use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use std::io;

/// Collection of possible HTTP errors
#[derive(Debug, Display, Error)]
pub enum HTTPError {
    #[display(fmt = "404 Not Found")]
    /// 404
    NotFound,
    #[display(fmt = "Unexpected server error")]
    /// 500
    InternalServerError,
}

/// Collection of possible CLI errors
#[derive(Debug, Display, Error)]
pub enum CliError {
    /// Database connection error
    #[display(fmt = "Failed to connect to the database")]
    DatabaseConnectionError,
    /// A generic fallback error
    #[display(fmt = "A CLI error occurred")]
    GenericError,
    /// Errors during archive parsing
    #[display(fmt = "Failed to parse the archive ")]
    ArchiveParseError,
}

impl From<io::Error> for CliError {
    fn from(_error: io::Error) -> Self {
        CliError::GenericError
    }
}

/// Collection of possible stelae web-facing errors
#[derive(Debug, Display, Error)]
pub enum StelaeError {
    /// Errors generated by the Git server
    #[display(fmt = "A Git server occurred")]
    GitError,
}

#[allow(clippy::missing_trait_methods)]
impl error::ResponseError for StelaeError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::GitError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
