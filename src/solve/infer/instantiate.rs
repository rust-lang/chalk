use fold::*;
use std::fmt::Debug;

use super::*;

impl InferenceTable {
    /// Create a instance of `arg` where each variable is replaced with
    /// a fresh inference variable of suitable kind.
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold + Debug,
              U: IntoIterator<Item = ParameterKind<UniverseIndex>>
    {
        debug!("instantiate(arg={:?})", arg);
        let vars: Vec<_> = universes.into_iter()
            .map(|u| self.new_parameter_variable(u))
            .collect();
        debug!("instantiate: vars={:?}", vars);
        let mut instantiator = Instantiator { vars };
        arg.fold_with(&mut instantiator, 0).expect("")
    }

    /// Variant on `instantiate` that takes a `Canonical<T>`.
    pub fn instantiate_canonical<T>(&mut self, bound: &Canonical<T>)
                                    -> T::Result
        where T: Fold + Debug,
    {
        self.instantiate(bound.binders.iter().cloned(), &bound.value)
    }

    /// Instantiates `arg` with fresh existential variables in the
    /// given universe; the kinds of the variables are implied by
    /// `binders`. This is used to apply a universally quantified
    /// clause like `forall X, 'Y. P => Q`. Here the `binders`
    /// argument is referring to `X, 'Y`.
    pub fn instantiate_in<U, T>(&mut self,
                                universe: UniverseIndex,
                                binders: U,
                                arg: &T)
                                -> T::Result
        where T: Fold,
              U: IntoIterator<Item = ParameterKind<()>>
    {
        self.instantiate(binders.into_iter().map(|pk| pk.map(|_| universe)), arg)
    }

    /// Variant on `instantiate_in` that takes a `Binders<T>`.
    pub fn instantiate_binders_in<T>(&mut self,
                                     universe: UniverseIndex,
                                     arg: &Binders<T>)
                                     -> T::Result
        where T: Fold
    {
        self.instantiate_in(universe, arg.binders.iter().cloned(), &arg.value)
    }
}

struct Instantiator {
    vars: Vec<ParameterInferenceVariable>,
}

impl DefaultTypeFolder for Instantiator { }

/// When we encounter a free variable (of any kind) with index
/// `i`, we want to map anything in the first N binders to
/// `self.vars[i]`. Everything else stays intact, but we have to
/// subtract `self.vars.len()` to account for the binders we are
/// instantiating.
impl ExistentialFolder for Instantiator {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        if depth < self.vars.len() {
            Ok(self.vars[depth].as_ref().ty().unwrap().to_ty().up_shift(binders))
        } else {
            Ok(Ty::Var(depth + binders - self.vars.len())) // see comment above
        }
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        if depth < self.vars.len() {
            Ok(self.vars[depth].as_ref().lifetime().unwrap().to_lifetime().up_shift(binders))
        } else {
            Ok(Lifetime::Var(depth + binders - self.vars.len())) // see comment above
        }
    }
}

impl IdentityUniversalFolder for Instantiator { }
