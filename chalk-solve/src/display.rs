use std::{
    fmt::{Display, Result},
    sync::Arc,
};

use crate::rust_ir::*;
use chalk_ir::{interner::Interner, *};
use itertools::Itertools;

use crate::{logging_db::RecordedItemId, split::Split, RustIrDatabase};

#[macro_use]
mod utils;

mod bounds;
mod identifiers;
mod items;
mod render_trait;
mod state;
mod ty;

pub use self::render_trait::*;
pub use self::state::*;

use self::utils::as_display;

pub fn write_item<F, I, T>(f: &mut F, ws: &WriterState<'_, I>, v: &T) -> Result
where
    F: std::fmt::Write + ?Sized,
    I: Interner,
    T: RenderAsRust<I>,
{
    write!(f, "{}\n", v.display(ws))
}

/// Writes out each item recorded by a [`LoggingRustIrDatabase`].
///
/// [`LoggingRustIrDatabase`]: crate::logging_db::LoggingRustIrDatabase
pub fn write_items<F, I, DB, T>(f: &mut F, db: &DB, ids: T) -> Result
where
    F: std::fmt::Write + ?Sized,
    I: Interner,
    DB: RustIrDatabase<I>,
    T: IntoIterator<Item = RecordedItemId<I>>,
{
    let ws = &WriterState::new(db);
    for id in ids {
        match id {
            RecordedItemId::Impl(id) => {
                let v = db.impl_datum(id);
                write_item(f, ws, &*v)?;
            }
            RecordedItemId::Adt(id) => {
                let v = db.adt_datum(id);
                write_item(f, ws, &*v)?;
            }
            RecordedItemId::Trait(id) => {
                let v = db.trait_datum(id);
                write_item(f, ws, &*v)?;
            }
            RecordedItemId::OpaqueTy(id) => {
                let v = db.opaque_ty_data(id);
                write_item(f, ws, &*v)?;
            }
            RecordedItemId::FnDef(id) => {
                let v = db.fn_def_datum(id);
                write_item(f, ws, &*v)?;
            }
        }
    }
    Ok(())
}

/// Displays a set of bounds, all targeting `Self`, as just the trait names,
/// separated by `+`.
///
/// For example, a list of quantified where clauses which would normally be
/// displayed as:
///
/// ```notrust
/// Self: A, Self: B, Self: C
/// ```
///
/// Is instead displayed by this function as:
///
/// ```notrust
/// A + B + C
/// ```
///
/// Shared between the `Trait` in `dyn Trait` and [`OpaqueTyDatum`] bounds.
fn display_self_where_clauses_as_bounds<'a, I: Interner>(
    s: &'a WriterState<'a, I>,
    bounds: &'a [QuantifiedWhereClause<I>],
) -> impl Display + 'a {
    as_display(move |f| {
        let interner = s.db.interner();
        write!(
            f,
            "{}",
            bounds
                .iter()
                .map(|bound| {
                    as_display(|f| {
                        // each individual trait can have a forall
                        let s = &s.add_debrujin_index(None);
                        if !bound.binders.is_empty(interner) {
                            write!(
                                f,
                                "forall<{}> ",
                                s.binder_var_display(&bound.binders)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )?;
                        }
                        match &bound.skip_binders() {
                            WhereClause::Implemented(trait_ref) => display_trait_with_generics(
                                s,
                                trait_ref.trait_id,
                                &trait_ref.substitution.parameters(interner)[1..],
                            )
                            .fmt(f),
                            WhereClause::AliasEq(alias_eq) => match &alias_eq.alias {
                                AliasTy::Projection(projection_ty) => {
                                    let (assoc_ty_datum, trait_params, assoc_type_params) =
                                        s.db.split_projection(&projection_ty);
                                    display_trait_with_assoc_ty_value(
                                        s,
                                        assoc_ty_datum,
                                        &trait_params[1..],
                                        assoc_type_params,
                                        &alias_eq.ty,
                                    )
                                    .fmt(f)
                                }
                                AliasTy::Opaque(_opaque) => todo!("opaque type AliasTy"),
                            },
                            WhereClause::LifetimeOutlives(lifetime) => lifetime.display(s).fmt(f),
                        }
                    })
                    .to_string()
                })
                .format(" + ")
        )
    })
}

/// Displays a trait with its parameters - something like `AsRef<T>`.
///
/// This is shared between where bounds & dyn Trait.
fn display_trait_with_generics<'a, I: Interner>(
    s: &'a WriterState<'a, I>,
    trait_name: impl RenderAsRust<I> + 'a,
    trait_params: impl IntoIterator<Item = &'a GenericArg<I>> + 'a,
) -> impl Display + 'a {
    use std::fmt::Write;
    let trait_params = trait_params.into_iter().map(|param| param.display(s));
    let mut trait_params_str = String::new();
    write_joined_non_empty_list!(trait_params_str, "<{}>", trait_params, ", ").unwrap();
    as_display(move |f| write!(f, "{}{}", trait_name.display(s), trait_params_str))
}

/// Displays a trait with its parameters and a single associated type -
/// something like `IntoIterator<Item=T>`.
///
/// This is shared between where bounds & dyn Trait.
fn display_trait_with_assoc_ty_value<'a, I: Interner>(
    s: &'a WriterState<'a, I>,
    assoc_ty_datum: Arc<AssociatedTyDatum<I>>,
    trait_params: &'a [GenericArg<I>],
    assoc_ty_params: &'a [GenericArg<I>],
    assoc_ty_value: &'a Ty<I>,
) -> impl Display + 'a {
    as_display(move |f| {
        write!(f, "{}<", assoc_ty_datum.trait_id.display(s))?;
        write_joined_non_empty_list!(
            f,
            "{}, ",
            trait_params.iter().map(|param| param.display(s)),
            ", "
        )?;
        write!(f, "{}", assoc_ty_datum.id.display(s))?;
        write_joined_non_empty_list!(
            f,
            "<{}>",
            assoc_ty_params.iter().map(|param| param.display(s)),
            ", "
        )?;
        write!(f, "={}>", assoc_ty_value.display(s))?;
        Ok(())
    })
}
