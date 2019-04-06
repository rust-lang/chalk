use chalk_ir::ProgramClause;
use chalk_ir::TraitId;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Indicates whether a given trait has coinductive semantics --
    /// at present, this is true only for auto traits.
    pub coinductive_traits: BTreeSet<TraitId>,

    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

impl ProgramEnvironment {
    pub fn new(coinductive_traits: BTreeSet<TraitId>, program_clauses: Vec<ProgramClause>) -> Self {
        Self {
            coinductive_traits,
            program_clauses,
        }
    }
}
