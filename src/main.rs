//! Stele commandline.
#![allow(clippy::self_named_module_files)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::implicit_return)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::exhaustive_structs)]

use clap::Parser;
use std::path::Path;
use stele::server::git::serve_git;
use stele::utils::library::find_library_path;

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

#[allow(clippy::print_stdout)]
fn main() -> std::io::Result<()> {
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
