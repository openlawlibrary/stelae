-- Add up migration script here
PRAGMA foreign_keys = OFF;

ALTER TABLE data_repo_commits
ADD COLUMN commit_type INTEGER;

PRAGMA foreign_keys = ON;
PRAGMA optimize;
