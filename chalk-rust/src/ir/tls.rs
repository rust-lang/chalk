use ir;
use std::cell::RefCell;

thread_local! {
    static PROGRAM: RefCell<Option<ir::Program>> = RefCell::new(None)
}

pub fn with_current_program<OP, R>(op: OP) -> R
    where OP: FnOnce(Option<&ir::Program>) -> R
{
    PROGRAM.with(|prog_cell| {
        let p = prog_cell.borrow();
        op(p.as_ref())
    })
}

pub fn set_program_in<OP, R>(p: ir::Program, op: OP) -> R
    where OP: FnOnce(&ir::Program) -> R
{
    PROGRAM.with(|prog_cell| {
        *prog_cell.borrow_mut() = Some(p);
        let r = with_current_program(|p| op(p.unwrap()));
        *prog_cell.borrow_mut() = None;
        r
    })
}
