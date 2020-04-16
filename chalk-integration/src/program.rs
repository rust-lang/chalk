use crate::{Identifier, TypeKind};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::debug::Angle;
use chalk_ir::interner::ChalkIr;
use chalk_ir::tls;
use chalk_ir::{
    debug::SeparatorTraitRef, AliasTy, ApplicationTy, AssocTypeId, Goal, Goals, ImplId, Lifetime,
    OpaqueTy, OpaqueTyId, Parameter, ProgramClause, ProgramClauseImplication, ProgramClauses,
    ProjectionTy, StructId, Substitution, TraitId, Ty, TypeName,
};
use chalk_rust_ir::{
    AssociatedTyDatum, AssociatedTyValue, AssociatedTyValueId, ImplDatum, ImplType, OpaqueTyDatum,
    StructDatum, TraitDatum, WellKnownTrait,
};
use chalk_solve::split::Split;
use chalk_solve::RustIrDatabase;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// From struct name to item-id. Used during lowering only.
    pub struct_ids: BTreeMap<Identifier, StructId<ChalkIr>>,

    /// For each struct:
    pub struct_kinds: BTreeMap<StructId<ChalkIr>, TypeKind>,

    /// From trait name to item-id. Used during lowering only.
    pub trait_ids: BTreeMap<Identifier, TraitId<ChalkIr>>,

    /// For each trait:
    pub trait_kinds: BTreeMap<TraitId<ChalkIr>, TypeKind>,

    /// For each struct:
    pub struct_data: BTreeMap<StructId<ChalkIr>, Arc<StructDatum<ChalkIr>>>,

    /// For each impl:
    pub impl_data: BTreeMap<ImplId<ChalkIr>, Arc<ImplDatum<ChalkIr>>>,

    /// For each associated ty value `type Foo = XXX` found in an impl:
    pub associated_ty_values:
        BTreeMap<AssociatedTyValueId<ChalkIr>, Arc<AssociatedTyValue<ChalkIr>>>,

    // From opaque type name to item-id. Used during lowering only.
    pub opaque_ty_ids: BTreeMap<Identifier, OpaqueTyId<ChalkIr>>,

    /// For each opaque type:
    pub opaque_ty_data: BTreeMap<OpaqueTyId<ChalkIr>, Arc<OpaqueTyDatum<ChalkIr>>>,

    /// For each trait:
    pub trait_data: BTreeMap<TraitId<ChalkIr>, Arc<TraitDatum<ChalkIr>>>,

    /// For each trait lang item
    pub well_known_traits: BTreeMap<WellKnownTrait, TraitId<ChalkIr>>,

    /// For each associated ty declaration `type Foo` found in a trait:
    pub associated_ty_data: BTreeMap<AssocTypeId<ChalkIr>, Arc<AssociatedTyDatum<ChalkIr>>>,

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
    fn debug_struct_id(
        &self,
        struct_id: StructId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        if let Some(k) = self.struct_kinds.get(&struct_id) {
            write!(fmt, "{}", k.name)
        } else {
            fmt.debug_struct("InvalidStructId")
                .field("index", &struct_id.0)
                .finish()
        }
    }

    fn debug_trait_id(
        &self,
        trait_id: TraitId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        if let Some(k) = self.trait_kinds.get(&trait_id) {
            write!(fmt, "{}", k.name)
        } else {
            fmt.debug_struct("InvalidTraitId")
                .field("index", &trait_id.0)
                .finish()
        }
    }

    fn debug_assoc_type_id(
        &self,
        assoc_type_id: AssocTypeId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        if let Some(d) = self.associated_ty_data.get(&assoc_type_id) {
            write!(fmt, "({:?}::{})", d.trait_id, d.name)
        } else {
            fmt.debug_struct("InvalidItemId")
                .field("index", &assoc_type_id.0)
                .finish()
        }
    }

    fn debug_opaque_ty_id(
        &self,
        opaque_ty_id: OpaqueTyId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        if let Some(d) = self.opaque_ty_data.get(&opaque_ty_id) {
            write!(fmt, "{:?}", d.bound.skip_binders().hidden_ty)
        } else {
            fmt.debug_struct("InvalidItemId")
                .field("index", &opaque_ty_id.0)
                .finish()
        }
    }

    fn debug_alias(
        &self,
        alias_ty: &AliasTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        match alias_ty {
            AliasTy::Projection(projection_ty) => self.debug_projection_ty(projection_ty, fmt),
            AliasTy::Opaque(opaque_ty) => self.debug_opaque_ty(opaque_ty, fmt),
        }
    }

    fn debug_projection_ty(
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

    fn debug_opaque_ty(
        &self,
        opaque_ty: &OpaqueTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        write!(fmt, "impl {:?}", opaque_ty.opaque_ty_id)
    }

    fn debug_ty(&self, ty: &Ty<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", ty.data(interner))
    }

    fn debug_lifetime(
        &self,
        lifetime: &Lifetime<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", lifetime.data(interner))
    }

    fn debug_parameter(
        &self,
        parameter: &Parameter<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", parameter.data(interner).inner_debug())
    }

    fn debug_goal(
        &self,
        goal: &Goal<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", goal.data(interner))
    }

    fn debug_goals(
        &self,
        goals: &Goals<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", goals.debug(interner))
    }

    fn debug_program_clause_implication(
        &self,
        pci: &ProgramClauseImplication<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", pci.debug(interner))
    }

    fn debug_program_clause(
        &self,
        clause: &ProgramClause<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", clause.data(interner))
    }

    fn debug_program_clauses(
        &self,
        clauses: &ProgramClauses<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", clauses.as_slice(interner))
    }

    fn debug_application_ty(
        &self,
        application_ty: &ApplicationTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", application_ty.debug(interner))
    }

    fn debug_substitution(
        &self,
        substitution: &Substitution<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", substitution.debug(interner))
    }

    fn debug_separator_trait_ref(
        &self,
        separator_trait_ref: &SeparatorTraitRef<'_, ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", separator_trait_ref.debug(interner))
    }

    fn debug_quantified_where_clauses(
        &self,
        clauses: &chalk_ir::QuantifiedWhereClauses<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        let interner = self.interner();
        write!(fmt, "{:?}", clauses.as_slice(interner))
    }
}

impl RustIrDatabase<ChalkIr> for Program {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        self.custom_clauses.clone()
    }

    fn associated_ty_data(&self, ty: AssocTypeId<ChalkIr>) -> Arc<AssociatedTyDatum<ChalkIr>> {
        self.associated_ty_data[&ty].clone()
    }

    fn trait_datum(&self, id: TraitId<ChalkIr>) -> Arc<TraitDatum<ChalkIr>> {
        self.trait_data[&id].clone()
    }

    fn impl_datum(&self, id: ImplId<ChalkIr>) -> Arc<ImplDatum<ChalkIr>> {
        self.impl_data[&id].clone()
    }

    fn associated_ty_value(
        &self,
        id: AssociatedTyValueId<ChalkIr>,
    ) -> Arc<AssociatedTyValue<ChalkIr>> {
        self.associated_ty_values[&id].clone()
    }

    fn opaque_ty_data(&self, id: OpaqueTyId<ChalkIr>) -> Arc<OpaqueTyDatum<ChalkIr>> {
        self.opaque_ty_data[&id].clone()
    }

    fn struct_datum(&self, id: StructId<ChalkIr>) -> Arc<StructDatum<ChalkIr>> {
        self.struct_data[&id].clone()
    }

    fn as_struct_id(&self, type_name: &TypeName<ChalkIr>) -> Option<StructId<ChalkIr>> {
        match type_name {
            TypeName::Struct(struct_id) => Some(*struct_id),
            _ => None,
        }
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<ChalkIr>,
        parameters: &[Parameter<ChalkIr>],
    ) -> Vec<ImplId<ChalkIr>> {
        let interner = self.interner();
        self.impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                let trait_ref = &impl_datum.binders.skip_binders().trait_ref;
                trait_id == trait_ref.trait_id && {
                    assert_eq!(trait_ref.substitution.len(interner), parameters.len());
                    <[_] as CouldMatch<[_]>>::could_match(
                        &parameters,
                        interner,
                        &trait_ref.substitution.parameters(interner),
                    )
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
        let interner = self.interner();
        // Look for an impl like `impl Send for Foo` where `Foo` is
        // the struct.  See `push_auto_trait_impls` for more.
        self.impl_data.values().any(|impl_datum| {
            impl_datum.trait_id() == auto_trait_id
                && impl_datum.self_type_struct_id(interner) == Some(struct_id)
        })
    }

    fn well_known_trait_id(&self, well_known_trait: WellKnownTrait) -> Option<TraitId<ChalkIr>> {
        self.well_known_traits.get(&well_known_trait).map(|x| *x)
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}
