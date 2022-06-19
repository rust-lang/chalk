use crate::infer::InferenceTable;
use chalk_ir::fold::TypeFoldable;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;

pub trait CanonicalExt<T: HasInterner, I: Interner> {
    fn map<OP, U>(self, interner: I, op: OP) -> Canonical<U>
    where
        OP: FnOnce(T) -> U,
        T: TypeFoldable<I>,
        U: TypeFoldable<I>,
        U: HasInterner<Interner = I>;
}

impl<T, I> CanonicalExt<T, I> for Canonical<T>
where
    T: HasInterner<Interner = I>,
    I: Interner,
{
    /// Maps the contents using `op`, but preserving the binders.
    ///
    /// NB. `op` will be invoked with an instantiated version of the
    /// canonical value, where inference variables (from a fresh
    /// inference context) are used in place of the quantified free
    /// variables. The result should be in terms of those same
    /// inference variables and will be re-canonicalized.
    fn map<OP, U>(self, interner: I, op: OP) -> Canonical<U>
    where
        OP: FnOnce(T) -> U,
        T: TypeFoldable<I>,
        U: TypeFoldable<I>,
        U: HasInterner<Interner = I>,
    {
        // Subtle: It is only quite rarely correct to apply `op` and
        // just re-use our existing binders. For that to be valid, the
        // result of `op` would have to ensure that it re-uses all the
        // existing free variables and in the same order. Otherwise,
        // the canonical form would be different: the variables might
        // be numbered differently, or some may not longer be used.
        // This would mean that two canonical values could no longer
        // be compared with `Eq`, which defeats a key invariant of the
        // `Canonical` type (indeed, its entire reason for existence).
        let mut infer = InferenceTable::new();
        let snapshot = infer.snapshot();
        let instantiated_value = infer.instantiate_canonical(interner, self);
        let mapped_value = op(instantiated_value);
        let result = infer.canonicalize(interner, mapped_value);
        infer.rollback_to(snapshot);
        result.quantified
    }
}

pub trait GoalExt<I: Interner> {
    fn into_peeled_goal(self, interner: I) -> UCanonical<InEnvironment<Goal<I>>>;
    fn into_closed_goal(self, interner: I) -> UCanonical<InEnvironment<Goal<I>>>;
}

impl<I: Interner> GoalExt<I> for Goal<I> {
    /// Returns a canonical goal in which the outermost `exists<>` and
    /// `forall<>` quantifiers (as well as implications) have been
    /// "peeled" and are converted into free universal or existential
    /// variables. Assumes that this goal is a "closed goal" which
    /// does not -- at present -- contain any variables. Useful for
    /// REPLs and tests but not much else.
    fn into_peeled_goal(self, interner: I) -> UCanonical<InEnvironment<Goal<I>>> {
        let mut infer = InferenceTable::new();
        let peeled_goal = {
            let mut env_goal = InEnvironment::new(&Environment::new(interner), self);
            loop {
                let InEnvironment { environment, goal } = env_goal;
                match goal.data(interner) {
                    GoalData::Quantified(QuantifierKind::ForAll, subgoal) => {
                        let subgoal =
                            infer.instantiate_binders_universally(interner, subgoal.clone());
                        env_goal = InEnvironment::new(&environment, subgoal);
                    }

                    GoalData::Quantified(QuantifierKind::Exists, subgoal) => {
                        let subgoal =
                            infer.instantiate_binders_existentially(interner, subgoal.clone());
                        env_goal = InEnvironment::new(&environment, subgoal);
                    }

                    GoalData::Implies(wc, subgoal) => {
                        let new_environment =
                            environment.add_clauses(interner, wc.iter(interner).cloned());
                        env_goal = InEnvironment::new(&new_environment, Goal::clone(subgoal));
                    }

                    _ => break InEnvironment::new(&environment, goal),
                }
            }
        };
        let canonical = infer.canonicalize(interner, peeled_goal).quantified;
        InferenceTable::u_canonicalize(interner, &canonical).quantified
    }

    /// Given a goal with no free variables (a "closed" goal), creates
    /// a canonical form suitable for solving. This is a suitable
    /// choice if you don't actually care about the values of any of
    /// the variables within; otherwise, you might want
    /// `into_peeled_goal`.
    ///
    /// # Panics
    ///
    /// Will panic if this goal does in fact contain free variables.
    fn into_closed_goal(self, interner: I) -> UCanonical<InEnvironment<Goal<I>>> {
        let mut infer = InferenceTable::new();
        let env_goal = InEnvironment::new(&Environment::new(interner), self);
        let canonical_goal = infer.canonicalize(interner, env_goal).quantified;
        InferenceTable::u_canonicalize(interner, &canonical_goal).quantified
    }
}
