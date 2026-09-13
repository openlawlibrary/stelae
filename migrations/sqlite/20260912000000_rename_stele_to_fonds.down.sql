-- Add down migration script here
PRAGMA foreign_keys = OFF;

DROP INDEX IF EXISTS data_repo_commits_publication_id_idx;
DROP INDEX IF EXISTS publication_fonds_idx;
DROP INDEX IF EXISTS library_fonds_idx;
DROP INDEX IF EXISTS document_element_fonds_idx;

ALTER TABLE publication RENAME COLUMN fonds TO stele;
ALTER TABLE library RENAME COLUMN fonds TO stele;
ALTER TABLE document_element RENAME COLUMN fonds TO stele;
ALTER TABLE fonds RENAME TO stele;

PRAGMA foreign_keys = ON;
PRAGMA optimize;
