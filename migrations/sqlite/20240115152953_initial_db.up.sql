-- Add migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE stele (
    name TEXT PRIMARY KEY
);
CREATE TABLE document (
    doc_id TEXT PRIMARY KEY
);
CREATE TABLE library (
    mpath TEXT PRIMARY KEY
);
CREATE TABLE library_document (
    collection_mpath TEXT,
    doc_id TEXT,
    start DATE,
    end DATE,
    CONSTRAINT fk_coll_mpath
        FOREIGN KEY (collection_mpath)
        REFERENCES library(mpath),
    CONSTRAINT fk_doc_id
        FOREIGN KEY (doc_id)
        REFERENCES document(doc_id),
    PRIMARY KEY (collection_mpath, doc_id)
);
CREATE TABLE publication (
    name TEXT,
    date INTEGER,
    stele TEXT,
    revoked INTEGER,
    last_valid_publication_name TEXT,
    last_valid_version TEXT,
    CONSTRAINT fk_last_valid_version
        FOREIGN KEY (last_valid_version)
        REFERENCES version(codified_date),
    CONSTRAINT fk_last_valid_publication
        FOREIGN KEY (last_valid_publication_name, stele)
        REFERENCES publication(name, stele),
    CONSTRAINT fk_stele
        FOREIGN KEY (stele)
        REFERENCES stele(name)
        ON DELETE CASCADE,
    PRIMARY KEY (name, stele)
);
CREATE TABLE publication_version (
    version TEXT,
    publication TEXT,
    stele TEXT,
    build_reason TEXT,
    CONSTRAINT fk_publication
        FOREIGN KEY (publication, stele)
        REFERENCES publication(name, stele)
        ON DELETE CASCADE,
    CONSTRAINT fk_version
        FOREIGN KEY (version)
        REFERENCES version(codified_date),
    PRIMARY KEY (publication, version, stele)
);
CREATE TABLE publication_has_publication_versions (
    publication TEXT,
    referenced_publication TEXT,
    referenced_version TEXT,
    stele TEXT,
    CONSTRAINT fk_publication FOREIGN KEY (publication, stele) REFERENCES publication(name, stele) ON DELETE CASCADE,
    CONSTRAINT fk_referenced_publication FOREIGN KEY (referenced_publication, referenced_version, stele) REFERENCES publication_version(publication, version, stele) ON DELETE CASCADE,
    PRIMARY KEY (publication, referenced_publication, referenced_version, stele)
);
CREATE TABLE version(
    codified_date TEXT PRIMARY KEY
);
CREATE TABLE document_change (
    doc_mpath TEXT,
    status TEXT,
    url TEXT,
    change_reason TEXT,
    publication TEXT,
    version TEXT,
    stele TEXT,
    doc_id TEXT,
    CONSTRAINT fk_doc_id
        FOREIGN KEY (doc_id)
        REFERENCES document(doc_id)
        ON DELETE CASCADE,
    CONSTRAINT fk_publication_version
        FOREIGN KEY (publication, version, stele)
        REFERENCES publication_version(publication, version, stele)
        ON DELETE CASCADE,
    PRIMARY KEY (doc_mpath, status, publication, version, stele)
);
CREATE INDEX document_change_doc_mpath_idx ON document_change(doc_mpath COLLATE NOCASE);
CREATE TABLE library_change (
    publication TEXT,
    version TEXT,
    stele TEXT,
    status TEXT,
    library_mpath TEXT,
    url TEXT,
    CONSTRAINT fk_publication_version
        FOREIGN KEY (publication, version, stele)
        REFERENCES publication_version(publication, version, stele)
        ON DELETE CASCADE,
    PRIMARY KEY (publication, version, stele, library_mpath, status)
);
CREATE TABLE changed_library_document (
    publication TEXT,
    version TEXT,
    stele TEXT,
    doc_mpath TEXT,
    status TEXT,
    library_mpath TEXT,
    url TEXT,
    CONSTRAINT fk_document_change
        FOREIGN KEY (publication, version, stele, doc_mpath, status)
        REFERENCES document_change(publication, version, stele, doc_mpath, status)
        ON DELETE CASCADE,
    PRIMARY KEY (publication, version, stele, library_mpath, doc_mpath, status)
);
CREATE INDEX library_change_library_mpath_idx ON library_change(library_mpath COLLATE NOCASE);
CREATE INDEX changed_library_document_library_mpath_idx ON changed_library_document(library_mpath COLLATE NOCASE);

PRAGMA optimize;