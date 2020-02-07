use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};

use chalk_ir::{
    interner::Interner, AliasEq, AliasTy, ApplicationTy, Binders, BoundVar, Fn as ChalkFn,
    LifetimeData, Parameter, ParameterData, ParameterKind, QuantifiedWhereClause, TraitId,
    TraitRef, Ty, TyData, TypeName, WhereClause,
};

use chalk_rust_ir::{
    AliasEqBound, AssociatedTyDatum, ImplDatum, InlineBound, QuantifiedInlineBound, StructDatum,
    TraitBound, TraitDatum,
};

use crate::{split::Split, RustIrDatabase};

/// Displays `RenderAsRust` data.
///
/// This is a utility struct for making `RenderAsRust` nice to use with rust format macros.
pub struct DisplayRenderAsRust<'a, I: Interner, T> {
    s: WriterState<'a, I>,
    rar: &'a T,
}

impl<I: Interner, T: RenderAsRust<I>> Display for DisplayRenderAsRust<'_, I, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.rar.fmt(self.s, f)
    }
}

fn as_display<F: Fn(&mut Formatter<'_>) -> Result>(f: F) -> impl Display {
    struct ClosureDisplay<F: Fn(&mut Formatter<'_>) -> Result>(F);

    impl<F: Fn(&mut Formatter<'_>) -> Result> Display for ClosureDisplay<F> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0(f)
        }
    }

    ClosureDisplay(f)
}

pub trait RenderAsRust<I: Interner> {
    fn fmt(&self, s: WriterState<'_, I>, f: &mut Formatter<'_>) -> Result;
    fn display<'a>(&'a self, s: WriterState<'a, I>) -> DisplayRenderAsRust<'a, I, Self>
    where
        Self: Sized,
    {
        DisplayRenderAsRust { s, rar: self }
    }
}

impl<I: Interner> RenderAsRust<I> for ImplDatum<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        dbg!(&self);
        let binders: Vec<_> = s.binder_struct_var_names(&self.binders).collect();

        let trait_ref = &self.binders.value.trait_ref;

        let binders_name = if binders.len() == 0 {
            "".to_string()
        } else {
            format!("<{}>", binders.join(", "))
        };
        // Ignore automatically added Self parameter by skipping first parameter
        let full_trait_name = display_trait_with_generics(
            s,
            trait_ref.trait_id,
            &trait_ref.substitution.parameters(interner)[1..],
        );
        write!(
            f,
            "impl{} {} for {}",
            binders_name,
            full_trait_name,
            trait_ref
                .self_type_parameter(interner)
                .data(interner)
                .display(s)
        )?;

        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for InlineBound<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            // Foo: Vec<T>
            InlineBound::TraitBound(trait_bound) => trait_bound.fmt(s, f),
            // Foo: Iterator<Item=Foo>
            InlineBound::AliasEqBound(eq_bound) => eq_bound.fmt(s, f),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for TraitBound<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        display_trait_with_generics(s, self.trait_id, &self.args_no_self).fmt(f)
    }
}

impl<I: Interner> RenderAsRust<I> for AliasEqBound<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        display_trait_with_assoc_ty_value(
            s,
            s.db.associated_ty_data(self.associated_ty_id),
            &self.trait_bound.args_no_self,
            &self.parameters,
            &self.value,
        )
        .fmt(f)
    }
}

impl<I: Interner> RenderAsRust<I> for TyData<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self {
            TyData::Dyn(dyn_ty) => write!(
                f,
                "dyn {}",
                dyn_ty
                    .bounds
                    .value
                    .iter()
                    .map(|bound| {
                        match &bound.value {
                            WhereClause::Implemented(trait_ref) => display_trait_with_generics(
                                s,
                                trait_ref.trait_id,
                                &trait_ref.substitution.parameters(interner)[1..],
                            )
                            .to_string(),
                            WhereClause::AliasEq(alias_eq) => {
                                let (assoc_ty_datum, trait_params, assoc_type_params) =
                                    s.db.split_projection(&alias_eq.alias);
                                display_trait_with_assoc_ty_value(
                                    s,
                                    assoc_ty_datum,
                                    &trait_params[1..],
                                    assoc_type_params,
                                    &alias_eq.ty,
                                )
                                .to_string()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            ),
            TyData::BoundVar(bound_var) => write!(f, "{}", s.name_for_bound_var(bound_var)),
            TyData::Alias(alias_ty) => alias_ty.fmt(s, f),
            TyData::Apply(apply_ty) => apply_ty.fmt(s, f),
            TyData::Placeholder(_) => write!(f, "{}", "todo-placeholder"),
            TyData::Function(func) => func.fmt(s, f),
            TyData::InferenceVar(_) => write!(f, "{}", "todo-inferenceVar"),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for AliasTy<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        // <X as Y<A1, A2, A3>>::Z<B1, B2, B3>

        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.

        let (assoc_ty_datum, trait_params, assoc_type_params) = s.db.split_projection(&self);
        write!(
            f,
            "<{} as {}>::{}<{}>",
            trait_params[0].data(interner).display(s),
            display_trait_with_generics(s, assoc_ty_datum.trait_id, &trait_params[1..]),
            s.db.identifier_name(&assoc_ty_datum.name),
            assoc_type_params
                .iter()
                .map(|param| param.data(interner).display(s).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ChalkFn<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        let s = s.add_debrujin_index();
        if self.num_binders > 0 {
            write!(
                f,
                "for<{}> ",
                (0..self.num_binders).map(|n| format!("'{}", s.name_for_introduced_bound_var(n))).collect::<Vec<_>>().join(", ")
            )?;
        }
        write!(
            f,
            "fn({})",
            self.parameters
                .iter()
                .map(|param| param.data(interner).display(s).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ApplicationTy<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self.name {
            TypeName::Struct(sid) => write!(
                f,
                "{}<{}>",
                s.db.struct_name(sid),
                self.substitution
                    .parameters(interner)
                    .iter()
                    .map(|param| param.data(interner).display(s).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            TypeName::AssociatedType(assoc_type_id) => {
                // (Iterator::Item)(x)
                // should be written in Rust as <X as Iterator>::Item
                let datum = s.db.associated_ty_data(assoc_type_id);
                assert!(
                    self.len_type_parameters(interner) >= 1,
                    "AssociatedType should have at least 1 parameter"
                );
                write!(
                    f,
                    "<{} as {}>::{}<{}>",
                    self.first_type_parameter(interner)
                        .unwrap()
                        .data(interner)
                        .display(s),
                    s.db.trait_name(datum.trait_id),
                    s.db.identifier_name(&datum.name),
                    self.substitution.parameters(interner)[1..]
                        .iter()
                        .map(|ty| ty.data(interner).display(s).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            }
            TypeName::Error => write!(f, "{{error}}"),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for LifetimeData<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            LifetimeData::BoundVar(v) => write!(f, "'{}", s.name_for_bound_var(v)),
            LifetimeData::InferenceVar(_) => write!(f, "'_"),
            _ => write!(f, "liftimeother-todo3"),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for ParameterData<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self {
            ParameterKind::Ty(ty) => write!(f, "{}", ty.data(interner).display(s)),
            ParameterKind::Lifetime(lt) => write!(f, "{}", lt.data(interner).display(s)),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedWhereClause<I> {
    fn fmt(&self, mut s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // if we have any binders, then we're a "for<'a> X: Y" "quantified"
        // where clause. Otherwise, we're just a regular where clause.
        // TODO: do we actually want to exclude the added index if there is no for<>?
        //if !self.binders.is_empty() {
        //}
        s = s.add_debrujin_index();
        self.value.fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedInlineBound<I> {
    fn fmt(&self, mut s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // TODO: do we want to do something similar to QuantifiedWhereClause here?
        s = s.add_debrujin_index();
        self.value.fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Vec<QuantifiedWhereClause<I>> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|where_clause| { where_clause.display(s).to_string() })
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for WhereClause<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            WhereClause::Implemented(trait_ref) => trait_ref.fmt(s, f),
            WhereClause::AliasEq(alias_eq) => alias_eq.fmt(s, f),
        }
    }
}

/// Displays a trait with its parameters - something like `AsRef<T>`.
///
/// This is shared between where bounds & dyn Trait.
fn display_trait_with_generics<'a, I: Interner>(
    s: WriterState<'a, I>,
    trait_id: TraitId<I>,
    trait_params: impl IntoIterator<Item = &'a Parameter<I>> + 'a,
) -> impl Display + 'a {
    let interner = s.db.interner();
    let trait_params = trait_params
        .into_iter()
        .map(|param| param.data(interner).display(s).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    as_display(move |f| write!(f, "{}<{}>", s.db.trait_name(trait_id), trait_params,))
}

/// This implementation correct inside where clauses.
impl<I: Interner> RenderAsRust<I> for TraitRef<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        write!(
            f,
            "{}: {}",
            self.self_type_parameter(interner).data(interner).display(s),
            display_trait_with_generics(
                s,
                self.trait_id,
                &self.substitution.parameters(interner)[1..]
            )
        )
    }
}

/// Displays a trait with its parameters and a single associated type -
/// something like `IntoIterator<Item=T>`.
///
/// This is shared between where bounds & dyn Trait.
fn display_trait_with_assoc_ty_value<'a, I: Interner>(
    s: WriterState<'a, I>,
    assoc_ty_datum: Arc<AssociatedTyDatum<I>>,
    trait_params: &'a [Parameter<I>],
    assoc_ty_params: &'a [Parameter<I>],
    assoc_ty_value: &'a Ty<I>,
) -> impl Display + 'a {
    let interner = s.db.interner();
    as_display(move |f| {
        write!(
            f,
            "{}<{}, {}<{}>={}>",
            s.db.trait_name(assoc_ty_datum.trait_id),
            trait_params
                .iter()
                .map(|param| param.data(interner).display(s).to_string())
                .collect::<Vec<_>>()
                .join(", "),
            s.db.identifier_name(&assoc_ty_datum.name),
            assoc_ty_params
                .iter()
                .map(|param| param.data(interner).display(s).to_string())
                .collect::<Vec<_>>()
                .join(", "),
            assoc_ty_value.data(interner).display(s)
        )
    })
}

/// This implementation correct inside where clauses.
impl<I: Interner> RenderAsRust<I> for AliasEq<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        // we have: X: Y<A1, A2, A3, Z<B1, B2, B3>=D>
        // B1, B2, B3, X, A1, A2, A3 are put into alias_eq.alias.substitution
        // D is alias_eq.ty
        // Z is alias_eq.alias.associated_ty_id
        // Y is also packed into alias_eq.alias.associated_ty_id
        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.
        let (assoc_ty_datum, trait_params, assoc_type_params) = s.db.split_projection(&self.alias);
        // An alternate form might be `<{} as {}<{}>>::{}<{}> = {}` (with same
        // parameter ordering). This alternate form would be using type equality
        // constraints (https://github.com/rust-lang/rust/issues/20041).
        write!(
            f,
            "{}: {}",
            trait_params[0].data(interner).display(s),
            display_trait_with_assoc_ty_value(
                s,
                assoc_ty_datum,
                &trait_params[1..],
                assoc_type_params,
                &self.ty
            ),
        )
    }
}

impl<I: Interner> RenderAsRust<I> for AssociatedTyDatum<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let binders: Vec<_> = s.binder_var_names(&self.binders).collect();

        write!(
            f,
            "{}<{}>",
            s.db.identifier_name(&self.name),
            binders.join(", ")
        )?;

        let datum_bounds = &self.binders.value;

        if !(datum_bounds.bounds.is_empty() && datum_bounds.where_clauses.is_empty()) {
            write!(f, ": ")?;
        }

        // bounds is `A: V, B: D, C = E`?
        // type Foo<A: V, B:D, C = E>: X + Y + Z;
        let bounds = datum_bounds
            .bounds
            .iter()
            .map(|bound| bound.display(s).to_string())
            .collect::<Vec<String>>()
            .join(" + ");
        write!(f, "{}", bounds)?;

        // where_clause is 'X: Y, Z: D'
        // type Foo<...>: ... where X: Y, Z: D;

        // note: it's a quantified clause b/c we could have `for<'a> T: Foo<'a>`
        // within 'where'
        if !datum_bounds.where_clauses.is_empty() {
            let where_clauses = datum_bounds.where_clauses.display(s);
            write!(f, "\nwhere\n{}", where_clauses)?;
        }
        write!(f, ";")?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for TraitDatum<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let trait_name = s.db.trait_name(self.id);
        if self.binders.len() == 0 {
            write!(f, "trait {} {{}}", trait_name)
        } else {
            let binders: Vec<_> = s.binder_var_names(&self.binders).collect();
            let assoc_types = self
                .associated_ty_ids
                .iter()
                .map(|assoc_ty_id| {
                    let assoc_ty_data = s.db.associated_ty_data(*assoc_ty_id);
                    assoc_ty_data.display(s).to_string()
                })
                .collect::<String>();
            write!(
                f,
                "trait {}<{}> {{{}}}",
                trait_name,
                binders.join(", "),
                assoc_types
            )
        }
    }
}

impl<I: Interner> RenderAsRust<I> for StructDatum<I> {
    fn fmt(&self, s: WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        let s = s.add_debrujin_index();
        write!(
            f,
            "struct {}<{}> ",
            s.db.struct_name(self.id),
            s.binder_struct_var_names(&self.binders)
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if !self.binders.value.where_clauses.is_empty() {
            write!(f, "where {} ", self.binders.value.where_clauses.display(s))?;
        }
        write!(
            f,
            "{{{}}}",
            self.binders
                .value
                .fields
                .iter()
                .map(|field| { field.data(interner).display(s).to_string() })
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WriterState<'a, I: Interner> {
    db: &'a dyn RustIrDatabase<I>,
    debrujin_indices_deep: u32,
}

impl<'a, I: Interner> WriterState<'a, I> {
    pub fn new(db: &'a dyn RustIrDatabase<I>) -> Self {
        WriterState {
            db,
            debrujin_indices_deep: 0,
        }
    }

    fn add_debrujin_index(&self) -> Self {
        let mut new_state = self.clone();
        new_state.debrujin_indices_deep += 1;
        new_state
    }

    fn name_for_bound_var(&self, b: &BoundVar) -> impl Display + '_ {
        // invert debrujin indexes so that we have a canonical name in rust
        // source. After this, the outermost thing is '0', and the innermost is
        // labeled by its depth, not the other way around.
        // let debrujin_index_name = self
        //     .debrujin_indices_deep
        //     .checked_sub(b.debruijn.depth())
        //     .expect("found debrujin index deeper than we thought possible");
        let debrujin_index_name = (self.debrujin_indices_deep as i64) - (b.debruijn.depth() as i64);
        let within_debrujin = b.index;
        as_display(move |f| write!(f, "_{}_{}", debrujin_index_name, within_debrujin))
    }

    fn name_for_introduced_bound_var(&self, idx: usize) -> impl Display + '_ {
        // invert debrujin indexes so that we have a canonical name in rust
        // source. After this, the outermost thing is '0', and the innermost is
        // labeled by its depth, not the other way around.

        // freshly introduced bound vars will always have debrujin index of 0,
        // they're always "innermost".
        let debrujin_index_name = self.debrujin_indices_deep;
        let within_debrujin = idx;
        as_display(move |f| write!(f, "_{}_{}", debrujin_index_name, within_debrujin))
    }

    fn binder_var_names<'b, T: 'b>(
        &'b self,
        binders: &'b Binders<T>,
    ) -> impl Iterator<Item = String> + 'b {
        binders.binders[1..]
            .iter()
            .enumerate()
            .map(move |(idx, parameter)| match parameter {
                ParameterKind::Ty(_) => format!("{}", self.name_for_introduced_bound_var(idx)),
                ParameterKind::Lifetime(_) => {
                    format!("'{}", self.name_for_introduced_bound_var(idx))
                }
            })
    }

    fn binder_struct_var_names<'b, T: 'b>(
        &'b self,
        binders: &'b Binders<T>,
    ) -> impl Iterator<Item = String> + 'b {
        binders
            .binders
            .iter()
            .enumerate()
            .map(move |(idx, parameter)| match parameter {
                ParameterKind::Ty(_) => format!("{}", self.name_for_introduced_bound_var(idx)),
                ParameterKind::Lifetime(_) => {
                    format!("'{}", self.name_for_introduced_bound_var(idx))
                }
            })
    }
}
