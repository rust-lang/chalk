use chalk_ir::DomainGoal;
use chalk_ir::FromEnv;
use chalk_ir::ProgramClause;
use chalk_ir::Ty;
use chalk_solve::clauses::ToProgramClauses;
use chalk_solve::RustIrDatabase;
use rustc_hash::FxHashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

impl ProgramEnvironment {
    pub fn new(program_clauses: Vec<ProgramClause>) -> Self {
        Self { program_clauses }
    }

    pub fn program_clauses_for_env<'db>(
        &self,
        program: &'db dyn RustIrDatabase,
        clauses: &mut Vec<ProgramClause>,
    ) {
        let mut last_round = FxHashSet::default();
        {
            let mut visitor = ClauseVisitor::new(program, &mut last_round);
            for clause in &self.program_clauses {
                visitor.visit_program_clause(clause.clone()); // TODO make ProgramClause copy or avoid clones
            }
        }

        let mut closure = last_round.clone();
        let mut next_round = FxHashSet::default();
        while !last_round.is_empty() {
            let mut visitor = ClauseVisitor::new(program, &mut next_round);
            for clause in last_round.drain() {
                visitor.visit_program_clause(clause);
            }
            last_round.extend(
                next_round
                    .drain()
                    .filter(|clause| closure.insert(clause.clone())),
            );
        }

        clauses.extend(closure.drain())
    }
}

struct ClauseVisitor<'db, 'set> {
    program: &'db dyn RustIrDatabase,
    round: &'set mut FxHashSet<ProgramClause>,
}

impl<'db, 'set> ClauseVisitor<'db, 'set> {
    fn new(program: &'db dyn RustIrDatabase, round: &'set mut FxHashSet<ProgramClause>) -> Self {
        ClauseVisitor { program, round }
    }

    fn visit_ty(&mut self, ty: Ty) {
        match ty {
            Ty::Projection(projection_ty) => {
                let mut clauses = vec![];
                self.program
                    .associated_ty_data(projection_ty.associated_ty_id)
                    .to_program_clauses(self.program, &mut clauses);
                self.round.extend(clauses);
            }
            _ => (), // TODO implement
        }
    }

    fn visit_from_env(&mut self, from_env: FromEnv) {
        match from_env {
            FromEnv::Trait(trait_ref) => {
                let mut clauses = vec![];
                self.program
                    .trait_datum(trait_ref.trait_id)
                    .to_program_clauses(self.program, &mut clauses);
                self.round.extend(clauses);
            }
            FromEnv::Ty(ty) => self.visit_ty(ty),
        }
    }

    fn visit_domain_goal(&mut self, domain_goal: DomainGoal) {
        if let DomainGoal::FromEnv(from_env) = domain_goal {
            self.visit_from_env(from_env);
        }
    }

    fn visit_program_clause(&mut self, clause: ProgramClause) {
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(clause.value.consequence),
        }
    }
}
