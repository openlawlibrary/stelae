-- Add up migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE data_repo_commits (
    commit_hash TEXT,
    date TEXT,
    repo_type TEXT,
    auth_commit_hash TEXT,
    auth_commit_timestamp INTEGER,
    publication_id TEXT,
    CONSTRAINT fk_data_repo_commits_publication
        FOREIGN KEY (publication_id)
        REFERENCES publication(id)
        ON DELETE CASCADE,
    PRIMARY KEY (commit_hash, auth_commit_hash, publication_id)
);

PRAGMA optimize;