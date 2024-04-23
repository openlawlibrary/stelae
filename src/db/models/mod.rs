//! This module contains all the sqlx structs for the database tables.

/// sqlx structs for `changed_library_document` table.
pub mod changed_library_document;
/// sqlx structs for `document` table.
pub mod document;
/// sqlx structs for `document_change` table.
pub mod document_change;
/// sqlx structs for `library` table.
pub mod library;
/// sqlx structs for `library_change` table.
pub mod library_change;
/// sqlx structs for `publication` table.
pub mod publication;
/// sqlx structs for `publication_has_publication_versions` table.
pub mod publication_has_publication_versions;
/// sqlx structs for `publication_version` table
pub mod publication_version;
/// sqlx structs for `stele` table.
pub mod stele;
