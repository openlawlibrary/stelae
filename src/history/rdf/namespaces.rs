/// This module contains the RDF namespaces used by Stelae.
use sophia::api::namespace;

/// Open Law Library ontology.
pub mod oll {
    use super::namespace;
    namespace! {
        "https://open.law/us/ngo/oll/_ontology/v0.1/ontology.owl#",
        CollectionVersion,
        DocumentVersion,
        docId,
        codifiedDate,
        lastValidPublication,
        lastValidCodifiedDate,
        hasChanges,
        documentMaterializedPath,
        url,
        reason,
        status,
        libraryMaterializedPath
    }
}

/// Dublin Core Terms ontology.
pub mod dcterms {
    use super::namespace;

    namespace! {
        "http://purl.org/dc/terms/",
        available
    }
}
