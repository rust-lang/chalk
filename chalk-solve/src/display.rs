use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter, Result},
    rc::Rc,
    sync::Arc,
};

use chalk_ir::{
    interner::Interner, AliasEq, AliasTy, ApplicationTy, BoundVar, Fn as ChalkFn, Lifetime,
    LifetimeData, Parameter, ParameterData, ParameterKind, QuantifiedWhereClause, StructId,
    TraitId, TraitRef, Ty, TyData, TypeName, WhereClause,
};

use chalk_rust_ir::{
    AliasEqBound, AssociatedTyDatum, AssociatedTyValue, ImplDatum, InlineBound,
    QuantifiedInlineBound, StructDatum, TraitBound, TraitDatum,
};

use crate::{split::Split, RustIrDatabase};

/// Displays `RenderAsRust` data.
///
/// This is a utility struct for making `RenderAsRust` nice to use with rust format macros.
pub struct DisplayRenderAsRust<'a, I: Interner, T> {
    s: &'a WriterState<'a, I>,
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
    fn fmt(&self, s: &WriterState<'_, I>, f: &mut Formatter<'_>) -> Result;
    fn display<'a>(&'a self, s: &'a WriterState<'a, I>) -> DisplayRenderAsRust<'a, I, Self>
    where
        Self: Sized,
    {
        DisplayRenderAsRust { s, rar: self }
    }
}

impl<I: Interner> RenderAsRust<I> for AssociatedTyValue<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // see comments for a similar empty env operation in AssociatedTyDatum's
        // impl of RenderAsRust.
        let assoc_ty_data = s.db.associated_ty_data(self.associated_ty_id);
        let impl_datum = s.db.impl_datum(self.impl_id);

        let impl_param_names_in_impl_env = s.binder_var_indices(&impl_datum.binders.binders);

        let s = &s.add_debrujin_index();

        let param_names_in_assoc_ty_value_env = s
            .binder_var_indices(&self.value.binders)
            .collect::<Vec<_>>();

        let (impl_params_in_assoc_ty_value_env, _assoc_ty_value_params) =
            s.db.split_associated_ty_value_parameters(&param_names_in_assoc_ty_value_env, self);

        let s = &s.add_parameter_mapping(
            impl_params_in_assoc_ty_value_env.iter().cloned(),
            impl_param_names_in_impl_env,
        );

        // let params = s
        //     .binder_var_display(&self.value.binders)
        //     .collect::<Vec<_>>();
        let display_params = s
            .binder_var_display(&self.value.binders)
            .collect::<Vec<_>>();

        let (_impl_display, assoc_ty_value_display) =
            s.db.split_associated_ty_value_parameters(&display_params, self);

        write!(
            f,
            "type {}<{}> = {};",
            s.db.identifier_name(&assoc_ty_data.name),
            assoc_ty_value_display.join(", "),
            self.value.value.ty.display(s)
        )?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for ImplDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        let binders: Vec<_> = s.binder_var_display(&self.binders.binders).collect();

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

        let assoc_ty_values = self
            .associated_ty_value_ids
            .iter()
            .map(|assoc_ty_value| {
                s.db.associated_ty_value(*assoc_ty_value)
                    .display(s)
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        write!(
            f,
            "impl{} {} for {} ",
            binders_name,
            full_trait_name,
            trait_ref.self_type_parameter(interner).display(s)
        )?;

        if !self.binders.value.where_clauses.is_empty() {
            write!(f, "where {} ", self.binders.value.where_clauses.display(s))?;
        }

        write!(f, "{{{}}}", assoc_ty_values)?;

        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for InlineBound<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            // Foo: Vec<T>
            InlineBound::TraitBound(trait_bound) => trait_bound.fmt(s, f),
            // Foo: Iterator<Item=Foo>
            InlineBound::AliasEqBound(eq_bound) => eq_bound.fmt(s, f),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for TraitBound<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        display_trait_with_generics(s, self.trait_id, &self.args_no_self).fmt(f)
    }
}

impl<I: Interner> RenderAsRust<I> for AliasEqBound<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
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

impl<I: Interner> RenderAsRust<I> for Ty<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to TyData
        self.data(s.db.interner()).fmt(s, f)
    }
}
impl<I: Interner> RenderAsRust<I> for Lifetime<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to LifetimeData
        self.data(s.db.interner()).fmt(s, f)
    }
}
impl<I: Interner> RenderAsRust<I> for Parameter<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to ParameterData
        self.data(s.db.interner()).fmt(s, f)
    }
}
impl<I: Interner> RenderAsRust<I> for StructId<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // TODO: use debug methods?
        f.write_str(&s.db.struct_name(*self))
    }
}
impl<I: Interner> RenderAsRust<I> for TraitId<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // TODO: use debug methods?
        f.write_str(&s.db.trait_name(*self))
    }
}

impl<I: Interner> RenderAsRust<I> for TyData<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self {
            TyData::Dyn(dyn_ty) => {
                let s = &s.add_debrujin_index();
                write!(f, "dyn ")?;
                // dyn_ty.bounds.binders creates a Self binding for the trait
                write!(
                    f,
                    "{}",
                    dyn_ty
                        .bounds
                        .value
                        .iter()
                        .map(|bound| {
                            as_display(|f| {
                                // each individual trait within the 'dyn' can have a
                                // forall clause.
                                let s = &s.add_debrujin_index();
                                if !bound.binders.is_empty() {
                                    write!(
                                        f,
                                        "forall<{}> ",
                                        s.binder_var_display(&bound.binders)
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )?;
                                }
                                match &bound.value {
                                    WhereClause::Implemented(trait_ref) => {
                                        display_trait_with_generics(
                                            s,
                                            trait_ref.trait_id,
                                            &trait_ref.substitution.parameters(interner)[1..],
                                        )
                                        .fmt(f)
                                    }
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
                                        .fmt(f)
                                    }
                                }
                            })
                            .to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(" + ")
                )
            }
            TyData::BoundVar(bound_var) => write!(f, "{}", s.display_bound_var(bound_var)),
            TyData::Alias(alias_ty) => alias_ty.fmt(s, f),
            TyData::Apply(apply_ty) => apply_ty.fmt(s, f),
            TyData::Function(func) => func.fmt(s, f),
            TyData::Placeholder(_) => unreachable!("cannot print placeholder variables"),
            _ => unimplemented!(),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for AliasTy<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // <X as Y<A1, A2, A3>>::Z<B1, B2, B3>

        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.

        let (assoc_ty_datum, trait_params, assoc_type_params) = s.db.split_projection(&self);
        write!(
            f,
            "<{} as {}>::{}<{}>",
            trait_params[0].display(s),
            display_trait_with_generics(s, assoc_ty_datum.trait_id, &trait_params[1..]),
            s.db.identifier_name(&assoc_ty_datum.name),
            assoc_type_params
                .iter()
                .map(|param| param.display(s).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ChalkFn<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index();
        if self.num_binders > 0 {
            write!(
                f,
                "for<{}> ",
                (0..self.num_binders)
                    .map(|n| format!("'{}", s.name_for_introduced_bound_var(n)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(
            f,
            "fn({})",
            self.parameters
                .iter()
                .map(|param| param.display(s).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ApplicationTy<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self.name {
            TypeName::Struct(sid) => {
                write!(f, "{}", sid.display(s))?;
                let parameters = self.substitution.parameters(interner);
                if parameters.len() > 0 {
                    write!(
                        f,
                        "<{}>",
                        parameters
                            .iter()
                            .map(|param| param.display(s).to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    )?;
                }
            }
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
                    "<{} as {}>::{}",
                    self.first_type_parameter(interner).unwrap().display(s),
                    datum.trait_id.display(s),
                    s.db.identifier_name(&datum.name),
                )?;
                let params = self.substitution.parameters(interner);
                if params.len() > 1 {
                    write!(
                        f,
                        "<{}>",
                        params[1..]
                            .iter()
                            .map(|ty| ty.display(s).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
            }
            TypeName::Error => write!(f, "{{error}}")?,
        }
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for LifetimeData<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            LifetimeData::BoundVar(v) => write!(f, "'{}", s.display_bound_var(v)),
            LifetimeData::InferenceVar(_) => write!(f, "'_"),
            LifetimeData::Placeholder(_) => unreachable!("cannot print placeholder variables"),
            // Matching the void ensures at compile time that this code is
            // unreachable
            LifetimeData::Phantom(void, _) => match *void {},
        }
    }
}

impl<I: Interner> RenderAsRust<I> for ParameterData<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            ParameterKind::Ty(ty) => write!(f, "{}", ty.display(s)),
            ParameterKind::Lifetime(lt) => write!(f, "{}", lt.display(s)),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedWhereClause<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index();
        if !self.binders.is_empty() {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders)
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        self.value.fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedInlineBound<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index();
        if !self.binders.is_empty() {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders)
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        self.value.fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Vec<QuantifiedWhereClause<I>> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
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
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
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
    s: &'a WriterState<'a, I>,
    trait_id: TraitId<I>,
    trait_params: impl IntoIterator<Item = &'a Parameter<I>> + 'a,
) -> impl Display + 'a {
    let trait_params = trait_params
        .into_iter()
        .map(|param| param.display(s).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    as_display(move |f| write!(f, "{}<{}>", trait_id.display(s), trait_params,))
}

/// This implementation correct inside where clauses.
impl<I: Interner> RenderAsRust<I> for TraitRef<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        write!(
            f,
            "{}: {}",
            self.self_type_parameter(interner).display(s),
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
    s: &'a WriterState<'a, I>,
    assoc_ty_datum: Arc<AssociatedTyDatum<I>>,
    trait_params: &'a [Parameter<I>],
    assoc_ty_params: &'a [Parameter<I>],
    assoc_ty_value: &'a Ty<I>,
) -> impl Display + 'a {
    as_display(move |f| {
        write!(
            f,
            "{}<{}, {}<{}>={}>",
            assoc_ty_datum.trait_id.display(s),
            trait_params
                .iter()
                .map(|param| param.display(s).to_string())
                .collect::<Vec<_>>()
                .join(", "),
            s.db.identifier_name(&assoc_ty_datum.name),
            assoc_ty_params
                .iter()
                .map(|param| param.display(s).to_string())
                .collect::<Vec<_>>()
                .join(", "),
            assoc_ty_value.display(s)
        )
    })
}

/// This implementation correct inside where clauses.
impl<I: Interner> RenderAsRust<I> for AliasEq<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
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
}

impl<I: Interner> RenderAsRust<I> for AssociatedTyDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // In lowering, a completely new empty environment is created for each
        // AssociatedTyDatum, and it's given generic parameters for each generic
        // parameter that its trait had. We want to map the new binders for
        // those generic parameters back into their original names. To do that,
        // first find their original names (trait_binder_names), then the names
        // they have inside the AssociatedTyDatum (assoc_ty_names_for_trait_params),
        // and then add that mapping to the WriterState when writing bounds and
        // where clauses.

        let trait_datum = s.db.trait_datum(self.trait_id);
        // inverted Debrujin indices for the trait's parameters in the trait
        // environment
        let trait_param_names_in_trait_env = s.binder_var_indices(&trait_datum.binders.binders);
        let s = &s.add_debrujin_index();
        // inverted Debrujin indices for the trait's parameters in the
        // associated type environment
        let param_names_in_assoc_ty_env = s
            .binder_var_indices(&self.binders.binders)
            .collect::<Vec<_>>();
        // inverted Debrujin indices to render the trait's parameters in the
        // associated type environment
        let (trait_param_names_in_assoc_ty_env, _) =
            s.db.split_associated_ty_parameters(&param_names_in_assoc_ty_env, self);

        let s = &s.add_parameter_mapping(
            trait_param_names_in_assoc_ty_env.iter().copied(),
            trait_param_names_in_trait_env,
        );

        // rendered names for the associated type's generics in the associated
        // type environment
        let binder_display_in_assoc_ty = s
            .binder_var_display(&self.binders.binders)
            .collect::<Vec<_>>();

        let (_, assoc_ty_params) =
            s.db.split_associated_ty_parameters(&binder_display_in_assoc_ty, self);
        write!(
            f,
            "type {}<{}>",
            s.db.identifier_name(&self.name),
            assoc_ty_params.join(", ")
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
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index();
        if self.binders.len() == 0 {
            write!(f, "trait {} {{}}", self.id.display(s))
        } else {
            let binders: Vec<_> = s
                .binder_var_display(&self.binders.binders)
                .skip(1)
                .collect();
            write!(f, "trait {}<{}> ", self.id.display(s), binders.join(", "))?;
            if !self.binders.value.where_clauses.is_empty() {
                write!(f, "where {} ", self.binders.value.where_clauses.display(s))?;
            }
            let assoc_types = self
                .associated_ty_ids
                .iter()
                .map(|assoc_ty_id| {
                    let assoc_ty_data = s.db.associated_ty_data(*assoc_ty_id);
                    assoc_ty_data.display(s).to_string()
                })
                .collect::<String>();
            write!(f, "{{{}}}", assoc_types)
        }
    }
}

impl<I: Interner> RenderAsRust<I> for StructDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index();
        write!(
            f,
            "struct {}<{}> ",
            self.id.display(s),
            s.binder_var_display(&self.binders.binders)
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
                .enumerate()
                .map(|(idx, field)| { format!("field_{}: {}", idx, field.display(s).to_string()) })
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }
}

/// Like a BoundVar, but with the debrujin index inverted so as to create a
/// canonical name we can use anywhere for each bound variable.
///
/// In BoundVar, the innermost bound variables have debrujin index `0`, and
/// each further out BoundVar has a debrujin index `1` higher.
///
/// In InvertedBoundVar, the outermost variables have inverted_debrujin_idx `0`,
/// and the innermost have their depth, not the other way around.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct InvertedBoundVar {
    /// The inverted debrujin index. Corresponds roughly to an inverted `DebrujinIndex::depth`.
    inverted_debrujin_idx: i64,
    /// The index within the debrujin index. Corresponds to `BoundVar::index`.
    within_idx: usize,
}

impl Display for InvertedBoundVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "_{}_{}", self.inverted_debrujin_idx, self.within_idx)
    }
}

#[derive(Clone, Debug)]
pub struct WriterState<'a, I: Interner> {
    db: &'a dyn RustIrDatabase<I>,
    debrujin_indices_deep: u32,
    // lowered_(inverted_debrujin_idx, index) -> src_correct_(inverted_debrujin_idx, index)
    remapping: Rc<BTreeMap<InvertedBoundVar, InvertedBoundVar>>,
}

impl<'a, I: Interner> WriterState<'a, I> {
    pub fn new(db: &'a dyn RustIrDatabase<I>) -> Self {
        WriterState {
            db,
            debrujin_indices_deep: 0,
            remapping: Rc::new(BTreeMap::new()),
        }
    }

    fn add_debrujin_index(&self) -> Self {
        let mut new_state = self.clone();
        new_state.debrujin_indices_deep += 1;
        new_state
    }

    /// Adds parameter remapping.
    ///
    /// Each of the parameters in `lowered_vars` will be mapped to its
    /// corresponding variable in `original_vars` when printed through the
    /// `WriterState` returned from this method.
    ///
    /// `lowered_vars` and `original_vars` must have the same length.
    fn add_parameter_mapping(
        &self,
        lowered_vars: impl Iterator<Item = InvertedBoundVar>,
        original_vars: impl Iterator<Item = InvertedBoundVar>,
    ) -> Self {
        let remapping = self
            .remapping
            .iter()
            .map(|(a, b)| (*a, *b))
            .chain(lowered_vars.zip(original_vars))
            .collect::<BTreeMap<_, _>>();

        WriterState {
            db: self.db,
            debrujin_indices_deep: self.debrujin_indices_deep,
            remapping: Rc::new(remapping),
        }
    }

    /// Inverts the debrujin index so as to create a canonical name we can
    /// anywhere for each bound variable.
    ///
    /// See [`InvertedBoundVar`][InvertedBoundVar].
    fn invert_debrujin_idx(&self, debrujin_idx: u32, index: usize) -> InvertedBoundVar {
        InvertedBoundVar {
            inverted_debrujin_idx: (self.debrujin_indices_deep as i64) - (debrujin_idx as i64),
            within_idx: index,
        }
    }

    fn apply_mappings(&self, b: InvertedBoundVar) -> impl Display {
        // TODO: sometimes produce "Self"
        self.remapping.get(&b).copied().unwrap_or(b)
    }

    fn indices_for_bound_var(&self, b: &BoundVar) -> InvertedBoundVar {
        self.invert_debrujin_idx(b.debruijn.depth(), b.index)
    }

    fn indices_for_introduced_bound_var(&self, idx: usize) -> InvertedBoundVar {
        // freshly introduced bound vars will always have debrujin index of 0,
        // they're always "innermost".
        self.invert_debrujin_idx(0, idx)
    }

    fn display_bound_var(&self, b: &BoundVar) -> impl Display {
        self.apply_mappings(self.indices_for_bound_var(b))
    }

    fn name_for_introduced_bound_var(&self, idx: usize) -> impl Display {
        self.apply_mappings(self.indices_for_introduced_bound_var(idx))
    }

    fn binder_var_indices<'b>(
        &'b self,
        binders: &'b [ParameterKind<()>],
    ) -> impl Iterator<Item = InvertedBoundVar> + 'b {
        binders
            .iter()
            .enumerate()
            .map(move |(idx, _param)| self.indices_for_introduced_bound_var(idx))
    }

    fn binder_var_display<'b>(
        &'b self,
        binders: &'b [ParameterKind<()>],
    ) -> impl Iterator<Item = String> + 'b {
        binders
            .iter()
            .zip(self.binder_var_indices(binders))
            .map(move |(parameter, var)| match parameter {
                ParameterKind::Ty(_) => format!("{}", self.apply_mappings(var)),
                ParameterKind::Lifetime(_) => format!("'{}", self.apply_mappings(var)),
            })
    }
}
