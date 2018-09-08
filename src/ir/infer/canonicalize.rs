use chalk_engine::fallible::*;
use ir::fold::{DefaultTypeFolder, ExistentialFolder, Fold, UniversalFolder};
use ir::fold::shift::Shift;
use ir::*;
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
    crate fn canonicalize<T: Fold>(&mut self, value: &T) -> Canonicalized<T::Result> {
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
crate struct Canonicalized<T> {
    /// The canonicalized result.
    crate quantified: Canonical<T>,

    /// The free existential variables, along with the universes they inhabit.
    pub(in ir) free_vars: Vec<ParameterInferenceVariable>,

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

impl<'q> DefaultTypeFolder for Canonicalizer<'q> {}

impl<'q> UniversalFolder for Canonicalizer<'q> {
    fn fold_free_universal_ty(&mut self, universe: UniverseIndex, _binders: usize) -> Fallible<Ty> {
        self.max_universe = max(self.max_universe, universe);
        Ok(TypeName::ForAll(universe).to_ty())
    }

    fn fold_free_universal_lifetime(
        &mut self,
        universe: UniverseIndex,
        _binders: usize,
    ) -> Fallible<Lifetime> {
        self.max_universe = max(self.max_universe, universe);
        Ok(universe.to_lifetime())
    }
}

impl<'q> ExistentialFolder for Canonicalizer<'q> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        debug_heading!(
            "fold_free_existential_ty(depth={:?}, binders={:?})",
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

    fn fold_free_existential_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        debug_heading!(
            "fold_free_existential_lifetime(depth={:?}, binders={:?})",
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
