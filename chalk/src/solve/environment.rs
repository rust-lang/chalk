use infer::*;
use formula::*;
use std::iter::once;
use std::sync::Arc;

pub struct Environment {
    parent: Option<Arc<Environment>>,
    depth: usize,
    clauses: Vec<Clause<Application>>,
}

impl Environment {
    pub fn new(parent: Option<Arc<Environment>>, clauses: Vec<Clause<Application>>) -> Self {
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

    pub fn iter_parents<'a>(&'a self) -> impl Iterator<Item=&'a Environment>+'a {
        Box::new(
            once(self).chain(
                self.parent.iter()
                           .flat_map(|e| e.iter_parents())))
            as Box<Iterator<Item=&'a Environment>>
    }

    pub fn clauses(&self) -> &[Clause<Application>] {
        &self.clauses
    }

    pub fn universe_index(&self) -> UniverseIndex {
        UniverseIndex { counter: self.depth }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn clauses_relevant_to<'a>(&'a self, appl: &Application)
                                   -> impl Iterator<Item=&'a Clause<Application>>+'a {
        let relevant = appl.constant_and_arity();
        self.iter_parents()
            .flat_map(move |parent| parent.clauses())
            .filter(move |clause| clause.constant_and_arity() == relevant)
    }
}
