//! Just main(). Keep as small as possible.

// The `main.rs` file is special in Rust.
// So attributes here have no affect on the main codebase. If the file remains minimal we can just
// blanket allow lint groups.
#![allow(clippy::cargo)]
#![allow(clippy::restriction)]

use stelae::utils::cli::run;

fn main() {
    run()
}
