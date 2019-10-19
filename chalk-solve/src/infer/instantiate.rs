use chalk_ir::family::ChalkIr;
use chalk_ir::fold::*;
use std::fmt::Debug;

use super::*;

impl InferenceTable {
    /// Given the binders from a canonicalized value C, returns a
    /// substitution S mapping each free variable in C to a fresh
    /// inference variable. This substitution can then be applied to
    /// C, which would be equivalent to
    /// `self.instantiate_canonical(v)`.
    pub(crate) fn fresh_subst(
        &mut self,
        binders: &[ParameterKind<UniverseIndex>],
    ) -> Substitution<ChalkIr> {
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
        T: Fold<ChalkIr> + Debug,
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
        T: Fold<ChalkIr>,
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
    #[allow(non_camel_case_types)]
    pub(crate) fn instantiate_binders_existentially<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold<ChalkIr>,
    {
        let (binders, value) = arg.split();
        let max_universe = self.max_universe;
        self.instantiate_in(max_universe, binders.iter().cloned(), value)
    }

    #[allow(non_camel_case_types)]
    pub(crate) fn instantiate_binders_universally<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold<ChalkIr>,
    {
        let (binders, value) = arg.split();
        let ui = self.new_universe();
        let parameters: Vec<_> = binders
            .iter()
            .enumerate()
            .map(|(idx, pk)| {
                let placeholder_idx = PlaceholderIndex { ui, idx };
                match *pk {
                    ParameterKind::Lifetime(()) => {
                        let lt = placeholder_idx.to_lifetime();
                        lt.cast()
                    }
                    ParameterKind::Ty(()) => placeholder_idx.to_ty::<ChalkIr>().cast(),
                }
            })
            .collect();
        Subst::apply(&parameters, value)
    }
}

pub(crate) trait BindersAndValue {
    type Output;

    fn split(&self) -> (&[ParameterKind<()>], &Self::Output);
}

impl<T> BindersAndValue for Binders<T> {
    type Output = T;

    fn split(&self) -> (&[ParameterKind<()>], &Self::Output) {
        (&self.binders, &self.value)
    }
}

impl<'a, T> BindersAndValue for (&'a Vec<ParameterKind<()>>, &'a T) {
    type Output = T;

    fn split(&self) -> (&[ParameterKind<()>], &Self::Output) {
        (&self.0, &self.1)
    }
}
