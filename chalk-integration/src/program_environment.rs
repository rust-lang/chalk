use crate::interner::ChalkIr;
use chalk_ir::ProgramClause;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause<ChalkIr>>,
}

impl ProgramEnvironment {
    pub fn new(program_clauses: Vec<ProgramClause<ChalkIr>>) -> Self {
        Self { program_clauses }
    }
}
