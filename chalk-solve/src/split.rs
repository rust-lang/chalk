use crate::rust_ir::*;
use crate::RustIrDatabase;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Methods for splitting up the projections for associated types from
/// the surrounding context.
pub trait Split<I: Interner>: RustIrDatabase<I> {
    /// Given a projection of an associated type, split the type
    /// parameters into those that come from the *trait* and those
    /// that come from the *associated type itself*. So e.g. if you
    /// have `(Iterator::Item)<F>`, this would return `([F], [])`,
    /// since `Iterator::Item` is not generic and hence doesn't have
    /// any type parameters itself.
    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy<I>,
    ) -> (
        Arc<AssociatedTyDatum<I>>,
        &'p [GenericArg<I>],
        &'p [GenericArg<I>],
    ) {
        let interner = self.interner();
        let ProjectionTy {
            associated_ty_id,
            ref substitution,
        } = *projection;
        let parameters = substitution.as_slice(interner);
        let associated_ty_data = &self.associated_ty_data(associated_ty_id);
        let (trait_params, other_params) =
            self.split_associated_ty_parameters(parameters, &**associated_ty_data);
        (associated_ty_data.clone(), trait_params, other_params)
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_parameters_from_projection<'p>(
        &self,
        projection: &'p ProjectionTy<I>,
    ) -> &'p [GenericArg<I>] {
        let (_, trait_params, _) = self.split_projection(projection);
        trait_params
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_ref_from_projection(&self, projection: &ProjectionTy<I>) -> TraitRef<I> {
        let interner = self.interner();
        let (associated_ty_data, trait_params, _) = self.split_projection(projection);
        TraitRef {
            trait_id: associated_ty_data.trait_id,
            substitution: Substitution::from_iter(interner, trait_params),
        }
    }

    /// Given the full set of parameters (or binders) for an
    /// associated type *value* (which appears in an impl), splits
    /// them into the substitutions for the *impl* and those for the
    /// *associated type*.
    ///
    /// # Example
    ///
    /// ```ignore (example)
    /// impl<T> Iterable for Vec<T> {
    ///     type Iter<'a>;
    /// }
    /// ```
    ///
    /// in this example, the full set of parameters would be `['x,
    /// Y]`, where `'x` is the value for `'a` and `Y` is the value for
    /// `T`.
    ///
    /// # Returns
    ///
    /// Returns the pair of:
    ///
    /// * the parameters for the impl (`[Y]`, in our example)
    /// * the parameters for the associated type value (`['a]`, in our example)
    fn split_associated_ty_value_parameters<'p, P>(
        &self,
        parameters: &'p [P],
        associated_ty_value: &AssociatedTyValue<I>,
    ) -> (&'p [P], &'p [P]) {
        let interner = self.interner();
        let impl_datum = self.impl_datum(associated_ty_value.impl_id);
        let impl_params_len = impl_datum.binders.len(interner);
        assert!(parameters.len() >= impl_params_len);

        // the impl parameters are a suffix
        //
        // [ P0..Pn, Pn...Pm ]
        //   ^^^^^^ impl parameters
        let (impl_params, other_params) = parameters.split_at(impl_params_len);
        (impl_params, other_params)
    }

    /// Given the full set of parameters for an associated type *value*
    /// (which appears in an impl), returns the trait reference
    /// and projection that are being satisfied by that value.
    ///
    /// # Example
    ///
    /// ```ignore (example)
    /// impl<T> Iterable for Vec<T> {
    ///     type Iter<'a>;
    /// }
    /// ```
    ///
    /// Here we expect the full set of parameters for `Iter`, which
    /// would be `['x, Y]`, where `'x` is the value for `'a` and `Y`
    /// is the value for `T`.
    ///
    /// Returns the pair of:
    ///
    /// * the parameters that apply to the impl (`Y`, in our example)
    /// * the projection `<Vec<Y> as Iterable>::Iter<'x>`
    #[instrument(level = "debug", skip(self, associated_ty_value))]
    fn impl_parameters_and_projection_from_associated_ty_value<'p>(
        &self,
        parameters: &'p [GenericArg<I>],
        associated_ty_value: &AssociatedTyValue<I>,
    ) -> (&'p [GenericArg<I>], ProjectionTy<I>) {
        let interner = self.interner();

        let impl_datum = self.impl_datum(associated_ty_value.impl_id);

        // Get the trait ref from the impl -- so in our example above
        // this would be `Box<!T>: Foo`.
        let (impl_parameters, atv_parameters) =
            self.split_associated_ty_value_parameters(parameters, associated_ty_value);
        let trait_ref = {
            let opaque_ty_ref = impl_datum.binders.map_ref(|b| &b.trait_ref).cloned();
            debug!(?opaque_ty_ref);
            opaque_ty_ref.substitute(interner, impl_parameters)
        };

        // Create the parameters for the projection -- in our example
        // above, this would be `['!a, Box<!T>]`, corresponding to
        // `<Box<!T> as Foo>::Item<'!a>`
        let projection_substitution = Substitution::from_iter(
            interner,
            trait_ref
                .substitution
                .iter(interner)
                .chain(atv_parameters.iter())
                .cloned(),
        );

        let projection = ProjectionTy {
            associated_ty_id: associated_ty_value.associated_ty_id,
            substitution: projection_substitution,
        };

        debug!(?impl_parameters, ?trait_ref, ?projection);

        (impl_parameters, projection)
    }

    /// Given the full set of parameters (or binders) for an
    /// associated type datum (the one appearing in a trait), splits
    /// them into the parameters for the *trait* and those for the
    /// *associated type*.
    ///
    /// # Example
    ///
    /// ```ignore (example)
    /// trait Foo<T> {
    ///     type Assoc<'a>;
    /// }
    /// ```
    ///
    /// in this example, the full set of parameters would be `['x,
    /// Y]`, where `'x` is the value for `'a` and `Y` is the value for
    /// `T`.
    ///
    /// # Returns
    ///
    /// Returns the tuple of:
    ///
    /// * the parameters for the impl (`[Y]`, in our example)
    /// * the parameters for the associated type value (`['a]`, in our example)
    fn split_associated_ty_parameters<'p, P>(
        &self,
        parameters: &'p [P],
        associated_ty_datum: &AssociatedTyDatum<I>,
    ) -> (&'p [P], &'p [P]) {
        let trait_datum = &self.trait_datum(associated_ty_datum.trait_id);
        let trait_num_params = trait_datum.binders.len(self.interner());
        let split_point = trait_num_params;
        let (trait_params, other_params) = parameters.split_at(split_point);
        (trait_params, other_params)
    }
}

impl<DB: RustIrDatabase<I> + ?Sized, I: Interner> Split<I> for DB {}
