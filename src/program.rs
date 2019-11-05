use chalk_ir::could_match::CouldMatch;
use chalk_ir::debug::Angle;
use chalk_ir::family::ChalkIr;
use chalk_ir::tls;
use chalk_ir::{
    Identifier, ImplId, Parameter, ProgramClause, ProjectionTy, StructId, TraitId, Ty, TypeId,
    TypeKindId, TypeName,
};
use chalk_rust_ir::{
    AssociatedTyDatum, AssociatedTyValue, AssociatedTyValueId, ImplDatum, ImplType, StructDatum,
    TraitDatum, TypeKind,
};
use chalk_solve::split::Split;
use chalk_solve::RustIrDatabase;
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

    /// For each associated ty value `type Foo = XXX` found in an impl:
    pub(crate) associated_ty_values: BTreeMap<AssociatedTyValueId, Arc<AssociatedTyValue>>,

    /// For each trait:
    pub(crate) trait_data: BTreeMap<TraitId, Arc<TraitDatum>>,

    /// For each associated ty declaration `type Foo` found in a trait:
    pub(crate) associated_ty_data: BTreeMap<TypeId, Arc<AssociatedTyDatum>>,

    /// For each user-specified clause
    pub(crate) custom_clauses: Vec<ProgramClause<ChalkIr>>,
}

impl Program {
    /// Returns the ids for all impls declared in this crate.
    pub(crate) fn local_impl_ids(&self) -> Vec<ImplId> {
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| impl_datum.impl_type == ImplType::Local)
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
        projection_ty: &ProjectionTy<ChalkIr>,
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

impl RustIrDatabase for Program {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        self.custom_clauses.clone()
    }

    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum> {
        self.associated_ty_data[&ty].clone()
    }

    fn trait_datum(&self, id: TraitId) -> Arc<TraitDatum> {
        self.trait_data[&id].clone()
    }

    fn impl_datum(&self, id: ImplId) -> Arc<ImplDatum> {
        self.impl_data[&id].clone()
    }

    fn associated_ty_value(&self, id: AssociatedTyValueId) -> Arc<AssociatedTyValue> {
        self.associated_ty_values[&id].clone()
    }

    fn struct_datum(&self, id: StructId) -> Arc<StructDatum> {
        self.struct_data[&id].clone()
    }

    fn impls_for_trait(&self, trait_id: TraitId, parameters: &[Parameter<ChalkIr>]) -> Vec<ImplId> {
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                let trait_ref = &impl_datum.binders.value.trait_ref;
                trait_id == trait_ref.trait_id && {
                    assert_eq!(trait_ref.parameters.len(), parameters.len());
                    <[_] as CouldMatch<[_]>>::could_match(&parameters, &trait_ref.parameters)
                }
            })
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId) -> Vec<ImplId> {
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                impl_datum.trait_id() == trait_id && impl_datum.impl_type == ImplType::Local
            })
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool {
        // Look for an impl like `impl Send for Foo` where `Foo` is
        // the struct.  See `push_auto_trait_impls` for more.
        let type_kind_id = TypeKindId::StructId(struct_id);
        self.impl_data.values().any(|impl_datum| {
            let impl_trait_ref = &impl_datum.binders.value.trait_ref;
            impl_trait_ref.trait_id == auto_trait_id
                && match impl_trait_ref.parameters[0].assert_ty_ref() {
                    Ty::Apply(apply) => match apply.name {
                        TypeName::TypeKindId(id) => id == type_kind_id,
                        _ => false,
                    },

                    _ => false,
                }
        })
    }

    fn type_name(&self, id: TypeKindId) -> Identifier {
        match self.type_kinds.get(&id) {
            Some(v) => v.name,
            None => panic!("no type with id `{:?}`", id),
        }
    }
}
