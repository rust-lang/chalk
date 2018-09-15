use chalk_engine::fallible::*;
use chalk_ir::fold::{DefaultTypeFolder, FreeVarFolder, Fold, PlaceholderFolder};
use chalk_ir::fold::shift::Shift;
use chalk_ir::*;
use std::cmp::max;

use super::{InferenceTable, InferenceVariable, ParameterInferenceVariable};

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
            quantified: Canonical {
                value,
                binders: q.into_binders(),
            },
            max_universe,
            free_vars,
        }
    }
}

#[derive(Debug)]
pub struct Canonicalized<T> {
    /// The canonicalized result.
    pub quantified: Canonical<T>,

    /// The free existential variables, along with the universes they inhabit.
    crate free_vars: Vec<ParameterInferenceVariable>,

    /// The maximum universe of any universally quantified variables
    /// encountered.
    max_universe: UniverseIndex,
}

struct Canonicalizer<'q> {
    table: &'q mut InferenceTable,
    free_vars: Vec<ParameterInferenceVariable>,
    max_universe: UniverseIndex,
}

impl<'q> Canonicalizer<'q> {
    fn into_binders(self) -> Vec<ParameterKind<UniverseIndex>> {
        let Canonicalizer {
            table,
            free_vars,
            max_universe: _,
        } = self;
        free_vars
            .into_iter()
            .map(|p_v| p_v.map(|v| table.universe_of_unbound_var(v)))
            .collect()
    }

    fn add(&mut self, free_var: ParameterInferenceVariable) -> usize {
        self.free_vars.iter().position(|&v| v == free_var).unwrap_or_else(|| {
            let next_index = self.free_vars.len();
            self.free_vars.push(free_var);
            next_index
        })
    }
}

impl<'q> DefaultTypeFolder for Canonicalizer<'q> {}

impl<'q> PlaceholderFolder for Canonicalizer<'q> {
    fn fold_free_placeholder_ty(&mut self, universe: PlaceholderIndex, _binders: usize) -> Fallible<Ty> {
        self.max_universe = max(self.max_universe, universe.ui);
        Ok(universe.to_ty())
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<Lifetime> {
        self.max_universe = max(self.max_universe, universe.ui);
        Ok(universe.to_lifetime())
    }
}

impl<'q> FreeVarFolder for Canonicalizer<'q> {
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        debug_heading!(
            "fold_free_var_ty(depth={:?}, binders={:?})",
            depth,
            binders
        );
        let var = InferenceVariable::from_depth(depth);
        match self.table.probe_ty_var(var) {
            Some(ty) => {
                debug!("bound to {:?}", ty);
                Ok(ty.fold_with(self, 0)?.shifted_in(binders))
            }
            None => {
                // If this variable is not yet bound, find its
                // canonical index `root_var` in the union-find table,
                // and then map `root_var` to a fresh index that is
                // unique to this quantification.
                let free_var = ParameterKind::Ty(self.table.unify.find(var));
                let position = self.add(free_var);
                debug!("not yet unified: position={:?}", position);
                Ok(InferenceVariable::from_depth(position + binders).to_ty())
            }
        }
    }

    fn fold_free_var_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        debug_heading!(
            "fold_free_var_lifetime(depth={:?}, binders={:?})",
            depth,
            binders
        );
        let var = InferenceVariable::from_depth(depth);
        match self.table.probe_lifetime_var(var) {
            Some(l) => {
                debug!("bound to {:?}", l);
                Ok(l.fold_with(self, 0)?.shifted_in(binders))
            }
            None => {
                let free_var = ParameterKind::Lifetime(self.table.unify.find(var));
                let position = self.add(free_var);
                debug!("not yet unified: position={:?}", position);
                Ok(InferenceVariable::from_depth(position + binders).to_lifetime())
            }
        }
    }
}
