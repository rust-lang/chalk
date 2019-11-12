use super::program_clauses::ToProgramClauses;
use crate::clauses::builder::ClauseBuilder;
use crate::clauses::match_type_kind;
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use crate::TyData;
use chalk_ir::family::ChalkIr;
use chalk_ir::ProjectionTy;
use chalk_ir::TypeName;
use rustc_hash::FxHashSet;

/// When proving a `FromEnv` goal, we elaborate all `FromEnv` goals
/// found in the environment.
///
/// For example, when `T: Clone` is in the environment, we can prove
/// `T: Copy` by adding the clauses from `trait Clone`, which includes
/// the rule `FromEnv(T: Copy) :- FromEnv(T: Clone)
pub(super) fn elaborate_env_clauses(
    db: &dyn RustIrDatabase,
    in_clauses: &Vec<ProgramClause<ChalkIr>>,
    out: &mut FxHashSet<ProgramClause<ChalkIr>>,
) {
    let mut this_round = vec![];
    let mut visitor = EnvElaborator::new(db, &mut this_round);
    for clause in in_clauses {
        visitor.visit_program_clause(&clause);
    }
    out.extend(this_round);
}

struct EnvElaborator<'me> {
    db: &'me dyn RustIrDatabase,
    builder: ClauseBuilder<'me>,
}

impl<'me> EnvElaborator<'me> {
    fn new(db: &'me dyn RustIrDatabase, out: &'me mut Vec<ProgramClause<ChalkIr>>) -> Self {
        EnvElaborator {
            db,
            builder: ClauseBuilder::new(db, out),
        }
    }

    fn visit_projection_ty(&mut self, projection_ty: &ProjectionTy<ChalkIr>) {
        self.db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(&mut self.builder);
    }

    fn visit_ty(&mut self, ty: &Ty<ChalkIr>) {
        match ty.data() {
            TyData::Apply(application_ty) => match application_ty.name {
                TypeName::TypeKindId(type_kind_id) => {
                    match_type_kind(&mut self.builder, type_kind_id)
                }
                TypeName::Placeholder(_) | TypeName::Error => (),
                TypeName::AssociatedType(type_id) => {
                    self.db
                        .associated_ty_data(type_id)
                        .to_program_clauses(&mut self.builder);
                }
            },
            TyData::Projection(projection_ty) => {
                self.visit_projection_ty(projection_ty);
            }

            // FIXME(#203) -- We haven't fully figured out the implied
            // bounds story around object and impl trait types.
            TyData::Dyn(_) | TyData::Opaque(_) => (),

            TyData::ForAll(_) | TyData::BoundVar(_) | TyData::InferenceVar(_) => (),
        }
    }

    fn visit_from_env(&mut self, from_env: &FromEnv<ChalkIr>) {
        match from_env {
            FromEnv::Trait(trait_ref) => {
                let trait_datum = self.db.trait_datum(trait_ref.trait_id);

                trait_datum.to_program_clauses(&mut self.builder);

                // If we know that `T: Iterator`, then we also know
                // things about `<T as Iterator>::Item`, so push those
                // implied bounds too:
                for &associated_ty_id in &trait_datum.associated_ty_ids {
                    self.db
                        .associated_ty_data(associated_ty_id)
                        .to_program_clauses(&mut self.builder);
                }
            }
            FromEnv::Ty(ty) => self.visit_ty(ty),
        }
    }

    fn visit_domain_goal(&mut self, domain_goal: &DomainGoal<ChalkIr>) {
        match domain_goal {
            DomainGoal::FromEnv(from_env) => self.visit_from_env(from_env),
            _ => {}
        }
    }

    fn visit_program_clause(&mut self, clause: &ProgramClause<ChalkIr>) {
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(&clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(&clause.value.consequence),
        }
    }
}
