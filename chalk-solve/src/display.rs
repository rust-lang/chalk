use std::{
    borrow::Borrow,
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
mod stub;
mod ty;

use self::render_trait::*;
pub use self::state::*;
pub use self::utils::sanitize_debug_name;

use self::utils::as_display;

fn write_item<F, I, T>(f: &mut F, ws: &InternalWriterState<'_, I>, v: &T) -> Result
where
    F: std::fmt::Write + ?Sized,
    I: Interner,
    T: RenderAsRust<I>,
{
    writeln!(f, "{}", v.display(ws))
}

/// Writes stubs for items which were referenced by name, but for which we
/// didn't directly access. For instance, traits mentioned in where bounds which
/// are only usually checked during well-formedness, when we weren't recording
/// well-formedness.
///
/// The "stub" nature of this means it writes output with the right names and
/// the right number of generics, but nothing else. Where clauses, bounds, and
/// fields are skipped. Associated types are ???? skipped.
///
/// `RecordedItemId::Impl` is not supported.
pub fn write_stub_items<F, I, DB, P, T>(f: &mut F, ws: &WriterState<I, DB, P>, ids: T) -> Result
where
    F: std::fmt::Write + ?Sized,
    I: Interner,
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    T: IntoIterator<Item = RecordedItemId<I>>,
{
    let wrapped_db = &ws.wrap_db_ref(|db| stub::StubWrapper::new(db.borrow()));

    write_items(f, wrapped_db, ids)
}

/// Writes out each item recorded by a [`LoggingRustIrDatabase`].
///
/// [`LoggingRustIrDatabase`]: crate::logging_db::LoggingRustIrDatabase
pub fn write_items<F, I, DB, P, T>(f: &mut F, ws: &WriterState<I, DB, P>, ids: T) -> Result
where
    F: std::fmt::Write + ?Sized,
    I: Interner,
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    T: IntoIterator<Item = RecordedItemId<I>>,
{
    for id in ids {
        match id {
            RecordedItemId::Impl(id) => {
                let v = ws.db().impl_datum(id);
                write_item(f, &InternalWriterState::new(ws), &*v)?;
            }
            RecordedItemId::Adt(id) => {
                let v = ws.db().adt_datum(id);
                write_item(f, &InternalWriterState::new(ws), &*v)?;
            }
            RecordedItemId::Trait(id) => {
                let v = ws.db().trait_datum(id);
                write_item(f, &InternalWriterState::new(ws), &*v)?;
            }
            RecordedItemId::OpaqueTy(id) => {
                let v = ws.db().opaque_ty_data(id);
                write_item(f, &InternalWriterState::new(ws), &*v)?;
            }
            RecordedItemId::FnDef(id) => {
                let v = ws.db().fn_def_datum(id);
                write_item(f, &InternalWriterState::new(ws), &*v)?;
            }
            RecordedItemId::Coroutine(id) => {
                let coroutine = ws.db().coroutine_datum(id);
                let witness = ws.db().coroutine_witness_datum(id);
                write_item(f, &InternalWriterState::new(ws), &(&*coroutine, &*witness))?;
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
    s: &'a InternalWriterState<'a, I>,
    bounds: &'a [QuantifiedWhereClause<I>],
) -> impl Display + 'a {
    as_display(move |f| {
        let interner = s.db().interner();
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
                            WhereClause::Implemented(trait_ref) => display_type_with_generics(
                                s,
                                trait_ref.trait_id,
                                &trait_ref.substitution.as_slice(interner)[1..],
                            )
                            .fmt(f),
                            WhereClause::AliasEq(alias_eq) => match &alias_eq.alias {
                                AliasTy::Projection(projection_ty) => {
                                    let (assoc_ty_datum, trait_params, assoc_type_params) =
                                        s.db().split_projection(projection_ty);
                                    display_trait_with_assoc_ty_value(
                                        s,
                                        assoc_ty_datum,
                                        &trait_params[1..],
                                        assoc_type_params,
                                        &alias_eq.ty,
                                    )
                                    .fmt(f)
                                }
                                AliasTy::Opaque(opaque) => opaque.display(s).fmt(f),
                            },
                            WhereClause::LifetimeOutlives(lifetime) => lifetime.display(s).fmt(f),
                            WhereClause::TypeOutlives(ty) => ty.display(s).fmt(f),
                        }
                    })
                    .to_string()
                })
                .format(" + ")
        )
    })
}

/// Displays a type with its parameters - something like `AsRef<T>`,
/// OpaqueTyName<U>, or `AdtName<Value>`.
///
/// This is shared between where bounds, OpaqueTy, & dyn Trait.
fn display_type_with_generics<'a, I: Interner>(
    s: &'a InternalWriterState<'a, I>,
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
    s: &'a InternalWriterState<'a, I>,
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
