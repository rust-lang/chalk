use errors::*;
use fold::{DefaultTypeFolder, ExistentialFolder, Fold, UniversalFolder};
use ir::*;
use std::cmp::max;

use super::{InferenceTable, TyInferenceVariable, LifetimeInferenceVariable,
            ParameterInferenceVariable};
use super::var::InferenceValue;

impl InferenceTable {
    /// Given a value `value` with variables in it, replaces those variables
    /// with their instantiated values; any variables not yet instantiated are
    /// replaced with a small integer index 0..N in order of appearance. The
    /// result is a canonicalized representation of `value`.
    ///
    /// Example:
    ///
    ///    ?22: Foo<?23>
    ///
    /// would be quantified to
    ///
    ///    Canonical { value: `?0: Foo<?1>`, binders: [ui(?22), ui(?23)] }
    ///
    /// where `ui(?22)` and `ui(?23)` are the universe indices of `?22` and
    /// `?23` respectively.
    ///
    /// A substitution mapping from the free variables to their re-bound form is
    /// also returned.
    pub fn canonicalize<T: Fold>(&mut self, value: &T) -> Canonicalized<T::Result> {
        debug!("canonicalize({:#?})", value);
        let mut q = Canonicalizer {
            table: self,
            free_vars: Vec::new(),
            max_universe: UniverseIndex::root(),
        };
        let value = value.fold_with(&mut q, 0).unwrap();
        let free_vars = q.free_vars.clone();
        let max_universe = q.max_universe;

        Canonicalized {
            quantified: Canonical { value, binders: q.into_binders() },
            max_universe,
            free_vars,
        }
    }
}

pub struct Canonicalized<T> {
    /// The canonicalized result.
    pub quantified: Canonical<T>,

    /// The free existential variables, along with the universes they inhabit.
    pub free_vars: Vec<ParameterInferenceVariable>,

    /// The maximum universe of any universally quantified variables
    /// encountered.
    pub max_universe: UniverseIndex,
}

impl<T> Canonicalized<T> {
    /// Returns a tuple of:
    ///
    /// - the quantified value Q
    /// - a substitution S which, if applied to Q, would yield the original value V
    ///   from which Q was derived.
    ///
    /// NB. You can apply a substitution with `Q.instantiate_with_subst(&S)`.
    pub fn into_quantified_and_subst(self) -> (Canonical<T>, Substitution) {
        let mut subst = Substitution::empty();
        for (i, free_var) in self.free_vars.iter().enumerate() {
            match free_var {
                ParameterKind::Ty(v) => {
                    subst.tys.insert(TyInferenceVariable::from_depth(i), v.to_ty());
                }
                ParameterKind::Lifetime(l) => {
                    subst.lifetimes.insert(LifetimeInferenceVariable::from_depth(i), l.to_lifetime());
                }
            }
        }

        (self.quantified, subst)
    }
}

struct Canonicalizer<'q> {
    table: &'q mut InferenceTable,
    free_vars: Vec<ParameterInferenceVariable>,
    max_universe: UniverseIndex,
}

impl<'q> Canonicalizer<'q> {
    fn into_binders(self) -> Vec<ParameterKind<UniverseIndex>> {
        let Canonicalizer { table, free_vars, max_universe: _ } = self;
        free_vars.into_iter()
            .map(|p_v| match p_v {
                     ParameterKind::Ty(v) => {
                         debug_assert!(table.ty_unify.find(v) == v);
                         match table.ty_unify.probe_value(v) {
                             InferenceValue::Unbound(ui) => ParameterKind::Ty(ui),
                             InferenceValue::Bound(_) => panic!("free var now bound"),
                         }
                     }

                     ParameterKind::Lifetime(v) => {
                         match table.lifetime_unify.probe_value(v) {
                             InferenceValue::Unbound(ui) => ParameterKind::Lifetime(ui),
                             InferenceValue::Bound(_) => panic!("free var now bound"),
                         }
                     }
                 })
            .collect()
    }

    fn add(&mut self, free_var: ParameterInferenceVariable) -> usize {
        match self.free_vars.iter().position(|&v| v == free_var) {
            Some(i) => i,
            None => {
                let next_index = self.free_vars.len();
                self.free_vars.push(free_var);
                next_index
            }
        }
    }
}

impl<'q> DefaultTypeFolder for Canonicalizer<'q> { }

impl<'q> UniversalFolder for Canonicalizer<'q> {
    fn fold_free_universal_ty(&mut self, universe: UniverseIndex, _binders: usize) -> Result<Ty> {
        self.max_universe = max(self.max_universe, universe);
        Ok(TypeName::ForAll(universe).to_ty())
    }

    fn fold_free_universal_lifetime(&mut self, universe: UniverseIndex, _binders: usize) -> Result<Lifetime> {
        self.max_universe = max(self.max_universe, universe);
        Ok(universe.to_lifetime())
    }
}

impl<'q> ExistentialFolder for Canonicalizer<'q> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        debug_heading!("fold_free_existential_ty(depth={:?}, binders={:?})", depth, binders);
        let var = TyInferenceVariable::from_depth(depth);
        match self.table.probe_var(var) {
            Some(ty) => {
                debug!("bound to {:?}", ty);
                Ok(ty.fold_with(self, 0)?.up_shift(binders))
            }
            None => {
                // If this variable is not yet bound, find its
                // canonical index `root_var` in the union-find table,
                // and then map `root_var` to a fresh index that is
                // unique to this quantification.
                let free_var = ParameterKind::Ty(self.table.ty_unify.find(var));
                let position = self.add(free_var);
                debug!("not yet unified: position={:?}", position);
                Ok(TyInferenceVariable::from_depth(position + binders).to_ty())
            }
        }
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        debug_heading!("fold_free_existential_lifetime(depth={:?}, binders={:?})", depth, binders);
        let var = LifetimeInferenceVariable::from_depth(depth);
        match self.table.probe_lifetime_var(var) {
            Some(l) => {
                debug!("bound to {:?}", l);
                Ok(l.fold_with(self, 0)?.up_shift(binders))
            }
            None => {
                let free_var = ParameterKind::Lifetime(self.table.lifetime_unify.find(var));
                let position = self.add(free_var);
                debug!("not yet unified: position={:?}", position);
                Ok(LifetimeInferenceVariable::from_depth(position + binders).to_lifetime())
            }
        }
    }
}
