use super::program_clauses::ToProgramClauses;
use crate::clauses::builder::ClauseBuilder;
use crate::clauses::{match_alias_ty, match_type_name};
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use crate::TyData;
use chalk_ir::interner::Interner;
use chalk_ir::visit::{Visit, Visitor};
use chalk_ir::DebruijnIndex;
use rustc_hash::FxHashSet;

/// When proving a `FromEnv` goal, we elaborate all `FromEnv` goals
/// found in the environment.
///
/// For example, when `T: Clone` is in the environment, we can prove
/// `T: Copy` by adding the clauses from `trait Clone`, which includes
/// the rule `FromEnv(T: Copy) :- FromEnv(T: Clone)
pub(super) fn elaborate_env_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    in_clauses: &[ProgramClause<I>],
    out: &mut FxHashSet<ProgramClause<I>>,
) {
    let mut this_round = vec![];
    in_clauses.visit_with(
        &mut EnvElaborator::new(db, &mut this_round),
        DebruijnIndex::INNERMOST,
    );
    out.extend(this_round);
}

struct EnvElaborator<'me, I: Interner> {
    db: &'me dyn RustIrDatabase<I>,
    builder: ClauseBuilder<'me, I>,
}

impl<'me, I: Interner> EnvElaborator<'me, I> {
    fn new(db: &'me dyn RustIrDatabase<I>, out: &'me mut Vec<ProgramClause<I>>) -> Self {
        EnvElaborator {
            db,
            builder: ClauseBuilder::new(db, out),
        }
    }
}

impl<'me, I: Interner> Visitor<'me, I> for EnvElaborator<'me, I> {
    type Result = ();

    fn as_dyn(&mut self) -> &mut dyn Visitor<'me, I, Result = Self::Result> {
        self
    }

    fn interner(&self) -> &'me I {
        self.db.interner()
    }

    fn visit_ty(&mut self, ty: &Ty<I>, _outer_binder: DebruijnIndex) {
        debug!("EnvElaborator::visit_ty(ty={:?})", ty);
        let interner = self.db.interner();
        match ty.data(interner) {
            TyData::Apply(application_ty) => {
                match_type_name(&mut self.builder, interner, application_ty)
            }
            TyData::Alias(alias_ty) => match_alias_ty(&mut self.builder, alias_ty),
            TyData::Placeholder(_) => {}

            // FIXME(#203) -- We haven't fully figured out the implied
            // bounds story around `dyn Trait` types.
            TyData::Dyn(_) => (),

            TyData::Function(_) | TyData::BoundVar(_) | TyData::InferenceVar(_) => (),
        }
    }

    fn visit_domain_goal(&mut self, domain_goal: &DomainGoal<I>, outer_binder: DebruijnIndex) {
        if let DomainGoal::FromEnv(from_env) = domain_goal {
            debug!("EnvElaborator::visit_domain_goal(from_env={:?})", from_env);
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
                FromEnv::Ty(ty) => ty.visit_with(self, outer_binder),
            }
        }
    }
}
