-- Add up migration script here
PRAGMA foreign_keys = OFF;

ALTER TABLE publication
  ADD COLUMN html_data_repo_name TEXT;

PRAGMA foreign_keys = ON;
PRAGMA optimize;
