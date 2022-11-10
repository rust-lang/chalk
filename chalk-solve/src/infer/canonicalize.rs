use crate::debug_span;
use chalk_derive::FallibleTypeFolder;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::{TypeFoldable, TypeFolder};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;
use std::cmp::max;
use tracing::{debug, instrument};

use super::{InferenceTable, ParameterEnaVariable};

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
    pub fn canonicalize<T>(&mut self, interner: I, value: T) -> Canonicalized<T>
    where
        T: TypeFoldable<I>,
        T: HasInterner<Interner = I>,
    {
        debug_span!("canonicalize", "{:#?}", value);
        let mut q = Canonicalizer {
            table: self,
            free_vars: Vec::new(),
            max_universe: UniverseIndex::root(),
            interner,
        };
        let value = value
            .try_fold_with(&mut q, DebruijnIndex::INNERMOST)
            .unwrap();
        let free_vars = q.free_vars.clone();

        Canonicalized {
            quantified: Canonical {
                value,
                binders: q.into_binders(),
            },
            free_vars,
        }
    }
}

#[derive(Debug)]
pub struct Canonicalized<T: HasInterner> {
    /// The canonicalized result.
    pub quantified: Canonical<T>,

    /// The free existential variables, along with the universes they inhabit.
    pub free_vars: Vec<ParameterEnaVariable<T::Interner>>,
}

#[derive(FallibleTypeFolder)]
struct Canonicalizer<'q, I: Interner> {
    table: &'q mut InferenceTable<I>,
    free_vars: Vec<ParameterEnaVariable<I>>,
    max_universe: UniverseIndex,
    interner: I,
}

impl<'q, I: Interner> Canonicalizer<'q, I> {
    fn into_binders(self) -> CanonicalVarKinds<I> {
        let Canonicalizer {
            table,
            free_vars,
            interner,
            ..
        } = self;
        CanonicalVarKinds::from_iter(
            interner,
            free_vars
                .into_iter()
                .map(|p_v| p_v.map(|v| table.universe_of_unbound_var(v))),
        )
    }

    fn add(&mut self, free_var: ParameterEnaVariable<I>) -> usize {
        self.max_universe = max(
            self.max_universe,
            self.table.universe_of_unbound_var(*free_var.skip_kind()),
        );

        self.free_vars
            .iter()
            .position(|v| v.skip_kind() == free_var.skip_kind())
            .unwrap_or_else(|| {
                let next_index = self.free_vars.len();
                self.free_vars.push(free_var);
                next_index
            })
    }
}

impl<'i, I: Interner> TypeFolder<I> for Canonicalizer<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn TypeFolder<I> {
        self
    }

    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Ty<I> {
        let interner = self.interner;
        self.max_universe = max(self.max_universe, universe.ui);
        universe.to_ty(interner)
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        let interner = self.interner;
        self.max_universe = max(self.max_universe, universe.ui);
        universe.to_lifetime(interner)
    }

    fn fold_free_placeholder_const(
        &mut self,
        ty: Ty<I>,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Const<I> {
        let interner = self.interner;
        self.max_universe = max(self.max_universe, universe.ui);
        universe.to_const(interner, ty)
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }

    #[instrument(level = "debug", skip(self))]
    fn fold_inference_ty(
        &mut self,
        var: InferenceVar,
        kind: TyVariableKind,
        outer_binder: DebruijnIndex,
    ) -> Ty<I> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(ty) => {
                let ty = ty.assert_ty_ref(interner);
                debug!("bound to {:?}", ty);
                ty.clone()
                    .fold_with(self, DebruijnIndex::INNERMOST)
                    .shifted_in_from(interner, outer_binder)
            }
            None => {
                // If this variable is not yet bound, find its
                // canonical index `root_var` in the union-find table,
                // and then map `root_var` to a fresh index that is
                // unique to this quantification.
                let free_var =
                    ParameterEnaVariable::new(VariableKind::Ty(kind), self.table.unify.find(var));

                let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.add(free_var));
                debug!(position=?bound_var, "not yet unified");
                TyKind::BoundVar(bound_var.shifted_in_from(outer_binder)).intern(interner)
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(l) => {
                let l = l.assert_lifetime_ref(interner);
                debug!("bound to {:?}", l);
                l.clone()
                    .fold_with(self, DebruijnIndex::INNERMOST)
                    .shifted_in_from(interner, outer_binder)
            }
            None => {
                let free_var =
                    ParameterEnaVariable::new(VariableKind::Lifetime, self.table.unify.find(var));
                let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.add(free_var));
                debug!(position=?bound_var, "not yet unified");
                LifetimeData::BoundVar(bound_var.shifted_in_from(outer_binder)).intern(interner)
            }
        }
    }

    #[instrument(level = "debug", skip(self, ty))]
    fn fold_inference_const(
        &mut self,
        ty: Ty<I>,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Const<I> {
        let interner = self.interner;
        match self.table.probe_var(var) {
            Some(c) => {
                let c = c.assert_const_ref(interner);
                debug!("bound to {:?}", c);
                c.clone()
                    .fold_with(self, DebruijnIndex::INNERMOST)
                    .shifted_in_from(interner, outer_binder)
            }
            None => {
                let free_var = ParameterEnaVariable::new(
                    VariableKind::Const(ty.clone()),
                    self.table.unify.find(var),
                );
                let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.add(free_var));
                debug!(position = ?bound_var, "not yet unified");
                bound_var
                    .shifted_in_from(outer_binder)
                    .to_const(interner, ty)
            }
        }
    }

    fn interner(&self) -> I {
        self.interner
    }
}
