use chalk_ir::IsCoinductive;
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

impl IsCoinductive for ProgramEnvironment {
    fn is_coinductive_trait(&self, trait_id: TraitId) -> bool {
        self.coinductive_traits.contains(&trait_id)
    }
}
