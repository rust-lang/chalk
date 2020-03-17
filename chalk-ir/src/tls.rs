use crate::interner::ChalkIr;
use crate::{
    AliasTy, AssocTypeId, Goal, Goals, Lifetime, Parameter, ProgramClauseImplication, StructId,
    TraitId, Ty,
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

    fn debug_alias(
        &self,
        alias: &AliasTy<ChalkIr>,
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
