-- Add migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE stele (
    name TEXT PRIMARY KEY
);
CREATE TABLE document (
    doc_id TEXT PRIMARY KEY
);
CREATE TABLE document_element (
    doc_mpath TEXT,
    url TEXT,
    doc_id TEXT,
    stele TEXT,
    CONSTRAINT fk_doc_id
        FOREIGN KEY (doc_id)
        REFERENCES document(doc_id),
    PRIMARY KEY (doc_mpath),
    CONSTRAINT fk_stele FOREIGN KEY (stele) REFERENCES stele(name) ON DELETE CASCADE
);
CREATE TABLE library (
    mpath TEXT PRIMARY KEY,
    url TEXT,
    stele TEXT,
    CONSTRAINT fk_stele FOREIGN KEY (stele) REFERENCES stele(name) ON DELETE CASCADE
);
CREATE TABLE publication (
    id TEXT,
    name TEXT,
    date INTEGER,
    stele TEXT,
    revoked INTEGER,
    last_valid_publication_id TEXT,
    last_valid_version TEXT,
    CONSTRAINT fk_last_valid_version
        FOREIGN KEY (last_valid_version)
        REFERENCES version(codified_date),
    CONSTRAINT fk_last_valid_publication
        FOREIGN KEY (last_valid_publication_id)
        REFERENCES publication(id),
    CONSTRAINT fk_stele
        FOREIGN KEY (stele)
        REFERENCES stele(name)
        ON DELETE CASCADE,
    PRIMARY KEY (id)
);
CREATE TABLE version(
    codified_date TEXT PRIMARY KEY
);
CREATE TABLE publication_version (
    id TEXT,
    version TEXT,
    publication_id TEXT,
    build_reason TEXT,
    CONSTRAINT fk_publication
        FOREIGN KEY (publication_id)
        REFERENCES publication(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_version
        FOREIGN KEY (version)
        REFERENCES version(codified_date),
    PRIMARY KEY (id)
);
CREATE TABLE publication_has_publication_versions (
    publication_id TEXT,
    publication_version_id TEXT,
    CONSTRAINT fk_publication FOREIGN KEY (publication_id) REFERENCES publication(id) ON DELETE CASCADE,
    CONSTRAINT fk_referenced_publication_version FOREIGN KEY (publication_version_id) REFERENCES publication_version(id) ON DELETE CASCADE,
    PRIMARY KEY (publication_id, publication_version_id)
);
CREATE TABLE document_change (
    id TEXT,
    status INTEGER,
    change_reason TEXT,
    publication_version_id TEXT,
    doc_mpath TEXT,
    CONSTRAINT fk_doc_el
        FOREIGN KEY (doc_mpath)
        REFERENCES document_element(doc_mpath)
        ON DELETE CASCADE,
    CONSTRAINT fk_publication_version
        FOREIGN KEY (publication_version_id)
        REFERENCES publication_version(id)
        ON DELETE CASCADE,
    PRIMARY KEY (id)
);

CREATE INDEX document_change_doc_mpath_idx ON document_change(doc_mpath COLLATE NOCASE);
CREATE INDEX document_change_publication_version_idx ON document_change(publication_version_id);
CREATE INDEX document_change_doc_mpath_pub_id_status_idx ON document_change(doc_mpath COLLATE NOCASE, status);

CREATE INDEX publication_has_publication_versions_publication_id_idx ON publication_has_publication_versions(publication_id);
CREATE INDEX publication_has_publication_versions_publication_version_id_idx ON publication_has_publication_versions(publication_version_id);

CREATE INDEX publication_version_id_idx ON publication_version(id);

CREATE TABLE library_change (
    publication_version_id TEXT,
    status TEXT,
    library_mpath TEXT,
    CONSTRAINT fk_publication_version
        FOREIGN KEY (publication_version_id)
        REFERENCES publication_version(id)
        ON DELETE CASCADE,
    PRIMARY KEY (publication_version_id, library_mpath, status)
);
CREATE TABLE changed_library_document (
    library_mpath TEXT,
    document_change_id TEXT,
    CONSTRAINT fk_document_change
        FOREIGN KEY (document_change_id)
        REFERENCES document_change(id)
        ON DELETE CASCADE,
    PRIMARY KEY (document_change_id, library_mpath)
);
CREATE INDEX library_change_library_mpath_idx ON library_change(library_mpath COLLATE NOCASE);
CREATE INDEX library_change_publication_version_idx ON library_change(publication_version_id);
CREATE INDEX library_change_status_idx ON library_change(library_mpath COLLATE NOCASE, status);

CREATE INDEX changed_library_document_library_mpath_idx ON changed_library_document(library_mpath COLLATE NOCASE);

PRAGMA optimize;