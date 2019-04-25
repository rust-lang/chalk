use super::program_clauses::ToProgramClauses;
use crate::clauses::match_type_kind;
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use chalk_ir::TypeName;
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

    fn visit_ty(&mut self, ty: &Ty) {
        let mut clauses = vec![];
        match ty {
            Ty::Apply(application_ty) => match application_ty.name {
                TypeName::TypeKindId(type_kind_id) => {
                    match_type_kind(self.program, type_kind_id, &mut clauses)
                }
                TypeName::Placeholder(_) => (),
                TypeName::AssociatedType(type_id) => {
                    self.program
                        .associated_ty_data(type_id)
                        .to_program_clauses(self.program, &mut clauses);
                }
            },
            Ty::Projection(projection_ty) => {
                self.program
                    .associated_ty_data(projection_ty.associated_ty_id)
                    .to_program_clauses(self.program, &mut clauses);
            }
            Ty::UnselectedProjection(_) | Ty::ForAll(_) | Ty::BoundVar(_) | Ty::InferenceVar(_) => {
                ()
            }
        }
        self.round.extend(clauses);
    }

    fn visit_from_env(&mut self, from_env: &FromEnv) {
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

    fn visit_domain_goal(&mut self, domain_goal: &DomainGoal) {
        if let DomainGoal::FromEnv(from_env) = domain_goal {
            self.visit_from_env(from_env);
        }
    }

    pub fn visit_program_clause(&mut self, clause: &ProgramClause) {
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(&clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(&clause.value.consequence),
        }
    }
}
