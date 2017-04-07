use fold::*;
use ir::QueryBinders;
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
        let mut vars = QueryBinders::default();
        for var in universes.into_iter().map(|u| self.new_parameter_variable(u)) {
            match var {
                ParameterKind::Ty(ty)       => vars.tys.push(ty),
                ParameterKind::Lifetime(l)  => vars.lifetimes.push(l),
                ParameterKind::Krate(krate) => vars.krates.push(krate),
            }
        }
        debug!("instantiate: vars={:?}", vars);
        let mut instantiator = Instantiator { vars: vars };
        arg.fold_with(&mut instantiator, 0).expect("")
    }
}

struct Instantiator {
    vars: QueryBinders<TyInferenceVariable, LifetimeInferenceVariable, KrateInferenceVariable>,
}

/// Folder: when we encounter a free variable (of any kind) with index
/// `i`, we want to map anything in the first N binders to
/// `self.vars[i]`. Everything else stays intact, but we have to
/// subtract `self.vars.len()` to account for the binders we are
/// instantiating.
impl Folder for Instantiator {
    fn fold_free_var(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        if depth < self.vars.tys.len() {
            Ok(self.vars.tys[depth].to_ty().up_shift(binders))
        } else {
            Ok(Ty::Var(depth + binders - self.vars.tys.len())) // see comment above
        }
    }

    fn fold_free_lifetime_var(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        if depth < self.vars.lifetimes.len() {
            Ok(self.vars.lifetimes[depth].to_lifetime().up_shift(binders))
        } else {
            Ok(Lifetime::Var(depth + binders - self.vars.lifetimes.len())) // see comment above
        }
    }

    fn fold_free_krate_var(&mut self, depth: usize, binders: usize) -> Result<Krate> {
        if depth < self.vars.krates.len() {
            Ok(self.vars.krates[depth].to_krate().up_shift(binders))
        } else {
            Ok(Krate::Var(depth + binders - self.vars.krates.len())) // see comment above
        }
    }
}
