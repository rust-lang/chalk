use crate::RustIrDatabase;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use chalk_rust_ir::*;
use std::sync::Arc;

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
        &'p [Parameter<I>],
        &'p [Parameter<I>],
    ) {
        let interner = self.interner();
        let ProjectionTy {
            associated_ty_id,
            ref substitution,
        } = *projection;
        let parameters = substitution.parameters(interner);
        let associated_ty_data = &self.associated_ty_data(associated_ty_id);
        let trait_datum = &self.trait_datum(associated_ty_data.trait_id);
        let trait_num_params = trait_datum.binders.len(interner);
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data.clone(), trait_params, other_params)
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_parameters_from_projection<'p>(
        &self,
        projection: &'p ProjectionTy<I>,
    ) -> &'p [Parameter<I>] {
        let (_, trait_params, _) = self.split_projection(projection);
        trait_params
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_ref_from_projection<'p>(&self, projection: &'p ProjectionTy<I>) -> TraitRef<I> {
        let interner = self.interner();
        let (associated_ty_data, trait_params, _) = self.split_projection(&projection);
        TraitRef {
            trait_id: associated_ty_data.trait_id,
            substitution: Substitution::from(interner, trait_params),
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
        //           ^^^^^^^ impl parameters
        let split_point = parameters.len() - impl_params_len;
        let (other_params, impl_params) = parameters.split_at(split_point);
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
    fn impl_parameters_and_projection_from_associated_ty_value<'p>(
        &self,
        parameters: &'p [Parameter<I>],
        associated_ty_value: &AssociatedTyValue<I>,
    ) -> (&'p [Parameter<I>], ProjectionTy<I>) {
        let interner = self.interner();
        debug_heading!(
            "impl_parameters_and_projection_from_associated_ty_value(parameters={:?})",
            parameters,
        );

        let impl_datum = self.impl_datum(associated_ty_value.impl_id);

        // Get the trait ref from the impl -- so in our example above
        // this would be `Box<!T>: Foo`.
        let (impl_parameters, atv_parameters) =
            self.split_associated_ty_value_parameters(&parameters, associated_ty_value);
        let trait_ref = {
            let opaque_ty_ref = impl_datum.binders.map_ref(|b| &b.trait_ref);
            debug!("opaque_ty_ref: {:?}", opaque_ty_ref);
            opaque_ty_ref.substitute(interner, impl_parameters)
        };

        // Create the parameters for the projection -- in our example
        // above, this would be `['!a, Box<!T>]`, corresponding to
        // `<Box<!T> as Foo>::Item<'!a>`
        let projection_substitution = Substitution::from(
            interner,
            atv_parameters
                .iter()
                .chain(trait_ref.substitution.iter(interner))
                .cloned(),
        );

        let projection = ProjectionTy {
            associated_ty_id: associated_ty_value.associated_ty_id,
            substitution: projection_substitution,
        };

        debug!("impl_parameters: {:?}", impl_parameters);
        debug!("trait_ref: {:?}", trait_ref);
        debug!("projection: {:?}", projection);

        (impl_parameters, projection)
    }
}

impl<DB: RustIrDatabase<I> + ?Sized, I: Interner> Split<I> for DB {}
