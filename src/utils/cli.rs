//! Running the CLI

// Allow exits because in this file we ideally handle all errors with known exit codes
#![allow(clippy::exit)]

use crate::history::changes;
use crate::server::app::serve_archive;
use crate::server::git::serve_git;
use crate::utils::archive::find_archive_path;
use clap::Parser;
use std::env;
use std::io;
use std::path::Path;
use std::process;
use tracing;
use tracing_subscriber::fmt;

/// Stelae is currently just a simple git server.
/// run from the library directory or pass
/// path to archive.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Stelae archive. Defaults to cwd.
    #[arg(short, long, default_value_t = String::from(".").to_owned())]
    archive_path: String,
    /// Stelae cli subcommands
    #[command(subcommand)]
    subcommands: Subcommands,
}

/// Subcommands for the Stelae CLI
#[derive(Clone, clap::Subcommand)]
enum Subcommands {
    /// Serve git repositories in the Stelae archive
    Git {
        /// Port on which to serve the archive.
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Serve documents in a Stelae archive.
    Serve {
        /// Port on which to serve the archive.
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        #[arg(short, long, default_value_t = false)]
        /// Serve an individual stele instead of the Stele specified in config.toml.
        individual: bool,
    },
    /// Insert historical information about the Steles in the archive.
    /// Populates the database with change objects loaded in from RDF repository
    /// By default inserts historical information for the root Stele (and all referenced stele) in the archive
    InsertHistory {
        /// Optionally insert historical information for this Stele only.
        stele: Option<String>,
    },
}

/// Place to initialize tracing
fn init_tracing() {
    fmt::init();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
}

/// Main entrypoint to application
///
/// # Errors
/// TODO: This function should not return errors
pub fn run() -> io::Result<()> {
    init_tracing();
    tracing::debug!("Starting application");
    let cli = Cli::parse();
    let archive_path_wd = Path::new(&cli.archive_path);
    let Ok(archive_path) = find_archive_path(archive_path_wd) else {
        tracing::error!(
            "error: could not find `.stelae` folder in `{}` or any parent directory",
            &cli.archive_path
        );
        process::exit(1);
    };

    match cli.subcommands {
        Subcommands::Git { port } => serve_git(&cli.archive_path, archive_path, port),
        Subcommands::Serve { port, individual } => {
            serve_archive(&cli.archive_path, archive_path, port, individual)
        }
        Subcommands::InsertHistory { stele } => {
            tracing::info!("Inserting history into archive");
            changes::insert(&cli.archive_path, archive_path, stele)
        }
    }
}
