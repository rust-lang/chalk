#![allow(unused)]
use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef};
use chalk_ir::cast::Cast;
use chalk_ir::{ApplicationTy, Ty, TyData, TypeName};
use std::iter;

pub fn argument_types_match<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    trait_ref: &TraitRef<I>,
    fn_ptr: &chalk_ir::Fn<I>,
) -> bool {
    let interner = db.interner();
    let trait_args: Vec<_> = trait_ref.type_parameters(interner).skip(1).collect();
    let fn_args: Vec<_> = fn_ptr
        .substitution
        .parameters(interner)
        .iter()
        .filter_map(|p| p.ty(interner))
        .collect();
    if trait_args.len() != fn_args.len() {
        return false;
    }
    for (arg, &param) in trait_args.iter().zip(fn_args.iter()) {
        if arg != param {
            return false;
        }
    }
    true
}

pub fn add_fn_family_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    ty: &TyData<I>,
) {
    match ty {
        TyData::Function(f) => {
            if argument_types_match(db, trait_ref, f) {
                builder.push_fact(trait_ref.clone());
            }
        }
        // TODO: handle closures once implemented
        _ => return,
    };
}
