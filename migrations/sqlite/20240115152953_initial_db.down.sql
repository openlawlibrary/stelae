-- Add down migration script here
PRAGMA foreign_keys = OFF;

DROP INDEX IF EXISTS changed_library_document_library_mpath_idx;
DROP INDEX IF EXISTS library_change_library_mpath_idx;
DROP INDEX IF EXISTS document_change_doc_mpath_idx;

DROP TABLE IF EXISTS changed_library_document;
DROP TABLE IF EXISTS library_change;
DROP TABLE IF EXISTS document_change;
DROP TABLE IF EXISTS publication_has_publication_versions;
DROP TABLE IF EXISTS publication_version;
DROP TABLE IF EXISTS publication;
DROP TABLE IF EXISTS version;
DROP TABLE IF EXISTS library_document;
DROP TABLE IF EXISTS library;
DROP TABLE IF EXISTS document;
DROP TABLE IF EXISTS stele;

PRAGMA optimize;