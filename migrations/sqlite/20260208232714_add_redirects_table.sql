CREATE TABLE redirects (
    stele_name TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    from_url TEXT KEY NOT NULL,
    to_url TEXT NOT NULL,
    PRIMARY KEY (stele_name,repo_name, from_url)
);

PRAGMA optimize;