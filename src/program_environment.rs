use chalk_ir::could_match::CouldMatch;
use chalk_ir::DomainGoal;
use chalk_ir::IsCoinductive;
use chalk_ir::ProgramClause;
use chalk_ir::TraitId;
use chalk_solve::solve::ProgramClauseSet;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Indicates whether a given trait has coinductive semantics --
    /// at present, this is true only for auto traits.
    coinductive_traits: BTreeSet<TraitId>,

    /// Compiled forms of the above:
    program_clauses: Vec<ProgramClause>,
}

impl ProgramEnvironment {
    pub fn new(coinductive_traits: BTreeSet<TraitId>, program_clauses: Vec<ProgramClause>) -> Self {
        Self {
            coinductive_traits,
            program_clauses,
        }
    }
}

impl ProgramClauseSet for ProgramEnvironment {
    fn program_clauses_that_could_match(&self, goal: &DomainGoal, vec: &mut Vec<ProgramClause>) {
        vec.extend(
            self.program_clauses
                .iter()
                .filter(|&clause| clause.could_match(goal))
                .cloned(),
        );
    }

    fn upcast(&self) -> &dyn IsCoinductive {
        self
    }
}

impl IsCoinductive for ProgramEnvironment {
    fn is_coinductive_trait(&self, trait_id: TraitId) -> bool {
        self.coinductive_traits.contains(&trait_id)
    }
}
