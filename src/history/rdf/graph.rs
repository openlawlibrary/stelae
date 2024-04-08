use anyhow::Context;
use sophia::api::graph::Graph;
use sophia::api::ns::NsTerm;
use sophia::api::{prelude::*, term::SimpleTerm};
/// The helper methods for working with RDF in Stelae.
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
        let triple = match (subject, predicate, object) {
            (Some(s), None, None) => {
                self.g.triples_matching([s], Any, Any).next().context("Did not find a triple matching provided subject in the graph")?
            },
            (None, Some(p), None) => {
                self.g.triples_matching(Any, [p], Any).next().context("Did not find a triple matching provided predicate in the graph")?
            },
            (None, None, Some(o)) => {
                self.g.triples_matching(Any, Any, [o]).next().context("Did not find a triple matching provided object in the graph")?
            },
            (Some(s), Some(p), None) => {
                self.g.triples_matching([s], [p], Any).next().context("Did not find a triple matching provided subject and predicate in the graph")?
            },
            (Some(s), None, Some(o)) => {
                self.g.triples_matching([s], Any, [o]).next().context("Did not find a triple matching provided subject and object in the graph")?
            },
            (None, Some(p), Some(o)) => {
                self.g.triples_matching(Any, [p], [o]).next().context("Did not find a triple matching provided predicate and object in the graph")?
            },
            (Some(s), Some(p), Some(o)) => {
                self.g.triples_matching([s], [p], [o]).next().context("Did not find a triple matching provided subject, predicate and object in the graph")?
            },
            (None, None, None) => {
                anyhow::bail!("No subject, predicate or object provided");
            }
        };
        // let triple = self.g.triples_matching(Any, [predicate], Any).next().context("Did not find a triple matching provided object in the graph")?;
        let literal = match triple?.o() {
            SimpleTerm::LiteralLanguage(literal, _) => literal,
            SimpleTerm::LiteralDatatype(literal, _) => literal,
            _ => {
                anyhow::bail!("Expected literal language, got - {:?}", triple?.o());
            }
        };
        Ok(literal.to_string())
    }
}
