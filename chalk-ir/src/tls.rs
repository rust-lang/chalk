use crate::interner::ChalkIr;
use crate::{
    debug::SeparatorTraitRef, AliasTy, ApplicationTy, AssocTypeId, CanonicalVarKinds, Goal, Goals,
    Lifetime, OpaqueTy, OpaqueTyId, Parameter, ParameterKinds, ProgramClause,
    ProgramClauseImplication, ProgramClauses, ProjectionTy, QuantifiedWhereClauses, StructId,
    Substitution, TraitId, Ty,
};
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

thread_local! {
    static PROGRAM: RefCell<Option<Arc<dyn DebugContext>>> = RefCell::new(None)
}

pub trait DebugContext {
    fn debug_struct_id(
        &self,
        id: StructId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_trait_id(
        &self,
        id: TraitId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_assoc_type_id(
        &self,
        id: AssocTypeId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_opaque_ty_id(
        &self,
        id: OpaqueTyId<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_alias(
        &self,
        alias: &AliasTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_opaque_ty(
        &self,
        opaque_ty: &OpaqueTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_projection_ty(
        &self,
        proj: &ProjectionTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_ty(&self, ty: &Ty<ChalkIr>, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>;

    fn debug_lifetime(
        &self,
        lifetime: &Lifetime<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_parameter(
        &self,
        parameter: &Parameter<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_parameter_kinds(
        &self,
        parameter_kinds: &ParameterKinds<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_parameter_kinds_with_angles(
        &self,
        parameter_kinds: &ParameterKinds<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_canonical_var_kinds(
        &self,
        parameter_kinds: &CanonicalVarKinds<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_goal(
        &self,
        goal: &Goal<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_goals(
        &self,
        goals: &Goals<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_program_clause_implication(
        &self,
        pci: &ProgramClauseImplication<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_program_clause(
        &self,
        clause: &ProgramClause<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_program_clauses(
        &self,
        clauses: &ProgramClauses<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_application_ty(
        &self,
        application_ty: &ApplicationTy<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_substitution(
        &self,
        substitution: &Substitution<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_separator_trait_ref(
        &self,
        separator_trait_ref: &SeparatorTraitRef<'_, ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;

    fn debug_quantified_where_clauses(
        &self,
        clauses: &QuantifiedWhereClauses<ChalkIr>,
        fmt: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error>;
}

pub fn with_current_program<R>(op: impl FnOnce(Option<&Arc<dyn DebugContext>>) -> R) -> R {
    PROGRAM.with(|prog_cell| {
        let p = prog_cell.borrow();
        op(p.as_ref())
    })
}

pub fn set_current_program<OP, R>(p: &Arc<impl DebugContext + 'static>, op: OP) -> R
where
    OP: FnOnce() -> R,
{
    let p: Arc<dyn DebugContext> = p.clone();
    PROGRAM.with(|prog_cell| {
        *prog_cell.borrow_mut() = Some(p);
        let r = op();
        *prog_cell.borrow_mut() = None;
        r
    })
}
