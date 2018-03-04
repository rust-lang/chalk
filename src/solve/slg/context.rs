use crate::fallible::Fallible;
use crate::ir::{Canonical, ConstrainedSubst, DomainGoal, Environment, Goal, InEnvironment,
                Lifetime, ParameterKind, ProgramClause, ProgramEnvironment, Substitution, Ty,
                UCanonical, UniverseIndex};
use crate::ir::could_match::CouldMatch;
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::infer::ucanonicalize::UCanonicalized;
use crate::solve::infer::unify::UnificationResult;
use crate::solve::slg::{CanonicalGoal, UCanonicalGoal};
use crate::fold::Fold;
use crate::zip::Zip;
use std::fmt::Debug;
use std::sync::Arc;

// Notes:
//
// How do we use `Substitution`:
// - we use them as key in a hashmap
// - we ask if they are trivial:
//   - for a given goal
//   - with no region constraints
// - we apply them to a given canonical goal to yield an instantiated goal
//
// How do we use a `UniverseMap`:
// - we apply it to the result we get back from an answer
// - which is then fed into `apply_answer_subst`
//
// How do we use region constraints:
// - opaquely, I imagine
//
// It seems clear we can extract a

crate trait Context: Sized {
    type InferenceTable: InferenceTable<Self>;
    type InferenceVariable: InferenceVariable<Self>;

    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &UCanonicalGoal<DomainGoal>) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(&self, goal: &InEnvironment<DomainGoal>) -> Vec<ProgramClause<DomainGoal>>;
}

crate trait InferenceVariable<C: Context>: Copy {
    fn to_ty(self) -> Ty;
    fn to_lifetime(self) -> Lifetime;
}

crate trait InferenceTable<C: Context>: Clone {
    fn new() -> Self;

    // Used by: simplify
    fn instantiate_binders_universally<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold;

    // Used by: logic
    fn instantiate_universes<'v, T>(&mut self, value: &'v UCanonical<T>) -> &'v Canonical<T>;

    // Used by: truncate
    fn max_universe(&self) -> UniverseIndex;

    // Used by: aggregate, truncate
    fn new_variable(&mut self, ui: UniverseIndex) -> C::InferenceVariable;

    // Used by: resolvent
    fn normalize_lifetime(&mut self, leaf: &Lifetime, binders: usize) -> Option<Lifetime>;

    // Used by: resolvent, truncate
    fn normalize_shallow(&mut self, leaf: &Ty, binders: usize) -> Option<Ty>;

    // Used by: resolvent, logic (but for debugging only)
    fn normalize_deep<T: Fold>(&mut self, value: &T) -> T::Result;

    // Used by: logic
    fn canonicalize_goal(
        &mut self,
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Canonical<InEnvironment<Goal<DomainGoal>>>;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        value: &ConstrainedSubst,
    ) -> Canonical<ConstrainedSubst>;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &CanonicalGoal<DomainGoal>,
    ) -> UCanonicalized<InEnvironment<Goal<DomainGoal>>>;

    // Used by: logic
    fn fresh_subst(&mut self, binders: &[ParameterKind<UniverseIndex>]) -> Substitution;

    // Used by: logic
    fn invert_goal(
        &mut self,
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Option<InEnvironment<Goal<DomainGoal>>>;

    // Used by: simplify, resolvent
    fn instantiate_binders_existentially<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold;

    // Used by: simplify, resolvent, truncate
    fn unify<T>(
        &mut self,
        environment: &Arc<Environment<DomainGoal>>,
        a: &T,
        b: &T,
    ) -> Fallible<UnificationResult>
    where
        T: ?Sized + Zip;

    // Used by: resolvent
    fn instantiate_canonical<T>(&mut self, bound: &Canonical<T>) -> T::Result
    where
        T: Fold + Debug;
}

///////////////////////////////////////////////////////////////////////////

#[allow(dead_code)] // for some reason rustc reports this as dead??
pub type SlgContext = Arc<ProgramEnvironment<DomainGoal>>;

impl Context for Arc<ProgramEnvironment<DomainGoal>> {
    type InferenceTable = ::crate::solve::infer::InferenceTable;
    type InferenceVariable = ::crate::solve::infer::var::InferenceVariable;

    fn is_coinductive(&self, goal: &UCanonicalGoal<DomainGoal>) -> bool {
        goal.is_coinductive(self)
    }

    fn program_clauses(&self, goal: &InEnvironment<DomainGoal>) -> Vec<ProgramClause<DomainGoal>> {
        let &InEnvironment {
            ref environment,
            ref goal,
        } = goal;

        let environment_clauses = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .map(|env_clause| env_clause.clone().into_program_clause());

        let program_clauses = self.program_clauses
            .iter()
            .filter(|clause| clause.could_match(goal))
            .cloned();

        environment_clauses.chain(program_clauses).collect()
    }
}

impl InferenceTable<SlgContext> for ::crate::solve::infer::InferenceTable {
    fn new() -> Self {
        Self::new()
    }

    fn fresh_subst(&mut self, binders: &[ParameterKind<UniverseIndex>]) -> Substitution {
        self.fresh_subst(binders)
    }

    fn instantiate_binders_universally<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold,
    {
        self.instantiate_binders_universally(arg)
    }

    fn instantiate_universes<'v, T>(&mut self, value: &'v UCanonical<T>) -> &'v Canonical<T> {
        self.instantiate_universes(value)
    }

    fn max_universe(&self) -> UniverseIndex {
        self.max_universe()
    }

    fn new_variable(&mut self, ui: UniverseIndex) -> ::crate::solve::infer::var::InferenceVariable {
        self.new_variable(ui)
    }

    fn normalize_lifetime(&mut self, leaf: &Lifetime, binders: usize) -> Option<Lifetime> {
        self.normalize_lifetime(leaf, binders)
    }

    fn normalize_shallow(&mut self, leaf: &Ty, binders: usize) -> Option<Ty> {
        self.normalize_shallow(leaf, binders)
    }

    fn normalize_deep<T: Fold>(&mut self, value: &T) -> T::Result {
        self.normalize_deep(value)
    }

    fn canonicalize_goal(
        &mut self,
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        value: &ConstrainedSubst,
    ) -> Canonical<ConstrainedSubst> {
        self.canonicalize(value).quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &CanonicalGoal<DomainGoal>,
    ) -> UCanonicalized<InEnvironment<Goal<DomainGoal>>> {
        self.u_canonicalize(value)
    }

    fn invert_goal(
        &mut self,
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Option<InEnvironment<Goal<DomainGoal>>> {
        self.invert(value)
    }

    fn instantiate_binders_existentially<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold,
    {
        self.instantiate_binders_existentially(arg)
    }

    fn instantiate_canonical<T>(&mut self, bound: &Canonical<T>) -> T::Result
    where
        T: Fold + Debug,
    {
        self.instantiate_canonical(bound)
    }

    fn unify<T>(
        &mut self,
        environment: &Arc<Environment<DomainGoal>>,
        a: &T,
        b: &T,
    ) -> Fallible<UnificationResult>
    where
        T: ?Sized + Zip,
    {
        self.unify(environment, a, b)
    }
}

impl InferenceVariable<SlgContext> for ::crate::solve::infer::var::InferenceVariable {
    fn to_ty(self) -> Ty {
        self.to_ty()
    }

    fn to_lifetime(self) -> Lifetime {
        self.to_lifetime()
    }
}

crate mod prelude {
    #![allow(unused_imports)] // rustc bug

    crate use super::Context;
    crate use super::InferenceTable;
    crate use super::InferenceVariable;
}
