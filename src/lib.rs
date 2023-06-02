//! # Stelae
//!
//! Stelae is a collection of tools in rust and python for preserving,
//! authenticating, and accessing laws in perpetuity.
//!
//! ## About the name
//!
//! Stelae, or large stone slabs, were used in some ancient cultures to
//! publish the law. The Code of Hammurabi, one of the earliest preserved
//! written laws, was published on a Stelae in ~1750 BCE and is still readable
//! nearly four millennia later.

// =========================================================================
//                  Canonical lints for whole crate
// =========================================================================
// Official docs:
//   https://doc.rust-lang.org/nightly/clippy/lints.html
// Useful app to lookup full details of individual lints:
//   https://rust-lang.github.io/rust-clippy/master/index.html
//
// We set base lints to give the fullest, most pedantic feedback possible.
// Though we prefer that they are just warnings during development so that build-denial
// is only enforced in CI.
//
#![warn(
    // `clippy::all` is already on by default. It implies the following:
    //   clippy::correctness code that is outright wrong or useless
    //   clippy::suspicious code that is most likely wrong or useless
    //   clippy::complexity code that does something simple but in a complex way
    //   clippy::perf code that can be written to run faster
    //   clippy::style code that should be written in a more idiomatic way
    clippy::all,

    // It's always good to write as much documentation as possible
    missing_docs,

    // > clippy::pedantic lints which are rather strict or might have false positives
    clippy::pedantic,

    // > new lints that are still under development"
    // (so "nursery" doesn't mean "Rust newbies")
    clippy::nursery,

    // > The clippy::cargo group gives you suggestions on how to improve your Cargo.toml file.
    // > This might be especially interesting if you want to publish your crate and are not sure
    // > if you have all useful information in your Cargo.toml.
    clippy::cargo
)]
// > The clippy::restriction group will restrict you in some way.
// > If you enable a restriction lint for your crate it is recommended to also fix code that
// > this lint triggers on. However, those lints are really strict by design and you might want
// > to #[allow] them in some special cases, with a comment justifying that.
#![allow(clippy::blanket_clippy_restriction_lints)]
#![warn(clippy::restriction)]
//
//
// =========================================================================
//   Individually blanket-allow single lints relevant to this whole crate
// =========================================================================
#![allow(
    // This is idiomatic Rust
    clippy::implicit_return,

    // Multiple deps are currently pinning `hermit-abi` — December 2022
    clippy::multiple_crate_versions,

    // We're not interested in becoming no-std compatible
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,

    // TODO: But I think the mod.rs is more conventional — @tombh
    clippy::mod_module_files,

    // Although performance is of course important for this application, it is not currently
    // such that it would benefit from explicit inline suggestions. Besides, not specifying
    // `#[inline]` doesn't mean that a function won't be inlined. And if performance does start
    // to become a problem, there are other avenues to explore before deciding on which functions
    // would benefit from explicit inlining.
    clippy::missing_inline_in_public_items,

    // I think marking `#[non_exhaustive]` is more for structs/enums that are imported into other crates
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::question_mark_used,
    clippy::semicolon_outside_block,
)]

pub mod server;
pub mod utils;
