use chalk_ir::ProgramClause;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

impl ProgramEnvironment {
    pub fn new(program_clauses: Vec<ProgramClause>) -> Self {
        Self { program_clauses }
    }
}
