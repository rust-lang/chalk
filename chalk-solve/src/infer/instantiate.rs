use chalk_ir::fold::*;
use chalk_ir::interner::HasInterner;
use std::fmt::Debug;
use tracing::instrument;

use super::*;

impl<I: Interner> InferenceTable<I> {
    /// Given the binders from a canonicalized value C, returns a
    /// substitution S mapping each free variable in C to a fresh
    /// inference variable. This substitution can then be applied to
    /// C, which would be equivalent to
    /// `self.instantiate_canonical(v)`.
    pub(super) fn fresh_subst(
        &mut self,
        interner: I,
        binders: &[CanonicalVarKind<I>],
    ) -> Substitution<I> {
        Substitution::from_iter(
            interner,
            binders.iter().map(|kind| {
                let param_infer_var = kind.map_ref(|&ui| self.new_variable(ui));
                param_infer_var.to_generic_arg(interner)
            }),
        )
    }

    /// Variant on `instantiate` that takes a `Canonical<T>`.
    pub fn instantiate_canonical<T>(&mut self, interner: I, bound: Canonical<T>) -> T
    where
        T: HasInterner<Interner = I> + TypeFoldable<I> + Debug,
    {
        let subst = self.fresh_subst(interner, bound.binders.as_slice(interner));
        subst.apply(bound.value, interner)
    }

    /// Instantiates `arg` with fresh existential variables in the
    /// given universe; the kinds of the variables are implied by
    /// `binders`. This is used to apply a universally quantified
    /// clause like `forall X, 'Y. P => Q`. Here the `binders`
    /// argument is referring to `X, 'Y`.
    fn instantiate_in<T>(
        &mut self,
        interner: I,
        universe: UniverseIndex,
        binders: impl Iterator<Item = VariableKind<I>>,
        arg: T,
    ) -> T
    where
        T: TypeFoldable<I>,
    {
        let binders: Vec<_> = binders
            .map(|pk| CanonicalVarKind::new(pk, universe))
            .collect();
        let subst = self.fresh_subst(interner, &binders);
        subst.apply(arg, interner)
    }

    /// Variant on `instantiate_in` that takes a `Binders<T>`.
    #[instrument(level = "debug", skip(self, interner))]
    pub fn instantiate_binders_existentially<T>(&mut self, interner: I, arg: Binders<T>) -> T
    where
        T: TypeFoldable<I> + HasInterner<Interner = I>,
    {
        let (value, binders) = arg.into_value_and_skipped_binders();

        let max_universe = self.max_universe;
        self.instantiate_in(
            interner,
            max_universe,
            binders.iter(interner).cloned(),
            value,
        )
    }

    #[instrument(level = "debug", skip(self, interner))]
    pub fn instantiate_binders_universally<T>(&mut self, interner: I, arg: Binders<T>) -> T
    where
        T: TypeFoldable<I> + HasInterner<Interner = I>,
    {
        let (value, binders) = arg.into_value_and_skipped_binders();

        let mut lazy_ui = None;
        let mut ui = || {
            lazy_ui.unwrap_or_else(|| {
                let ui = self.new_universe();
                lazy_ui = Some(ui);
                ui
            })
        };
        let parameters: Vec<_> = binders
            .iter(interner)
            .cloned()
            .enumerate()
            .map(|(idx, pk)| {
                let placeholder_idx = PlaceholderIndex { ui: ui(), idx };
                match pk {
                    VariableKind::Lifetime => {
                        let lt = placeholder_idx.to_lifetime(interner);
                        lt.cast(interner)
                    }
                    VariableKind::Ty(_) => placeholder_idx.to_ty(interner).cast(interner),
                    VariableKind::Const(ty) => {
                        placeholder_idx.to_const(interner, ty).cast(interner)
                    }
                }
            })
            .collect();
        Subst::apply(interner, &parameters, value)
    }
}
