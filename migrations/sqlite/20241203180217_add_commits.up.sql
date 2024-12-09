-- Add up migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE auth_commits (
    commit_hash TEXT,
    timestamp INTEGER,
    publication_version_id TEXT,
    CONSTRAINT fk_auth_commits_publication_version
        FOREIGN KEY (publication_version_id)
        REFERENCES publication_version(id)
        ON DELETE CASCADE,
    PRIMARY KEY (commit_hash, publication_version_id)
);
CREATE TABLE data_commits (
    commit_hash TEXT,
    date TEXT,
    data_repo_type TEXT,
    auth_commit_hash TEXT,
    publication_id TEXT,
    CONSTRAINT fk_data_commits_auth_commit
        FOREIGN KEY (auth_commit_hash)
        REFERENCES auth_commits(commit_hash)
        ON DELETE CASCADE,
    CONSTRAINT fk_data_commits_publication
        FOREIGN KEY (publication_id)
        REFERENCES publication(id)
        ON DELETE CASCADE,
    PRIMARY KEY (commit_hash, auth_commit_hash)
);

CREATE UNIQUE INDEX auth_commits_hash_unq_idx ON auth_commits(commit_hash);

PRAGMA optimize;