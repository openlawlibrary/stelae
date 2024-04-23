#![allow(
    clippy::module_name_repetitions,
    clippy::min_ident_chars,
    clippy::pattern_type_mismatch
)]
/// The helper methods for working with RDF in Stelae.
use anyhow::Context;
use sophia::api::graph::{GTripleSource, Graph};
use sophia::api::ns::NsTerm;
use sophia::api::MownStr;
use sophia::api::{prelude::*, term::SimpleTerm};
use sophia::inmem::graph::FastGraph;
use std::iter;
/// Stelae representation of an RDF graph.
pub struct StelaeGraph {
    /// The underlying graph.
    pub g: FastGraph,
}

impl Default for StelaeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl StelaeGraph {
    /// Create a new graph.
    #[must_use]
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
    ///
    /// # Errors
    /// Errors if the term is not an RDF literal.
    pub fn term_to_literal(&self, term: &[&SimpleTerm<'_>; 3]) -> anyhow::Result<String> {
        match &term.o() {
            SimpleTerm::LiteralDatatype(literal, _) | SimpleTerm::LiteralLanguage(literal, _) => {
                Ok(literal.to_string())
            }
            SimpleTerm::Iri(_)
            | SimpleTerm::BlankNode(_)
            | SimpleTerm::Triple(_)
            | SimpleTerm::Variable(_) => {
                anyhow::bail!("Expected literal language, got - {:?}", term)
            }
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
        let mut literals = Vec::new();
        let triples_iter = self.triples_matching_inner(subject, predicate, object);
        for term in triples_iter {
            literals.push(self.term_to_literal(&term?)?);
        }
        Ok(literals)
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
        let SimpleTerm::Iri(iri) = &triple.o() else {
            anyhow::bail!("Expected literal language, got - {:?}", triple.o());
        };
        Ok(SimpleTerm::Iri(iri.clone()))
    }

    /// Returns the next triple matching the given subject, predicate, and object.
    ///
    /// # Errors
    /// Errors if the triple matching the object is not found.
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

    /// Utility method to convert from Option method arguments to a triple source.
    fn triples_matching_inner<'graph>(
        &'graph self,
        subject: Option<&'graph SimpleTerm>,
        predicate: Option<NsTerm<'graph>>,
        object: Option<NsTerm<'graph>>,
    ) -> GTripleSource<'graph, FastGraph> {
        let triple = match (subject, predicate, object) {
            (Some(s), None, None) => self.g.triples_matching([s], Any, Any),
            (None, Some(p), None) => self.g.triples_matching(Any, [p], Any),
            (None, None, Some(o)) => self.g.triples_matching(Any, Any, [o]),
            (Some(s), Some(p), None) => self.g.triples_matching([s], [p], Any),
            (Some(s), None, Some(o)) => self.g.triples_matching([s], Any, [o]),
            (None, Some(p), Some(o)) => self.g.triples_matching(Any, [p], [o]),
            (Some(s), Some(p), Some(o)) => self.g.triples_matching([s], [p], [o]),
            (None, None, None) => Box::new(iter::empty()),
        };
        triple
    }

    /// Extract all IRIs from a triple matching.
    ///
    /// # Errors
    /// Errors if the triple matching the object is not found.
    /// Errors if the object is not an RDF IRI.
    pub fn all_iris_from_triple_matching<'graph>(
        &'graph self,
        subject: Option<&'graph SimpleTerm>,
        predicate: Option<NsTerm<'graph>>,
        object: Option<NsTerm<'graph>>,
    ) -> anyhow::Result<Vec<&SimpleTerm>> {
        let triples_iter = self.triples_matching_inner(subject, predicate, object);
        let iris = triples_iter
            .into_iter()
            .filter_map(|triple| {
                let found_triple = triple.ok()?;
                let subj = found_triple.s();
                Some(subj)
            })
            .collect();
        Ok(iris)
    }
}

/// Unordered container of RDF items.
pub struct Bag<'graph> {
    /// The container URI.
    uri: SimpleTerm<'graph>,
    /// The underlying graph.
    graph: &'graph StelaeGraph,
}

impl Bag<'_> {
    /// Create a new Bag.
    #[must_use]
    pub const fn new<'graph>(graph: &'graph StelaeGraph, uri: SimpleTerm<'graph>) -> Bag<'graph> {
        Bag { uri, graph }
    }

    /// Extract items from the container.
    ///
    /// # Errors
    /// Errors if the items are not found.
    #[allow(clippy::separated_literal_suffix)]
    pub fn items(&self) -> anyhow::Result<Vec<SimpleTerm>> {
        let container = &self.uri;
        let mut i = 1_u32;
        let mut l_ = vec![];
        loop {
            let el_uri = format!("http://www.w3.org/1999/02/22-rdf-syntax-ns#_{i}");
            let elem_iri = SimpleTerm::Iri(IriRef::new_unchecked(MownStr::from_str(&el_uri)));
            if self
                .graph
                .g
                .triples_matching([container], Some(elem_iri.clone()), Any)
                .next()
                .is_some()
            {
                i += 1;
                let item = self
                    .graph
                    .g
                    .triples_matching([container], Some(elem_iri), Any)
                    .next()
                    .context(format!("Expected to find item in {container:?}"))?
                    .context("Expected to find item in container")?
                    .o()
                    .clone();
                l_.push(item);
            } else {
                break;
            }
        }
        Ok(l_)
    }
}
