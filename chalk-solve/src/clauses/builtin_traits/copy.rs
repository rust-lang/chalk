use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::TyData;

pub fn add_copy_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    let _interner = db.interner();

    match ty {
        TyData::Function(_) => builder.push_fact(trait_ref.clone()),
        // TODO(areredify)
        // when #368 lands, extend this to handle everything accordingly
        _ => return,
    };
}
