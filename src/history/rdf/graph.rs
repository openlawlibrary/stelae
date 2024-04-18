/// The helper methods for working with RDF in Stelae.
use anyhow::Context;
use sophia::api::graph::Graph;
use sophia::api::ns::NsTerm;
use sophia::api::{prelude::*, term::SimpleTerm};
use sophia::inmem::graph::FastGraph;

/// Stelae representation of an RDF graph.
pub struct StelaeGraph {
    /// The underlying graph.
    pub g: FastGraph,
}

impl StelaeGraph {
    /// Create a new graph.
    pub fn new() -> Self {
        Self {
            g: FastGraph::new(),
        }
    }
    /// Extract a literal from a triple matching.
    ///
    /// # Errors
    /// Errors if the triple matching the object is not found.
    /// Errors if the object is not an RDF literal.
    pub fn literal_from_triple_matching(
        &self,
        subject: Option<&SimpleTerm>,
        predicate: Option<NsTerm>,
        object: Option<NsTerm>,
    ) -> anyhow::Result<String> {
        let triple = self.get_next_triples_matching(subject, predicate, object)?;
        let literal = self.term_to_literal(&triple)?;
        Ok(literal)
    }

    /// Convert a term to a literal.
    pub fn term_to_literal(&self, term: &[&SimpleTerm<'_>; 3]) -> anyhow::Result<String> {
        match term.o() {
            SimpleTerm::LiteralLanguage(literal, _) => Ok(literal.to_string()),
            SimpleTerm::LiteralDatatype(literal, _) => Ok(literal.to_string()),
            _ => anyhow::bail!("Expected literal language, got - {:?}", term),
        }
    }

    /// Extract all literals from a triple matching.
    ///
    /// # Errors
    /// Errors if the triple matching the object is not found.
    /// Errors if the object is not an RDF literal.
    pub fn all_literals_from_triple_matching(
        &self,
        subject: Option<&SimpleTerm>,
        predicate: Option<NsTerm>,
        object: Option<NsTerm>,
    ) -> anyhow::Result<Vec<String>> {
        Ok(self
            .literal_from_triple_matching(subject, predicate, object)
            .into_iter()
            .map(|t| t)
            .collect())
    }

    /// Extract an IRI from a triple matching.
    ///
    /// # Errors
    /// Errors if the triple matching the object is not found.
    /// Errors if the object is not an RDF IRI.
    pub fn iri_from_triple_matching<'graph>(
        &'graph self,
        subject: Option<&'graph SimpleTerm>,
        predicate: Option<NsTerm<'graph>>,
        object: Option<NsTerm<'graph>>,
    ) -> anyhow::Result<SimpleTerm> {
        let triple = self.get_next_triples_matching(subject, predicate, object)?;
        let iri = match triple.o() {
            SimpleTerm::Iri(literal) => literal,
            _ => {
                anyhow::bail!("Expected literal language, got - {:?}", triple.o());
            }
        };
        Ok(SimpleTerm::Iri(iri.clone()))
    }

    fn get_next_triples_matching<'graph>(
        &'graph self,
        subject: Option<&'graph SimpleTerm>,
        predicate: Option<NsTerm<'graph>>,
        object: Option<NsTerm<'graph>>,
    ) -> anyhow::Result<[&'graph SimpleTerm<'_>; 3]> {
        let triple = self
            .triples_matching_inner(subject, predicate, object)
            .next()
            .context("Expected to find triple matching")?;
        Ok(triple?)
    }
}
