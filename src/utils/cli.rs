//! Running the CLI

// Allow exits because in this file we ideally handle all errors with known exit codes
#![allow(clippy::exit)]

use crate::server::git::serve_git;
use crate::utils::library::find_library_path;
use clap::Parser;
use std::path::Path;

/// Stele is currently just a simple git server.
/// run from the library directory or pass
/// path to library.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Stele library. Defaults to cwd.
    #[arg(short, long, default_value_t = String::from(".").to_owned())]
    library_path: String,
    /// Stele cli subcommands
    #[command(subcommand)]
    subcommands: Subcommands,
}

///
#[derive(Clone, clap::Subcommand)]
enum Subcommands {
    /// Serve git repositories in the Stele library
    Git {
        /// Port on which to serve the library.
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

/// Main entrypoint to application
///
/// # Errors
/// TODO: This function should not return errors
#[allow(clippy::print_stdout)]
#[inline]
pub fn run() -> std::io::Result<()> {
    let cli = Cli::parse();
    let library_path_wd = Path::new(&cli.library_path);
    let library_path = if let Ok(lpath) = find_library_path(library_path_wd) {
        lpath
    } else {
        println!(
            "error: could not find `.stele` folder in `{}` or any parent directory",
            &cli.library_path
        );
        std::process::exit(1);
    };

    match cli.subcommands {
        Subcommands::Git { port } => serve_git(&cli.library_path, library_path, port),
    }
}
