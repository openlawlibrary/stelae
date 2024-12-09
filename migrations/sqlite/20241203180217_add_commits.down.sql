-- Add down migration script here
PRAGMA foreign_keys = OFF;

DROP INDEX IF EXISTS auth_commits_hash_unq_idx;
DROP TABLE IF EXISTS data_repo_commits;
DROP TABLE IF EXISTS auth_repo_commits;

PRAGMA optimize;