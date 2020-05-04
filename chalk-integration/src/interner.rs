use crate::tls;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::{
    AliasTy, ApplicationTy, AssocTypeId, CanonicalVarKinds, Goals, Lifetime, OpaqueTy, OpaqueTyId,
    ParameterKinds, ProgramClauseImplication, ProgramClauses, ProjectionTy, QuantifiedWhereClauses,
    SeparatorTraitRef, Substitution, TraitId, Ty,
};
use chalk_ir::{
    Goal, GoalData, LifetimeData, Parameter, ParameterData, ParameterKind, ProgramClause,
    ProgramClauseData, QuantifiedWhereClause, StructId, TyData, UniverseIndex,
};
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use string_cache::DefaultAtom;

pub type Identifier = DefaultAtom;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawId {
    pub index: u32,
}

impl Debug for RawId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "#{}", self.index)
    }
}

/// The default "interner" and the only interner used by chalk
/// itself. In this interner, no interning actually occurs.
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ChalkIr;

impl Interner for ChalkIr {
    type InternedType = Arc<TyData<ChalkIr>>;
    type InternedLifetime = LifetimeData<ChalkIr>;
    type InternedParameter = ParameterData<ChalkIr>;
    type InternedGoal = Arc<GoalData<ChalkIr>>;
    type InternedGoals = Vec<Goal<ChalkIr>>;
    type InternedSubstitution = Vec<Parameter<ChalkIr>>;
    type InternedProgramClause = ProgramClauseData<ChalkIr>;
    type InternedProgramClauses = Vec<ProgramClause<ChalkIr>>;
    type InternedQuantifiedWhereClauses = Vec<QuantifiedWhereClause<ChalkIr>>;
    type InternedParameterKinds = Vec<ParameterKind<()>>;
    type InternedCanonicalVarKinds = Vec<ParameterKind<UniverseIndex>>;
    type DefId = RawId;
    type Identifier = Identifier;

    fn debug_struct_id(
        type_kind_id: StructId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_struct_id(type_kind_id, fmt)))
    }

    fn debug_trait_id(
        type_kind_id: TraitId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_trait_id(type_kind_id, fmt)))
    }

    fn debug_assoc_type_id(
        id: AssocTypeId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_assoc_type_id(id, fmt)))
    }

    fn debug_opaque_ty_id(
        id: OpaqueTyId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_opaque_ty_id(id, fmt)))
    }

    fn debug_alias(alias: &AliasTy<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_alias(alias, fmt)))
    }

    fn debug_projection_ty(
        proj: &ProjectionTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_projection_ty(proj, fmt)))
    }

    fn debug_opaque_ty(
        opaque_ty: &OpaqueTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_opaque_ty(opaque_ty, fmt)))
    }

    fn debug_ty(ty: &Ty<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_ty(ty, fmt)))
    }

    fn debug_lifetime(
        lifetime: &Lifetime<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_lifetime(lifetime, fmt)))
            .or_else(|| Some(write!(fmt, "{:?}", lifetime.interned())))
    }

    fn debug_parameter(
        parameter: &Parameter<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_parameter(parameter, fmt)))
    }

    fn debug_parameter_kinds(
        parameter_kinds: &ParameterKinds<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_parameter_kinds(parameter_kinds, fmt)))
    }

    fn debug_parameter_kinds_with_angles(
        parameter_kinds: &ParameterKinds<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| {
            Some(prog?.debug_parameter_kinds_with_angles(parameter_kinds, fmt))
        })
    }

    fn debug_canonical_var_kinds(
        canonical_var_kinds: &CanonicalVarKinds<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| {
            Some(prog?.debug_canonical_var_kinds(canonical_var_kinds, fmt))
        })
    }

    fn debug_goal(goal: &Goal<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_goal(goal, fmt)))
    }

    fn debug_goals(goals: &Goals<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_goals(goals, fmt)))
    }

    fn debug_program_clause_implication(
        pci: &ProgramClauseImplication<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_program_clause_implication(pci, fmt)))
    }

    fn debug_program_clause(
        clause: &ProgramClause<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_program_clause(clause, fmt)))
    }

    fn debug_program_clauses(
        clause: &ProgramClauses<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_program_clauses(clause, fmt)))
    }

    fn debug_application_ty(
        application_ty: &ApplicationTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_application_ty(application_ty, fmt)))
    }

    fn debug_substitution(
        substitution: &Substitution<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_substitution(substitution, fmt)))
    }

    fn debug_separator_trait_ref(
        separator_trait_ref: &SeparatorTraitRef<'_, ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| {
            Some(prog?.debug_separator_trait_ref(separator_trait_ref, fmt))
        })
    }

    fn debug_quantified_where_clauses(
        clauses: &QuantifiedWhereClauses<Self>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Option<fmt::Result> {
        tls::with_current_program(|prog| Some(prog?.debug_quantified_where_clauses(clauses, fmt)))
    }

    fn intern_ty(&self, ty: TyData<ChalkIr>) -> Arc<TyData<ChalkIr>> {
        Arc::new(ty)
    }

    fn ty_data<'a>(&self, ty: &'a Arc<TyData<ChalkIr>>) -> &'a TyData<Self> {
        ty
    }

    fn intern_lifetime(&self, lifetime: LifetimeData<ChalkIr>) -> LifetimeData<ChalkIr> {
        lifetime
    }

    fn lifetime_data<'a>(&self, lifetime: &'a LifetimeData<ChalkIr>) -> &'a LifetimeData<ChalkIr> {
        lifetime
    }

    fn intern_parameter(&self, parameter: ParameterData<ChalkIr>) -> ParameterData<ChalkIr> {
        parameter
    }

    fn parameter_data<'a>(
        &self,
        parameter: &'a ParameterData<ChalkIr>,
    ) -> &'a ParameterData<ChalkIr> {
        parameter
    }

    fn intern_goal(&self, goal: GoalData<ChalkIr>) -> Arc<GoalData<ChalkIr>> {
        Arc::new(goal)
    }

    fn goal_data<'a>(&self, goal: &'a Arc<GoalData<ChalkIr>>) -> &'a GoalData<ChalkIr> {
        goal
    }

    fn intern_goals<E>(
        &self,
        data: impl IntoIterator<Item = Result<Goal<ChalkIr>, E>>,
    ) -> Result<Vec<Goal<ChalkIr>>, E> {
        data.into_iter().collect()
    }

    fn goals_data<'a>(&self, goals: &'a Vec<Goal<ChalkIr>>) -> &'a [Goal<ChalkIr>] {
        goals
    }

    fn intern_substitution<E>(
        &self,
        data: impl IntoIterator<Item = Result<Parameter<ChalkIr>, E>>,
    ) -> Result<Vec<Parameter<ChalkIr>>, E> {
        data.into_iter().collect()
    }

    fn substitution_data<'a>(
        &self,
        substitution: &'a Vec<Parameter<ChalkIr>>,
    ) -> &'a [Parameter<ChalkIr>] {
        substitution
    }

    fn intern_program_clause(&self, data: ProgramClauseData<Self>) -> ProgramClauseData<Self> {
        data
    }

    fn program_clause_data<'a>(
        &self,
        clause: &'a ProgramClauseData<Self>,
    ) -> &'a ProgramClauseData<Self> {
        clause
    }

    fn intern_program_clauses<E>(
        &self,
        data: impl IntoIterator<Item = Result<ProgramClause<Self>, E>>,
    ) -> Result<Vec<ProgramClause<Self>>, E> {
        data.into_iter().collect()
    }

    fn program_clauses_data<'a>(
        &self,
        clauses: &'a Vec<ProgramClause<Self>>,
    ) -> &'a [ProgramClause<Self>] {
        clauses
    }

    fn intern_quantified_where_clauses<E>(
        &self,
        data: impl IntoIterator<Item = Result<QuantifiedWhereClause<Self>, E>>,
    ) -> Result<Self::InternedQuantifiedWhereClauses, E> {
        data.into_iter().collect()
    }

    fn quantified_where_clauses_data<'a>(
        &self,
        clauses: &'a Self::InternedQuantifiedWhereClauses,
    ) -> &'a [QuantifiedWhereClause<Self>] {
        clauses
    }
    fn intern_parameter_kinds<E>(
        &self,
        data: impl IntoIterator<Item = Result<ParameterKind<()>, E>>,
    ) -> Result<Self::InternedParameterKinds, E> {
        data.into_iter().collect()
    }

    fn parameter_kinds_data<'a>(
        &self,
        parameter_kinds: &'a Self::InternedParameterKinds,
    ) -> &'a [ParameterKind<()>] {
        parameter_kinds
    }

    fn intern_canonical_var_kinds<E>(
        &self,
        data: impl IntoIterator<Item = Result<ParameterKind<UniverseIndex>, E>>,
    ) -> Result<Self::InternedCanonicalVarKinds, E> {
        data.into_iter().collect()
    }

    fn canonical_var_kinds_data<'a>(
        &self,
        canonical_var_kinds: &'a Self::InternedCanonicalVarKinds,
    ) -> &'a [ParameterKind<UniverseIndex>] {
        canonical_var_kinds
    }
}

impl HasInterner for ChalkIr {
    type Interner = ChalkIr;
}
