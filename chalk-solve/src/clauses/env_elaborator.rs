use super::program_clauses::ToProgramClauses;
use crate::clauses::builder::ClauseBuilder;
use crate::clauses::{match_alias_ty, match_ty};
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use crate::{debug_span, TyKind};
use chalk_ir::interner::Interner;
use chalk_ir::visit::{TypeVisitable, TypeVisitor};
use chalk_ir::{DebruijnIndex, Environment};
use rustc_hash::FxHashSet;
use std::ops::ControlFlow;
use tracing::instrument;

/// When proving a `FromEnv` goal, we elaborate all `FromEnv` goals
/// found in the environment.
///
/// For example, when `T: Clone` is in the environment, we can prove
/// `T: Copy` by adding the clauses from `trait Clone`, which includes
/// the rule `FromEnv(T: Copy) :- FromEnv(T: Clone)`
pub(super) fn elaborate_env_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    in_clauses: &[ProgramClause<I>],
    out: &mut FxHashSet<ProgramClause<I>>,
    environment: &Environment<I>,
) {
    let mut this_round = vec![];
    let builder = &mut ClauseBuilder::new(db, &mut this_round);
    let mut elaborater = EnvElaborator {
        db,
        builder,
        environment,
    };
    in_clauses.visit_with(&mut elaborater, DebruijnIndex::INNERMOST);
    out.extend(this_round);
}

struct EnvElaborator<'me, 'builder, I: Interner> {
    db: &'me dyn RustIrDatabase<I>,
    builder: &'builder mut ClauseBuilder<'me, I>,
    environment: &'me Environment<I>,
}

impl<'me, 'builder, I: Interner> TypeVisitor<I> for EnvElaborator<'me, 'builder, I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn interner(&self) -> I {
        self.db.interner()
    }
    #[instrument(level = "debug", skip(self, _outer_binder))]
    fn visit_ty(&mut self, ty: &Ty<I>, _outer_binder: DebruijnIndex) -> ControlFlow<()> {
        match ty.kind(self.interner()) {
            TyKind::Alias(alias_ty) => match_alias_ty(self.builder, self.environment, alias_ty),
            TyKind::Placeholder(_) => {}

            // FIXME(#203) -- We haven't fully figured out the implied
            // bounds story around `dyn Trait` types.
            TyKind::Dyn(_) => (),

            TyKind::Function(_) | TyKind::BoundVar(_) | TyKind::InferenceVar(_, _) => (),

            _ => {
                // This shouldn't fail because of the above clauses
                match_ty(self.builder, self.environment, ty)
                    .map_err(|_| ())
                    .unwrap()
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_domain_goal(
        &mut self,
        domain_goal: &DomainGoal<I>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<()> {
        if let DomainGoal::FromEnv(from_env) = domain_goal {
            debug_span!("visit_domain_goal", ?from_env);
            match from_env {
                FromEnv::Trait(trait_ref) => {
                    let trait_datum = self.db.trait_datum(trait_ref.trait_id);

                    trait_datum.to_program_clauses(self.builder, self.environment);

                    // If we know that `T: Iterator`, then we also know
                    // things about `<T as Iterator>::Item`, so push those
                    // implied bounds too:
                    for &associated_ty_id in &trait_datum.associated_ty_ids {
                        self.db
                            .associated_ty_data(associated_ty_id)
                            .to_program_clauses(self.builder, self.environment);
                    }
                    ControlFlow::Continue(())
                }
                FromEnv::Ty(ty) => ty.visit_with(self, outer_binder),
            }
        } else {
            ControlFlow::Continue(())
        }
    }
}
