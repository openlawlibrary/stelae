//! Running the CLI

// Allow exits because in this file we ideally handle all errors with known exit codes
#![allow(clippy::exit)]

use crate::history::changes;
use crate::server::app::serve_archive;
use crate::server::errors::CliError;
use crate::server::git::serve_git;
use crate::utils::archive::find_archive_path;
use clap::Parser;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use tracing;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

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
    /// Update the archive
    ///
    /// NOTE: Once TAF is embedded with stelae, this command will be used to update the repositories within the archive.
    /// Currently inserts historical information about the Steles in the archive.
    ///
    ///  - Populates the database with change objects loaded in from RDF repository
    ///  - By default inserts historical information for the root and all referenced stele in the archive
    Update,
}

/// Place to initialize tracing
///
/// We create `debug` and `error` log files in `.stelae` dir.
/// `debug` log file contains all logs, `error` log file contains only `warn` and `error`
/// NOTE: once `https://github.com/tokio-rs/tracing/pull/2497` is merged,
/// update `init_tracing` to rotate log files based on size.
#[allow(clippy::expect_used)]
fn init_tracing(archive_path: &Path) {
    let stelae_dir = archive_path.join(PathBuf::from("./.stelae"));

    let debug_file_appender =
        rolling::never(&stelae_dir, "stelae-debug.log").with_max_level(Level::DEBUG);
    let error_file_appender =
        rolling::never(&stelae_dir, "stelae-error.log").with_max_level(Level::WARN);

    let mut debug_layer = fmt::layer().with_writer(debug_file_appender);
    let mut error_layer = fmt::layer().with_writer(error_file_appender);

    // disable ANSI colors.
    // this is to avoid color coding in log files and to make it easier to read.
    debug_layer = debug_layer.with_ansi(false);
    error_layer = error_layer.with_ansi(false);
    // also log to console
    let console_layer = fmt::layer().with_target(true).with_filter(
        EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("Failed to initialize console logging"),
    );

    tracing_subscriber::registry()
        .with(debug_layer)
        .with(error_layer)
        .with(console_layer)
        .init();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
}

/// Central place to execute commands
///
/// # Errors
/// This function returns the generic `CliError`, based on which we exit with a known exit code.
fn execute_command(cli: &Cli, archive_path: PathBuf) -> Result<(), CliError> {
    match cli.subcommands {
        Subcommands::Git { port } => serve_git(&cli.archive_path, archive_path, port),
        Subcommands::Serve { port, individual } => {
            serve_archive(&cli.archive_path, archive_path, port, individual)
        }
        Subcommands::Update => changes::insert(&cli.archive_path, archive_path),
    }
}

/// Main entrypoint to application
///
/// Exits with 1 if we encounter an error
pub fn run() {
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

    init_tracing(&archive_path);

    match execute_command(&cli, archive_path) {
        Ok(()) => process::exit(0),
        Err(err) => {
            // Exit with 1 if we encounter an error
            tracing::error!("Application error: {err:?}");
            process::exit(1);
        }
    }
}
