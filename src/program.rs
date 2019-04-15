use chalk_ir::debug::Angle;
use chalk_ir::tls;
use chalk_ir::{
    Identifier, ImplId, Parameter, ProgramClause, ProjectionTy, StructId, TraitId, TypeId,
    TypeKindId,
};
use chalk_rust_ir::{AssociatedTyDatum, ImplDatum, ImplType, StructDatum, TraitDatum, TypeKind};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// From type-name to item-id. Used during lowering only.
    pub(crate) type_ids: BTreeMap<Identifier, TypeKindId>,

    /// For each struct/trait:
    pub(crate) type_kinds: BTreeMap<TypeKindId, TypeKind>,

    /// For each struct:
    pub(crate) struct_data: BTreeMap<StructId, Arc<StructDatum>>,

    /// For each impl:
    pub(crate) impl_data: BTreeMap<ImplId, Arc<ImplDatum>>,

    /// For each trait:
    pub(crate) trait_data: BTreeMap<TraitId, Arc<TraitDatum>>,

    /// For each associated ty:
    pub(crate) associated_ty_data: BTreeMap<TypeId, Arc<AssociatedTyDatum>>,

    /// For each user-specified clause
    pub(crate) custom_clauses: Vec<ProgramClause>,
}

impl Program {
    /// Given a projection of an associated type, split the type parameters
    /// into those that come from the *trait* and those that come from the
    /// *associated type itself*. So e.g. if you have `(Iterator::Item)<F>`,
    /// this would return `([F], [])`, since `Iterator::Item` is not generic
    /// and hence doesn't have any type parameters itself.
    ///
    /// Used primarily for debugging output.
    pub(crate) fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy,
    ) -> (Arc<AssociatedTyDatum>, &'p [Parameter], &'p [Parameter]) {
        let ProjectionTy {
            associated_ty_id,
            ref parameters,
        } = *projection;
        let associated_ty_data = &self.associated_ty_data[&associated_ty_id];
        let trait_datum = &self.trait_data[&associated_ty_data.trait_id];
        let trait_num_params = trait_datum.binders.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data.clone(), trait_params, other_params)
    }

    /// Returns the ids for all impls declared in this crate.
    pub(crate) fn local_impl_ids(&self) -> Vec<ImplId> {
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| impl_datum.binders.value.impl_type == ImplType::Local)
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }
}

impl tls::DebugContext for Program {
    fn debug_type_kind_id(
        &self,
        type_kind_id: TypeKindId,
        fmt: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        if let Some(k) = self.type_kinds.get(&type_kind_id) {
            write!(fmt, "{}", k.name)
        } else if let Some(k) = self.type_kinds.get(&type_kind_id) {
            write!(fmt, "{}", k.name)
        } else if let TypeKindId::TypeId(type_id) = type_kind_id {
            if let Some(k) = self.associated_ty_data.get(&type_id) {
                write!(fmt, "({:?}::{})", k.trait_id, k.name)
            } else {
                fmt.debug_struct("InvalidItemId")
                    .field("index", &type_id.0)
                    .finish()
            }
        } else {
            fmt.debug_struct("InvalidItemId")
                .field("index", &type_kind_id.raw_id())
                .finish()
        }
    }

    fn debug_projection(
        &self,
        projection_ty: &ProjectionTy,
        fmt: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        let (associated_ty_data, trait_params, other_params) = self.split_projection(projection_ty);
        write!(
            fmt,
            "<{:?} as {:?}{:?}>::{}{:?}",
            &trait_params[0],
            associated_ty_data.trait_id,
            Angle(&trait_params[1..]),
            associated_ty_data.name,
            Angle(&other_params)
        )
    }
}
