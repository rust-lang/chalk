use chalk_ir::fold::*;
use chalk_ir::interner::HasInterner;
use std::fmt::Debug;

use super::*;

impl<I: Interner> InferenceTable<I> {
    /// Given the binders from a canonicalized value C, returns a
    /// substitution S mapping each free variable in C to a fresh
    /// inference variable. This substitution can then be applied to
    /// C, which would be equivalent to
    /// `self.instantiate_canonical(v)`.
    pub(crate) fn fresh_subst(
        &mut self,
        interner: &I,
        binders: &[CanonicalVarKind<I>],
    ) -> Substitution<I> {
        Substitution::from(
            interner,
            binders.iter().map(|kind| {
                let param_infer_var = kind.map_ref(|&ui| self.new_variable(ui));
                param_infer_var.to_generic_arg(interner)
            }),
        )
    }

    /// Variant on `instantiate` that takes a `Canonical<T>`.
    pub(crate) fn instantiate_canonical<T>(
        &mut self,
        interner: &I,
        bound: &Canonical<T>,
    ) -> T::Result
    where
        T: HasInterner<Interner = I> + Fold<I> + Debug,
    {
        let subst = self.fresh_subst(interner, &bound.binders.as_slice(interner));
        subst.apply(&bound.value, interner)
    }

    /// Instantiates `arg` with fresh existential variables in the
    /// given universe; the kinds of the variables are implied by
    /// `binders`. This is used to apply a universally quantified
    /// clause like `forall X, 'Y. P => Q`. Here the `binders`
    /// argument is referring to `X, 'Y`.
    pub(crate) fn instantiate_in<U, T>(
        &mut self,
        interner: &I,
        universe: UniverseIndex,
        binders: U,
        arg: &T,
    ) -> T::Result
    where
        T: Fold<I>,
        U: IntoIterator<Item = VariableKind<I>>,
    {
        let binders: Vec<_> = binders
            .into_iter()
            .map(|pk| CanonicalVarKind::new(pk, universe))
            .collect();
        let subst = self.fresh_subst(interner, &binders);
        subst.apply(&arg, interner)
    }

    /// Variant on `instantiate_in` that takes a `Binders<T>`.
    pub(crate) fn instantiate_binders_existentially<'a, T>(
        &mut self,
        interner: &'a I,
        arg: impl IntoBindersAndValue<'a, I, Value = T>,
    ) -> T::Result
    where
        T: Fold<I>,
    {
        let (binders, value) = arg.into_binders_and_value(interner);
        let max_universe = self.max_universe;
        self.instantiate_in(interner, max_universe, binders, &value)
    }

    pub(crate) fn instantiate_binders_universally<'a, T>(
        &mut self,
        interner: &'a I,
        arg: impl IntoBindersAndValue<'a, I, Value = T>,
    ) -> T::Result
    where
        T: Fold<I>,
    {
        let (binders, value) = arg.into_binders_and_value(interner);
        let ui = self.new_universe();
        let parameters: Vec<_> = binders
            .into_iter()
            .enumerate()
            .map(|(idx, pk)| {
                let placeholder_idx = PlaceholderIndex { ui, idx };
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
        Subst::apply(interner, &parameters, &value)
    }
}

pub(crate) trait IntoBindersAndValue<'a, I: Interner> {
    type Binders: IntoIterator<Item = VariableKind<I>>;
    type Value;

    fn into_binders_and_value(self, interner: &'a I) -> (Self::Binders, Self::Value);
}

impl<'a, I, T> IntoBindersAndValue<'a, I> for &'a Binders<T>
where
    I: Interner,
    T: HasInterner<Interner = I>,
    I: 'a,
{
    type Binders = std::iter::Cloned<std::slice::Iter<'a, VariableKind<I>>>;
    type Value = &'a T;

    fn into_binders_and_value(self, interner: &'a I) -> (Self::Binders, Self::Value) {
        (self.binders.iter(interner).cloned(), self.skip_binders())
    }
}

impl<'a, I> IntoBindersAndValue<'a, I> for &'a Fn<I>
where
    I: Interner,
{
    type Binders = std::iter::Map<std::ops::Range<usize>, fn(usize) -> chalk_ir::VariableKind<I>>;
    type Value = &'a Substitution<I>;

    fn into_binders_and_value(self, _interner: &'a I) -> (Self::Binders, Self::Value) {
        let p: fn(usize) -> VariableKind<I> = make_lifetime;
        ((0..self.num_binders).map(p), &self.substitution)
    }
}

fn make_lifetime<I: Interner>(_: usize) -> VariableKind<I> {
    VariableKind::Lifetime
}

impl<'a, T, I: Interner> IntoBindersAndValue<'a, I> for (&'a Vec<VariableKind<I>>, &'a T) {
    type Binders = std::iter::Cloned<std::slice::Iter<'a, VariableKind<I>>>;
    type Value = &'a T;

    fn into_binders_and_value(self, _interner: &'a I) -> (Self::Binders, Self::Value) {
        (self.0.iter().cloned(), &self.1)
    }
}
