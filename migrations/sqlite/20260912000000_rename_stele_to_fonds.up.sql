-- Add up migration script here
PRAGMA foreign_keys = OFF;

ALTER TABLE stele RENAME TO fonds;
ALTER TABLE document_element RENAME COLUMN stele TO fonds;
ALTER TABLE library RENAME COLUMN stele TO fonds;
ALTER TABLE publication RENAME COLUMN stele TO fonds;

-- These FK columns have never been indexed, despite every query filtering on them.
CREATE INDEX document_element_fonds_idx ON document_element(fonds);
CREATE INDEX library_fonds_idx ON library(fonds);
CREATE INDEX publication_fonds_idx ON publication(fonds);
CREATE INDEX data_repo_commits_publication_id_idx ON data_repo_commits(publication_id);

PRAGMA foreign_keys = ON;
PRAGMA optimize;
