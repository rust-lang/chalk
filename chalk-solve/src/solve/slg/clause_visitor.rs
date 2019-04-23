use crate::clauses::ToProgramClauses;
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use rustc_hash::FxHashSet;

pub struct ClauseVisitor<'db, 'set> {
    program: &'db dyn RustIrDatabase,
    round: &'set mut FxHashSet<ProgramClause>,
}

impl<'db, 'set> ClauseVisitor<'db, 'set> {
    pub fn new(
        program: &'db dyn RustIrDatabase,
        round: &'set mut FxHashSet<ProgramClause>,
    ) -> Self {
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

    pub fn visit_program_clause(&mut self, clause: ProgramClause) {
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(clause.value.consequence),
        }
    }
}
