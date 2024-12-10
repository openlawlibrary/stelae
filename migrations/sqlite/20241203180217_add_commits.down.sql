-- Add down migration script here
PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS data_repo_commits;

PRAGMA optimize;