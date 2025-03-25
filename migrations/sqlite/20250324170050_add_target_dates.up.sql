-- Add up migration script here
PRAGMA foreign_keys = OFF;

ALTER TABLE data_repo_commits
  DROP COLUMN date;

ALTER TABLE data_repo_commits
  ADD COLUMN codified_date TEXT;

ALTER TABLE data_repo_commits
  ADD COLUMN build_date TEXT;

PRAGMA foreign_keys = ON;
PRAGMA optimize;
