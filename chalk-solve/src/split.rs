use crate::RustIrDatabase;
use chalk_ir::family::ChalkIr;
use chalk_ir::*;
use chalk_rust_ir::*;
use std::sync::Arc;

/// Methods for splitting up the projections for associated types from
/// the surrounding context.
pub trait Split: RustIrDatabase {
    /// Given a projection of an associated type, split the type
    /// parameters into those that come from the *trait* and those
    /// that come from the *associated type itself*. So e.g. if you
    /// have `(Iterator::Item)<F>`, this would return `([F], [])`,
    /// since `Iterator::Item` is not generic and hence doesn't have
    /// any type parameters itself.
    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy<ChalkIr>,
    ) -> (
        Arc<AssociatedTyDatum>,
        &'p [Parameter<ChalkIr>],
        &'p [Parameter<ChalkIr>],
    ) {
        let ProjectionTy {
            associated_ty_id,
            ref parameters,
        } = *projection;
        let associated_ty_data = &self.associated_ty_data(associated_ty_id);
        let trait_datum = &self.trait_datum(associated_ty_data.trait_id);
        let trait_num_params = trait_datum.binders.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data.clone(), trait_params, other_params)
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_parameters_from_projection<'p>(
        &self,
        projection: &'p ProjectionTy<ChalkIr>,
    ) -> &'p [Parameter<ChalkIr>] {
        let (_, trait_params, _) = self.split_projection(projection);
        trait_params
    }

    /// Given a projection `<P0 as Trait<P1..Pn>>::Item<Pn..Pm>`,
    /// returns the trait parameters `[P0..Pn]` (see
    /// `split_projection`).
    fn trait_ref_from_projection<'p>(
        &self,
        projection: &'p ProjectionTy<ChalkIr>,
    ) -> TraitRef<ChalkIr> {
        let (associated_ty_data, trait_params, _) = self.split_projection(&projection);
        TraitRef {
            trait_id: associated_ty_data.trait_id,
            parameters: trait_params.to_owned(),
        }
    }
}

impl<DB: RustIrDatabase + ?Sized> Split for DB {}
