use chalk_ir::could_match::CouldMatch;
use chalk_ir::debug::Angle;
use chalk_ir::family::ChalkIr;
use chalk_ir::tls;
use chalk_ir::{
    Identifier, ImplId, Parameter, ProgramClause, ProjectionTy, StructId, TraitId, TyData, TypeId,
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
    pub type_ids: BTreeMap<Identifier, TypeKindId<ChalkIr>>,

    /// For each struct/trait:
    pub type_kinds: BTreeMap<TypeKindId<ChalkIr>, TypeKind>,

    /// For each struct:
    pub struct_data: BTreeMap<StructId<ChalkIr>, Arc<StructDatum<ChalkIr>>>,

    /// For each impl:
    pub impl_data: BTreeMap<ImplId<ChalkIr>, Arc<ImplDatum<ChalkIr>>>,

    /// For each associated ty value `type Foo = XXX` found in an impl:
    pub associated_ty_values: BTreeMap<AssociatedTyValueId, Arc<AssociatedTyValue<ChalkIr>>>,

    /// For each trait:
    pub trait_data: BTreeMap<TraitId<ChalkIr>, Arc<TraitDatum<ChalkIr>>>,

    /// For each associated ty declaration `type Foo` found in a trait:
    pub associated_ty_data: BTreeMap<TypeId<ChalkIr>, Arc<AssociatedTyDatum<ChalkIr>>>,

    /// For each user-specified clause
    pub custom_clauses: Vec<ProgramClause<ChalkIr>>,
}

impl Program {
    /// Returns the ids for all impls declared in this crate.
    pub(crate) fn local_impl_ids(&self) -> Vec<ImplId<ChalkIr>> {
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
        type_kind_id: TypeKindId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
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
        fmt: &mut fmt::Formatter<'_>,
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

impl RustIrDatabase<ChalkIr> for Program {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        self.custom_clauses.clone()
    }

    fn associated_ty_data(&self, ty: TypeId<ChalkIr>) -> Arc<AssociatedTyDatum<ChalkIr>> {
        self.associated_ty_data[&ty].clone()
    }

    fn trait_datum(&self, id: TraitId<ChalkIr>) -> Arc<TraitDatum<ChalkIr>> {
        self.trait_data[&id].clone()
    }

    fn impl_datum(&self, id: ImplId<ChalkIr>) -> Arc<ImplDatum<ChalkIr>> {
        self.impl_data[&id].clone()
    }

    fn associated_ty_value(&self, id: AssociatedTyValueId) -> Arc<AssociatedTyValue<ChalkIr>> {
        self.associated_ty_values[&id].clone()
    }

    fn struct_datum(&self, id: StructId<ChalkIr>) -> Arc<StructDatum<ChalkIr>> {
        self.struct_data[&id].clone()
    }

    fn as_struct_id(&self, type_name: &TypeName<ChalkIr>) -> Option<StructId<ChalkIr>> {
        match type_name {
            TypeName::TypeKindId(TypeKindId::StructId(struct_id)) => Some(*struct_id),
            _ => None,
        }
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<ChalkIr>,
        parameters: &[Parameter<ChalkIr>],
    ) -> Vec<ImplId<ChalkIr>> {
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

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<ChalkIr>) -> Vec<ImplId<ChalkIr>> {
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                impl_datum.trait_id() == trait_id && impl_datum.impl_type == ImplType::Local
            })
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }

    fn impl_provided_for(
        &self,
        auto_trait_id: TraitId<ChalkIr>,
        struct_id: StructId<ChalkIr>,
    ) -> bool {
        // Look for an impl like `impl Send for Foo` where `Foo` is
        // the struct.  See `push_auto_trait_impls` for more.
        let type_kind_id = TypeKindId::StructId(struct_id);
        self.impl_data.values().any(|impl_datum| {
            let impl_trait_ref = &impl_datum.binders.value.trait_ref;
            impl_trait_ref.trait_id == auto_trait_id
                && match impl_trait_ref.parameters[0].assert_ty_ref().data() {
                    TyData::Apply(apply) => match apply.name {
                        TypeName::TypeKindId(id) => id == type_kind_id,
                        _ => false,
                    },

                    _ => false,
                }
        })
    }

    fn type_name(&self, id: TypeKindId<ChalkIr>) -> Identifier {
        match self.type_kinds.get(&id) {
            Some(v) => v.name,
            None => panic!("no type with id `{:?}`", id),
        }
    }
}
