//! Writer logic for `where` clauses and other bounds.
//!
//! Contains logic for writing the various forms of `Foo: Bar`.
use std::fmt::{Display, Formatter, Result};

use crate::rust_ir::*;
use chalk_ir::{interner::Interner, *};
use itertools::Itertools;

use super::{
    display_trait_with_assoc_ty_value, display_type_with_generics, render_trait::RenderAsRust,
    state::InternalWriterState,
};
use crate::split::Split;

impl<I: Interner> RenderAsRust<I> for InlineBound<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            // Foo: Vec<T>
            InlineBound::TraitBound(trait_bound) => trait_bound.fmt(s, f),
            // Foo: Iterator<Item=Foo>
            InlineBound::AliasEqBound(eq_bound) => eq_bound.fmt(s, f),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for TraitBound<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        display_type_with_generics(s, self.trait_id, &self.args_no_self).fmt(f)
    }
}

impl<I: Interner> RenderAsRust<I> for AliasEqBound<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        display_trait_with_assoc_ty_value(
            s,
            s.db().associated_ty_data(self.associated_ty_id),
            &self.trait_bound.args_no_self,
            &self.parameters,
            &self.value,
        )
        .fmt(f)
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedWhereClause<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        let s = &s.add_debrujin_index(None);
        if !self.binders.is_empty(interner) {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders).format(", ")
            )?;
        }
        self.skip_binders().fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedInlineBound<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        let s = &s.add_debrujin_index(None);
        if !self.binders.is_empty(interner) {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders).format(", ")
            )?;
        }
        self.skip_binders().fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Vec<QuantifiedWhereClause<I>> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|where_clause| { format!("{}{}", s.indent(), where_clause.display(s)) })
                .format(",\n")
        )?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for WhereClause<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            WhereClause::Implemented(trait_ref) => trait_ref.fmt(s, f),
            WhereClause::AliasEq(alias_eq) => alias_eq.fmt(s, f),
            WhereClause::LifetimeOutlives(lifetime) => lifetime.display(s).fmt(f),
            WhereClause::TypeOutlives(ty) => ty.display(s).fmt(f),
        }
    }
}

/// This renders `TraitRef` as a clause in a where clause, as opposed to its
/// usage in other places.
impl<I: Interner> RenderAsRust<I> for TraitRef<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        write!(
            f,
            "{}: {}",
            self.self_type_parameter(interner).display(s),
            display_type_with_generics(
                s,
                self.trait_id,
                &self.substitution.as_slice(interner)[1..]
            )
        )
    }
}

/// This renders `AliasEq` as a clause in a where clause, as opposed to its
/// usage in other places.
impl<I: Interner> RenderAsRust<I> for AliasEq<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // we have: X: Y<A1, A2, A3, Z<B1, B2, B3>=D>
        // B1, B2, B3, X, A1, A2, A3 are put into alias_eq.alias.substitution
        // D is alias_eq.ty
        // Z is alias_eq.alias.associated_ty_id
        // Y is also packed into alias_eq.alias.associated_ty_id
        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.
        match &self.alias {
            AliasTy::Projection(projection_ty) => {
                let (assoc_ty_datum, trait_params, assoc_type_params) =
                    s.db().split_projection(projection_ty);
                // An alternate form might be `<{} as {}<{}>>::{}<{}> = {}` (with same
                // parameter ordering). This alternate form would require type equality
                // constraints (https://github.com/rust-lang/rust/issues/20041).
                write!(
                    f,
                    "{}: {}",
                    trait_params[0].display(s),
                    display_trait_with_assoc_ty_value(
                        s,
                        assoc_ty_datum,
                        &trait_params[1..],
                        assoc_type_params,
                        &self.ty
                    ),
                )
            }
            AliasTy::Opaque(opaque) => write!(f, "{}", opaque.display(s)),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for LifetimeOutlives<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        // a': 'b
        write!(f, "{}: {}", self.a.display(s), self.b.display(s))
    }
}

impl<I: Interner> RenderAsRust<I> for TypeOutlives<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        // T: 'a
        write!(f, "{}: {}", self.ty.display(s), self.lifetime.display(s))
    }
}
