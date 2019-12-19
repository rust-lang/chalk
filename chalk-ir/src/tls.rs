use crate::family::ChalkIr;
use crate::{AssocTypeId, ProjectionTy, StructId, TraitId};
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
        fmt: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error>;

    fn debug_trait_id(
        &self,
        id: TraitId<ChalkIr>,
        fmt: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error>;

    fn debug_assoc_type_id(
        &self,
        id: AssocTypeId<ChalkIr>,
        fmt: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error>;

    fn debug_projection(
        &self,
        projection: &ProjectionTy<ChalkIr>,
        fmt: &mut fmt::Formatter,
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
