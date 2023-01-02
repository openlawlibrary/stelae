//! Running the CLI

// Allow exits because in this file we ideally handle all errors with known exit codes
#![allow(clippy::exit)]

use crate::server::git::serve_git;
use crate::utils::archive::find_archive_path;
use clap::Parser;
use std::path::Path;
use tracing;

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

///
#[derive(Clone, clap::Subcommand)]
enum Subcommands {
    /// Serve git repositories in the Stelae archive
    Git {
        /// Port on which to serve the archive.
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

///
fn init_tracing() {
    tracing_subscriber::fmt::init();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

/// Main entrypoint to application
///
/// # Errors
/// TODO: This function should not return errors
pub fn run() -> std::io::Result<()> {
    init_tracing();
    tracing::debug!("Starting application");
    let cli = Cli::parse();
    let archive_path_wd = Path::new(&cli.archive_path);
    let Ok(archive_path) = find_archive_path(archive_path_wd) else {
        tracing::error!(
            "error: could not find `.stelae` folder in `{}` or any parent directory",
            &cli.archive_path
        );
        std::process::exit(1);
    };

    match cli.subcommands {
        Subcommands::Git { port } => serve_git(&cli.archive_path, archive_path, port),
    }
}
