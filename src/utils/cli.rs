//! Running the CLI
#![expect(
    clippy::exit,
    reason = "Allow exits because in this file we ideally handle all errors with known exit codes"
)]

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
use tracing_subscriber::fmt::writer::MakeWriterExt as _;
use tracing_subscriber::Layer as _;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

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
    Update {
        /// List of stelae to include in update
        #[arg(short = 'i', long = "include", num_args(1..))]
        include: Vec<String>,
        /// List of stelae to exclude in update
        #[arg(short = 'e', long = "exclude", num_args(1..))]
        exclude: Vec<String>,
    },
}

/// Place to initialize tracing
///
/// We create `debug` and `error` log files in `.taf` dir.
/// `debug` log file contains all logs, `error` log file contains only `warn` and `error`
/// NOTE: once `https://github.com/tokio-rs/tracing/pull/2497` is merged,
/// update `init_tracing` to rotate log files based on size.
#[expect(
    clippy::expect_used,
    reason = "Expect that console logging can be initialized"
)]
fn init_tracing(archive_path: &Path) {
    let taf_dir = archive_path.join(PathBuf::from("./.taf"));

    let debug_file_appender =
        rolling::never(&taf_dir, "stelae-debug.log").with_max_level(Level::DEBUG);
    let error_file_appender =
        rolling::never(&taf_dir, "stelae-error.log").with_max_level(Level::WARN);

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

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Matching on a reference (&cli.subcommands) instead of by value; the match patterns borrow fields, which is intentional to avoid moving data."
)]
/// Central place to execute commands
///
/// # Errors
/// This function returns the generic `CliError`, based on which we exit with a known exit code.
fn execute_command(cli: &Cli, archive_path: PathBuf) -> Result<(), CliError> {
    match &cli.subcommands {
        Subcommands::Git { port } => serve_git(&cli.archive_path, archive_path, *port),
        Subcommands::Serve { port, individual } => {
            serve_archive(&cli.archive_path, archive_path, *port, *individual)
        }
        Subcommands::Update { include, exclude } => {
            changes::insert(&cli.archive_path, archive_path, include, exclude)
        }
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
            "error: could not find `.taf` folder in `{}` or any parent directory",
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
