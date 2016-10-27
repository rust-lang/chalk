use ena::unify::UnificationTable;
use solve::*;

pub struct Solver {
    unify: UnificationTable<InferenceVariable>,
    obligations: Vec<Obligation>,
}

impl Solver {
    pub fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.unify.new_key(ui)
    }

    pub fn universe_index(&mut self, v: InferenceVariable) -> UniverseIndex {
        self.unify.probe_value(v)
    }
}
