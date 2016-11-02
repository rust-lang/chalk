use infer::*;
use formula::*;
use std::iter::once;
use std::sync::Arc;

pub struct Environment {
    parent: Option<Arc<Environment>>,
    depth: usize,
    clauses: Vec<Clause<Leaf>>,
}

impl Environment {
    pub fn new(parent: Option<Arc<Environment>>, clauses: Vec<Clause<Leaf>>) -> Self {
        let depth = match parent {
            None => 0,
            Some(ref parent) => parent.depth + 1,
        };

        Environment {
            parent: parent,
            depth: depth,
            clauses: clauses,
        }
    }

    pub fn iter_parents<'a>(&'a self) -> impl Iterator<Item=&'a Environment> {
        once(self)
            .chain(
                self.parent.as_ref()
                           .iter()
                           .flat_map(|e| e.iter_parents()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn clauses(&self) -> &[Clause<Leaf>] {
        &self.clauses
    }

    pub fn universe_index(&self) -> UniverseIndex {
        UniverseIndex { counter: self.depth }
    }
}
