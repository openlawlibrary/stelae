//! This module contains all the sqlx structs for the database tables.

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

/// module for interacting with `auth_commits` table.
pub mod auth_commits;
/// module for interacting with the `changed_library_document` table.
pub mod changed_library_document;
/// module for interacting with the `data_repos` table.
pub mod data_commits;
/// module for interacting with the `document` table.
pub mod document;
/// module for interacting with the `document_change` table.
pub mod document_change;
/// module for interacting with the `document_element` table.
pub mod document_element;
/// module for interacting with the `library` table.
pub mod library;
/// module for interacting with the `library_change` table.
pub mod library_change;
/// module for interacting with the `publication` table.
pub mod publication;
/// module for interacting with the `publication_has_publication_versions` table.
pub mod publication_has_publication_versions;
/// module for interacting with the `publication_version` table
pub mod publication_version;
/// module for the document or library status utility.
pub mod status;
/// module for interacting with the `stele` table.
pub mod stele;
/// module for interacting with the `version` table.
pub mod version;
