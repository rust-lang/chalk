use chalk_engine::fallible::*;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;
use std::cmp::max;

use super::{EnaVariable, InferenceTable, ParameterEnaVariable};

impl<I: Interner> InferenceTable<I> {
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
    pub(crate) fn canonicalize<T>(&mut self, interner: &I, value: &T) -> Canonicalized<T::Result>
    where
        T: Fold<I>,
        T::Result: HasInterner<Interner = I>,
    {
        debug!("canonicalize({:#?})", value);
        let mut q = Canonicalizer {
            table: self,
            free_vars: Vec::new(),
            max_universe: UniverseIndex::root(),
            interner,
        };
        let value = value.fold_with(&mut q, DebruijnIndex::INNERMOST).unwrap();
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
pub(crate) struct Canonicalized<T: HasInterner> {
    /// The canonicalized result.
    pub(crate) quantified: Canonical<T>,

    /// The free existential variables, along with the universes they inhabit.
    pub(crate) free_vars: Vec<ParameterEnaVariable<T::Interner>>,

    /// The maximum universe of any universally quantified variables
    /// encountered.
    max_universe: UniverseIndex,
}

struct Canonicalizer<'q, I: Interner> {
    table: &'q mut InferenceTable<I>,
    free_vars: Vec<ParameterEnaVariable<I>>,
    max_universe: UniverseIndex,
    interner: &'q I,
}

impl<'q, I: Interner> Canonicalizer<'q, I> {
    fn into_binders(self) -> CanonicalVarKinds<I> {
        let Canonicalizer {
            table,
            free_vars,
            interner,
            ..
        } = self;
        CanonicalVarKinds::from(
            interner,
            free_vars
                .into_iter()
                .map(|p_v| p_v.map(|v| table.universe_of_unbound_var(v))),
        )
    }

    fn add(&mut self, free_var: ParameterEnaVariable<I>) -> usize {
        self.free_vars
            .iter()
            .position(|&v| v == free_var)
            .unwrap_or_else(|| {
                let next_index = self.free_vars.len();
                self.free_vars.push(free_var);
                next_index
            })
    }
}

impl<'i, I: Interner> Folder<'i, I> for Canonicalizer<'i, I>
where
    I: 'i,
{
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let interner = self.interner;
        self.max_universe = max(self.max_universe, universe.ui);
        Ok(universe.to_ty(interner))
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        let interner = self.interner;
        self.max_universe = max(self.max_universe, universe.ui);
        Ok(universe.to_lifetime(interner))
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }

    fn fold_inference_ty(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        debug_heading!(
            "fold_inference_ty(depth={:?}, binders={:?})",
            var,
            outer_binder
        );
        let interner = self.interner;
        let var = EnaVariable::from(var);
        match self.table.probe_ty_var(interner, var) {
            Some(ty) => {
                debug!("bound to {:?}", ty);
                Ok(ty
                    .fold_with(self, DebruijnIndex::INNERMOST)?
                    .shifted_in_from(interner, outer_binder))
            }
            None => {
                // If this variable is not yet bound, find its
                // canonical index `root_var` in the union-find table,
                // and then map `root_var` to a fresh index that is
                // unique to this quantification.
                let free_var = ParameterKind::Ty(self.table.unify.find(var));
                let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.add(free_var));
                debug!("not yet unified: position={:?}", bound_var);
                Ok(TyData::BoundVar(bound_var.shifted_in_from(outer_binder)).intern(interner))
            }
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        debug_heading!(
            "fold_inference_lifetime(depth={:?}, outer_binder={:?})",
            var,
            outer_binder
        );
        let interner = self.interner;
        let var = EnaVariable::from(var);
        match self.table.probe_lifetime_var(interner, var) {
            Some(l) => {
                debug!("bound to {:?}", l);
                Ok(l.fold_with(self, DebruijnIndex::INNERMOST)?
                    .shifted_in_from(interner, outer_binder))
            }
            None => {
                let free_var = ParameterKind::Lifetime(self.table.unify.find(var));
                let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.add(free_var));
                debug!("not yet unified: position={:?}", bound_var);
                Ok(
                    LifetimeData::BoundVar(bound_var.shifted_in_from(outer_binder))
                        .intern(interner),
                )
            }
        }
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
