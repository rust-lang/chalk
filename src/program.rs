use crate::program_environment::ProgramEnvironment;
use crate::rules::{self, RustIrSource, ToProgramClauses};
use chalk_ir::debug::Angle;
use chalk_ir::tls;
use chalk_ir::{
    Identifier, ImplId, Parameter, ProgramClause, ProjectionTy, StructId, TraitId, Ty, TypeId,
    TypeKindId, TypeName,
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
    pub(crate) struct_data: BTreeMap<StructId, StructDatum>,

    /// For each impl:
    pub(crate) impl_data: BTreeMap<ImplId, ImplDatum>,

    /// For each trait:
    pub(crate) trait_data: BTreeMap<TraitId, TraitDatum>,

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

impl RustIrSource for Program {
    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum> {
        self.associated_ty_data[&ty].clone()
    }

    fn impl_datum(&self, id: ImplId) -> &ImplDatum {
        &self.impl_data[&id]
    }

    fn struct_datum(&self, id: StructId) -> &StructDatum {
        &self.struct_data[&id]
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool {
        // Look for an impl like `impl Send for Foo` where `Foo` is
        // the struct.  See `push_auto_trait_impls` for more.
        let type_kind_id = TypeKindId::StructId(struct_id);
        self.impl_data.values().any(|impl_datum| {
            let impl_trait_ref = impl_datum.binders.value.trait_ref.trait_ref();
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

    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy,
    ) -> (Arc<AssociatedTyDatum>, &'p [Parameter], &'p [Parameter]) {
        self.split_projection(projection)
    }
}

impl Program {
    pub fn environment(&self) -> ProgramEnvironment {
        // Construct the set of *clauses*; these are sort of a compiled form
        // of the data above that always has the form:
        //
        //       forall P0...Pn. Something :- Conditions
        let mut program_clauses = self.custom_clauses.clone();

        self.associated_ty_data
            .values()
            .for_each(|d| d.to_program_clauses(self, &mut program_clauses));

        self.trait_data
            .values()
            .for_each(|d| d.to_program_clauses(self, &mut program_clauses));

        self.struct_data
            .values()
            .for_each(|d| d.to_program_clauses(self, &mut program_clauses));

        for (&auto_trait_id, auto_trait) in self
            .trait_data
            .iter()
            .filter(|(_, auto_trait)| auto_trait.binders.value.flags.auto)
        {
            for (&struct_id, struct_datum) in self.struct_data.iter() {
                rules::push_auto_trait_impls(
                    auto_trait_id,
                    auto_trait,
                    struct_id,
                    struct_datum,
                    self,
                    &mut program_clauses,
                );
            }
        }

        for datum in self.impl_data.values() {
            // If we encounter a negative impl, do not generate any rule. Negative impls
            // are currently just there to deactivate default impls for auto traits.
            if datum.binders.value.trait_ref.is_positive() {
                datum.to_program_clauses(self, &mut program_clauses);
                datum
                    .binders
                    .value
                    .associated_ty_values
                    .iter()
                    .for_each(|atv| atv.to_program_clauses(self, &mut program_clauses));
            }
        }

        let coinductive_traits = self
            .trait_data
            .iter()
            .filter_map(|(&trait_id, trait_datum)| {
                if trait_datum.binders.value.flags.auto {
                    Some(trait_id)
                } else {
                    None
                }
            })
            .collect();

        ProgramEnvironment::new(coinductive_traits, program_clauses)
    }
}
