/// The helper methods for working with RDF in Stelae.
use anyhow::Context;
use sophia::api::graph::{GTripleSource, Graph};
use sophia::api::ns::NsTerm;
use sophia::api::MownStr;
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
        let mut literals = Vec::new();
        let mut triples_iter = self.triples_matching_inner(subject, predicate, object);
        while let Some(term) = triples_iter.next() {
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
            (None, None, None) => Box::new(::std::iter::empty()),
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
            .filter_map(|t| {
                let t = t.ok()?;
                let subject = t.s();
                Some(subject)
            })
            .collect();
        Ok(iris)
    }
}

/// Unordered container of RDF items.
pub struct Bag<'graph> {
    uri: SimpleTerm<'graph>,
    graph: &'graph StelaeGraph,
}

impl Bag<'_> {
    /// Create a new Bag.
    pub fn new<'graph>(graph: &'graph StelaeGraph, uri: SimpleTerm<'graph>) -> Bag<'graph> {
        Bag { graph, uri }
    }

    /// Extract items from the container.
    pub fn items(&self) -> anyhow::Result<Vec<SimpleTerm>> {
        let container = &self.uri;
        let mut i = 1;
        let mut l_ = vec![];
        loop {
            let elem_uri = format!("http://www.w3.org/1999/02/22-rdf-syntax-ns#_{i}");
            let elem_uri = SimpleTerm::Iri(IriRef::new_unchecked(MownStr::from_str(&elem_uri)));
            if self
                .graph
                .g
                .triples_matching([container], Some(elem_uri.clone()), Any)
                .next()
                .is_some()
            {
                i += 1;
                let item = self
                    .graph
                    .g
                    .triples_matching([container], Some(elem_uri), Any)
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
