use super::program_clauses::ToProgramClauses;
use crate::clauses::match_type_kind;
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use chalk_ir::ProjectionEq;
use chalk_ir::ProjectionTy;
use chalk_ir::TypeName;
use chalk_ir::WhereClause;
use rustc_hash::FxHashSet;

/// When proving a `FromEnv` goal, we elaborate all `FromEnv` goals
/// found in the environment.
///
/// For example, when `T: Clone` is in the environment, we can prove
/// `T: Copy` by adding the clauses from `trait Clone`, which includes
/// the rule `FromEnv(T: Copy) :- FromEnv(T: Clone)
pub(super) fn elaborate_env_clauses(
    db: &dyn RustIrDatabase,
    in_clauses: &Vec<ProgramClause>,
    out: &mut FxHashSet<ProgramClause>,
) {
    let mut visitor = ClauseVisitor::new(db, out);
    for clause in in_clauses {
        visitor.visit_program_clause(&clause);
    }
}

struct ClauseVisitor<'db, 'set> {
    db: &'db dyn RustIrDatabase,
    round: &'set mut FxHashSet<ProgramClause>,
}

impl<'db, 'set> ClauseVisitor<'db, 'set> {
    fn new(db: &'db dyn RustIrDatabase, round: &'set mut FxHashSet<ProgramClause>) -> Self {
        ClauseVisitor { db, round }
    }

    fn visit_projection_ty(&mut self, projection_ty: &ProjectionTy) {
        let mut clauses = vec![];
        self.db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(self.db, &mut clauses);
        self.round.extend(clauses);
    }

    fn visit_ty(&mut self, ty: &Ty) {
        let mut clauses = vec![];
        match ty {
            Ty::Apply(application_ty) => match application_ty.name {
                TypeName::TypeKindId(type_kind_id) => {
                    match_type_kind(self.db, type_kind_id, &mut clauses)
                }
                TypeName::Placeholder(_) => (),
                TypeName::AssociatedType(type_id) => {
                    self.db
                        .associated_ty_data(type_id)
                        .to_program_clauses(self.db, &mut clauses);
                }
            },
            Ty::Projection(projection_ty) => {
                self.visit_projection_ty(projection_ty);
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
                let trait_datum = self.db.trait_datum(trait_ref.trait_id);

                trait_datum.to_program_clauses(self.db, &mut clauses);

                // If we know that `T: Iterator`, then we also know
                // things about `<T as Iterator>::Item`, so push those
                // implied bounds too:
                for &associated_ty_id in &trait_datum.binders.value.associated_ty_ids {
                    self.db
                        .associated_ty_data(associated_ty_id)
                        .to_program_clauses(self.db, &mut clauses);
                }

                self.round.extend(clauses);
            }
            FromEnv::Ty(ty) => self.visit_ty(ty),
        }
    }

    fn visit_domain_goal(&mut self, domain_goal: &DomainGoal) {
        match domain_goal {
            DomainGoal::FromEnv(from_env) => self.visit_from_env(from_env),
            _ => {}
        }
    }

    fn visit_program_clause(&mut self, clause: &ProgramClause) {
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(&clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(&clause.value.consequence),
        }
    }
}
