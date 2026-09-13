//! The Fonds module contains tools for interacting with Archives,
//! Fonds, and Repositories.

pub mod archive;
#[expect(
    clippy::module_inception,
    reason = "`fonds` is both singular and plural, so there's no better name for the submodule housing the `Fonds` struct"
)]
pub mod fonds;
pub mod types;
