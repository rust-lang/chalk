use chalk_ir::fold::*;
use std::fmt::Debug;

use super::*;

impl<TF: TypeFamily> InferenceTable<TF> {
    /// Given the binders from a canonicalized value C, returns a
    /// substitution S mapping each free variable in C to a fresh
    /// inference variable. This substitution can then be applied to
    /// C, which would be equivalent to
    /// `self.instantiate_canonical(v)`.
    pub(crate) fn fresh_subst(
        &mut self,
        binders: &[ParameterKind<UniverseIndex>],
    ) -> Substitution<TF> {
        Substitution {
            parameters: binders
                .iter()
                .map(|kind| {
                    let param_infer_var = kind.map(|ui| self.new_variable(ui));
                    param_infer_var.to_parameter()
                })
                .collect(),
        }
    }

    /// Variant on `instantiate` that takes a `Canonical<T>`.
    pub(crate) fn instantiate_canonical<T>(&mut self, bound: &Canonical<T>) -> T::Result
    where
        T: Fold<TF> + Debug,
    {
        let subst = self.fresh_subst(&bound.binders);
        bound.value.fold_with(&mut &subst, 0).unwrap()
    }

    /// Instantiates `arg` with fresh existential variables in the
    /// given universe; the kinds of the variables are implied by
    /// `binders`. This is used to apply a universally quantified
    /// clause like `forall X, 'Y. P => Q`. Here the `binders`
    /// argument is referring to `X, 'Y`.
    pub(crate) fn instantiate_in<U, T>(
        &mut self,
        universe: UniverseIndex,
        binders: U,
        arg: &T,
    ) -> T::Result
    where
        T: Fold<TF>,
        U: IntoIterator<Item = ParameterKind<()>>,
    {
        let binders: Vec<_> = binders
            .into_iter()
            .map(|pk| pk.map(|()| universe))
            .collect();
        let subst = self.fresh_subst(&binders);
        arg.fold_with(&mut &subst, 0).unwrap()
    }

    /// Variant on `instantiate_in` that takes a `Binders<T>`.
    pub(crate) fn instantiate_binders_existentially<T>(
        &mut self,
        arg: impl IntoBindersAndValue<Value = T>,
    ) -> T::Result
    where
        T: Fold<TF>,
    {
        let (binders, value) = arg.into_binders_and_value();
        let max_universe = self.max_universe;
        self.instantiate_in(max_universe, binders, &value)
    }

    pub(crate) fn instantiate_binders_universally<T>(
        &mut self,
        arg: impl IntoBindersAndValue<Value = T>,
    ) -> T::Result
    where
        T: Fold<TF>,
    {
        let (binders, value) = arg.into_binders_and_value();
        let ui = self.new_universe();
        let parameters: Vec<_> = binders
            .into_iter()
            .enumerate()
            .map(|(idx, pk)| {
                let placeholder_idx = PlaceholderIndex { ui, idx };
                match pk {
                    ParameterKind::Lifetime(()) => {
                        let lt = placeholder_idx.to_lifetime::<TF>();
                        lt.cast()
                    }
                    ParameterKind::Ty(()) => placeholder_idx.to_ty::<TF>().cast(),
                }
            })
            .collect();
        Subst::apply(&parameters, &value)
    }
}

pub(crate) trait IntoBindersAndValue {
    type Binders: IntoIterator<Item = ParameterKind<()>>;
    type Value;

    fn into_binders_and_value(self) -> (Self::Binders, Self::Value);
}

impl<'a, T> IntoBindersAndValue for &'a Binders<T> {
    type Binders = std::iter::Cloned<std::slice::Iter<'a, ParameterKind<()>>>;
    type Value = &'a T;

    fn into_binders_and_value(self) -> (Self::Binders, Self::Value) {
        (self.binders.iter().cloned(), &self.value)
    }
}

impl<'a, TF> IntoBindersAndValue for &'a QuantifiedTy<TF>
where
    TF: TypeFamily,
{
    type Binders = std::iter::Map<std::ops::Range<usize>, fn(usize) -> chalk_ir::ParameterKind<()>>;
    type Value = &'a Ty<TF>;

    fn into_binders_and_value(self) -> (Self::Binders, Self::Value) {
        fn make_lifetime(_: usize) -> ParameterKind<()> {
            ParameterKind::Lifetime(())
        }

        let p: fn(usize) -> ParameterKind<()> = make_lifetime;
        ((0..self.num_binders).map(p), &self.ty)
    }
}

impl<'a, T> IntoBindersAndValue for (&'a Vec<ParameterKind<()>>, &'a T) {
    type Binders = std::iter::Cloned<std::slice::Iter<'a, ParameterKind<()>>>;
    type Value = &'a T;

    fn into_binders_and_value(self) -> (Self::Binders, Self::Value) {
        (self.0.iter().cloned(), &self.1)
    }
}
