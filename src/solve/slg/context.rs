use crate::fallible::Fallible;
use crate::ir::{Canonical, ConstrainedSubst, Environment, Goal, InEnvironment, Lifetime,
                ParameterKind, Substitution, Ty, UCanonical, UniverseIndex};
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::infer::ucanonicalize::UCanonicalized;
use crate::solve::infer::unify::UnificationResult;
use crate::solve::slg::CanonicalGoal;
use crate::fold::Fold;
use crate::zip::Zip;
use std::fmt::Debug;
use std::sync::Arc;

crate trait Context: Copy + Debug {
    type InferenceTable: InferenceTable<Self>;
    type InferenceVariable: InferenceVariable<Self>;
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
    fn canonicalize_goal(&mut self, value: &InEnvironment<Goal>) -> Canonical<InEnvironment<Goal>>;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        value: &ConstrainedSubst,
    ) -> Canonical<ConstrainedSubst>;

    // Used by: logic
    fn u_canonicalize_goal(&mut self, value: &CanonicalGoal) -> UCanonicalized<InEnvironment<Goal>>;

    // Used by: logic
    fn fresh_subst(&mut self, binders: &[ParameterKind<UniverseIndex>]) -> Substitution;

    // Used by: logic
    fn invert_goal(&mut self, value: &InEnvironment<Goal>) -> Option<InEnvironment<Goal>>;

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
        environment: &Arc<Environment>,
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

#[derive(Copy, Clone, Debug, Default)]
pub struct SlgContext;

impl Context for SlgContext {
    type InferenceTable = ::crate::solve::infer::InferenceTable;
    type InferenceVariable = ::crate::solve::infer::var::InferenceVariable;
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

    fn canonicalize_goal(&mut self, value: &InEnvironment<Goal>) -> Canonical<InEnvironment<Goal>> {
        self.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        value: &ConstrainedSubst,
    ) -> Canonical<ConstrainedSubst> {
        self.canonicalize(value).quantified
    }

    fn u_canonicalize_goal(&mut self, value: &CanonicalGoal) -> UCanonicalized<InEnvironment<Goal>> {
        self.u_canonicalize(value)
    }

    fn invert_goal(&mut self, value: &InEnvironment<Goal>) -> Option<InEnvironment<Goal>> {
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
        environment: &Arc<Environment>,
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
