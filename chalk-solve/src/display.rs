use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter, Result},
    rc::Rc,
    sync::Arc,
};

use chalk_ir::{
    interner::Interner, AliasEq, AliasTy, ApplicationTy, AssocTypeId, BoundVar, Fn as ChalkFn,
    Lifetime, LifetimeData, Mutability, OpaqueTy, OpaqueTyId, Parameter, ParameterData,
    ParameterKind, ParameterKinds, ProjectionTy, QuantifiedWhereClause, Scalar, StructId, TraitId,
    TraitRef, Ty, TyData, TypeName, WhereClause,
};
use chalk_rust_ir::{
    AliasEqBound, AssociatedTyDatum, AssociatedTyValue, ImplDatum, InlineBound, OpaqueTyDatum,
    Polarity, QuantifiedInlineBound, StructDatum, TraitBound, TraitDatum,
};
use itertools::Itertools;

use crate::{logging_db::RecordedItemId, split::Split, RustIrDatabase};

pub fn write_top_level<I, DB, T, F>(f: &mut F, db: &DB, v: &T) -> Result
where
    I: Interner,
    DB: RustIrDatabase<I>,
    T: RenderAsRust<I>,
    F: std::fmt::Write,
{
    let ws = &WriterState::new(db);
    write!(f, "{}\n", v.display(ws))
}

/// Writes out each item recorded by a [`LoggingRustIrDatabase`].
///
/// [`LoggingRustIrDatabase`]: crate::logging_db::LoggingRustIrDatabase
pub fn write_program<I, DB, T>(f: &mut Formatter<'_>, db: &DB, ids: T) -> Result
where
    I: Interner,
    DB: RustIrDatabase<I>,
    T: IntoIterator<Item = RecordedItemId<I>>,
{
    for id in ids {
        match id {
            RecordedItemId::Impl(id) => {
                let v = db.impl_datum(id);
                write_top_level(f, db, &*v)?;
            }
            RecordedItemId::Struct(id) => {
                let v = db.struct_datum(id);
                write_top_level(f, db, &*v)?;
            }
            RecordedItemId::Trait(id) => {
                let v = db.trait_datum(id);
                write_top_level(f, db, &*v)?;
            }
            RecordedItemId::OpaqueTy(id) => {
                let v = db.opaque_ty_data(id);
                write_top_level(f, db, &*v)?;
            }
        }
    }
    Ok(())
}

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

macro_rules! write_joined_non_empty_list {
    ($f:expr,$template:tt,$list:expr,$sep:expr) => {{
        let mut x = $list.into_iter().peekable();
        if x.peek().is_some() {
            write!($f, $template, x.format($sep))
        } else {
            Ok(())
        }
    }};
}

impl<I: Interner> RenderAsRust<I> for AssociatedTyValue<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // see comments for a similar empty env operation in AssociatedTyDatum's
        // impl of RenderAsRust.
        let assoc_ty_data = s.db.associated_ty_data(self.associated_ty_id);
        let impl_datum = s.db.impl_datum(self.impl_id);

        let impl_param_names_in_impl_env = s.binder_var_indices(&impl_datum.binders.binders);

        let s = &s.add_debrujin_index(None);
        let value = self.value.skip_binders();

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

        write!(f, "{}type {}", s.indent(), assoc_ty_data.id.display(s))?;
        write_joined_non_empty_list!(f, "<{}>", &assoc_ty_value_display, ", ")?;
        write!(f, " = {};", value.ty.display(s))?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for Polarity {
    fn fmt(&self, _s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        if !self.is_positive() {
            write!(f, "!")?;
        }
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for ImplDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();

        let s = &s.add_debrujin_index(None);
        let binders = s.binder_var_display(&self.binders.binders);

        let value = self.binders.skip_binders();

        let trait_ref = &value.trait_ref;
        // Ignore automatically added Self parameter by skipping first parameter
        let full_trait_name = display_trait_with_generics(
            s,
            trait_ref.trait_id,
            &trait_ref.substitution.parameters(interner)[1..],
        );
        write!(f, "impl")?;
        write_joined_non_empty_list!(f, "<{}>", binders, ", ")?;
        write!(
            f,
            " {}{} for {}",
            self.polarity.display(s),
            full_trait_name,
            trait_ref.self_type_parameter(interner).display(s)
        )?;
        if !value.where_clauses.is_empty() {
            let s = &s.add_indent();
            write!(f, "\nwhere\n{}\n", value.where_clauses.display(s))?;
        } else {
            write!(f, " ")?;
        }
        write!(f, "{{")?;
        {
            let s = &s.add_indent();
            let assoc_ty_values = self.associated_ty_value_ids.iter().map(|assoc_ty_value| {
                s.db.associated_ty_value(*assoc_ty_value)
                    .display(s)
                    .to_string()
            });
            write_joined_non_empty_list!(f, "\n{}\n", assoc_ty_values, "\n")?;
        }
        write!(f, "}}")?;
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
impl<I: Interner> RenderAsRust<I> for AssocTypeId<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // TODO: use debug methods?
        f.write_str(&s.db.assoc_type_name(*self))
    }
}
impl<I: Interner> RenderAsRust<I> for OpaqueTyId<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // TODO: use debug methods?
        f.write_str(&s.db.opaque_type_name(*self))
    }
}

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
                        }
                    })
                    .to_string()
                })
                .collect::<Vec<_>>()
                .join(" + ")
        )
    })
}

impl<I: Interner> RenderAsRust<I> for TyData<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        match self {
            TyData::Dyn(dyn_ty) => {
                let s = &s.add_debrujin_index(None);
                // dyn_ty.bounds.binders creates a Self binding for the trait
                let bounds = dyn_ty.bounds.skip_binders();
                write!(
                    f,
                    "dyn {}",
                    display_self_where_clauses_as_bounds(s, bounds.as_slice(interner))
                )?;
                Ok(())
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
        match self {
            AliasTy::Projection(projection_ty) => projection_ty.fmt(s, f),
            AliasTy::Opaque(opaque_ty) => opaque_ty.fmt(s, f),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for ProjectionTy<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // <X as Y<A1, A2, A3>>::Z<B1, B2, B3>

        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.
        let (assoc_ty_datum, trait_params, assoc_type_params) = s.db.split_projection(&self);
        write!(
            f,
            "<{} as {}>::{}",
            trait_params[0].display(s),
            display_trait_with_generics(s, assoc_ty_datum.trait_id, &trait_params[1..]),
            assoc_ty_datum.id.display(s),
        )?;
        write_joined_non_empty_list!(
            f,
            "<{}>",
            assoc_type_params.iter().map(|param| param.display(s)),
            ", "
        )?;
        Ok(())
    }
}
impl<I: Interner> RenderAsRust<I> for OpaqueTy<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        write!(
            f,
            "{}",
            display_trait_with_generics(
                s,
                self.opaque_ty_id,
                self.substitution.parameters(interner),
            )
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ChalkFn<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        let s = &s.add_debrujin_index(None);
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
            self.substitution
                .parameters(interner)
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
                let parameters = parameters.iter().map(|param| param.display(s));
                write_joined_non_empty_list!(f, "<{}>", parameters, ", ")?;
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
                    datum.id.display(s),
                )?;
                let params = self.substitution.parameters(interner);
                write_joined_non_empty_list!(
                    f,
                    "<{}>",
                    params[1..].iter().map(|ty| ty.display(s)),
                    ","
                )?;
            }
            TypeName::Scalar(scalar) => write!(f, "{}", scalar.display(s))?,
            TypeName::Tuple(arity) => {
                write!(
                    f,
                    "({}{})",
                    self.substitution
                        .parameters(interner)
                        .iter()
                        .map(|p| p.display(s))
                        .format(", "),
                    if arity == 1 {
                        // need trailing single comma
                        ","
                    } else {
                        ""
                    }
                )?
            }
            TypeName::OpaqueType(_) => todo!("opaque type usage"),
            TypeName::Raw(raw) => {
                let mutability = match raw {
                    Mutability::Mut => "*mut ",
                    Mutability::Not => "*const ",
                };
                write!(
                    f,
                    "{}{}",
                    mutability,
                    self.first_type_parameter(interner).unwrap().display(s)
                )?
            }
            TypeName::Ref(mutability) => {
                let mutability = match mutability {
                    Mutability::Mut => "mut ",
                    Mutability::Not => "",
                };
                write!(
                    f,
                    "&{} {}{}",
                    self.substitution.at(interner, 0).display(s),
                    mutability,
                    self.substitution.at(interner, 1).display(s)
                )?;
            }
            TypeName::Str => write!(f, "str")?,
            TypeName::Slice => {
                write!(
                    f,
                    "[{}]",
                    self.first_type_parameter(interner).unwrap().display(s)
                )?;
            }
            TypeName::Error => write!(f, "{{error}}")?,
        }
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for Scalar {
    fn fmt(&self, _s: &WriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        use chalk_ir::{FloatTy::*, IntTy::*, UintTy::*};
        write!(
            f,
            "{}",
            match self {
                Scalar::Bool => "bool",
                Scalar::Char => "char",
                Scalar::Int(int) => match int {
                    Isize => "isize",
                    I8 => "i8",
                    I16 => "i16",
                    I32 => "i32",
                    I64 => "i64",
                    I128 => "i128",
                },
                Scalar::Uint(uint) => match uint {
                    Usize => "usize",
                    U8 => "u8",
                    U16 => "u16",
                    U32 => "u32",
                    U64 => "u64",
                    U128 => "u128",
                },
                Scalar::Float(float) => match float {
                    F32 => "f32",
                    F64 => "f64",
                },
            }
        )
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
        let interner = s.db.interner();
        let s = &s.add_debrujin_index(None);
        if !self.binders.is_empty(interner) {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders)
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        self.skip_binders().fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for QuantifiedInlineBound<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db.interner();
        let s = &s.add_debrujin_index(None);
        if !self.binders.is_empty(&interner) {
            write!(
                f,
                "forall<{}> ",
                s.binder_var_display(&self.binders)
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        self.skip_binders().fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Vec<QuantifiedWhereClause<I>> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|where_clause| { format!("{}{}", s.indent(), where_clause.display(s)) })
                .collect::<Vec<String>>()
                .join(",\n")
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
    trait_name: impl RenderAsRust<I> + 'a,
    trait_params: impl IntoIterator<Item = &'a Parameter<I>> + 'a,
) -> impl Display + 'a {
    use std::fmt::Write;
    let trait_params = trait_params.into_iter().map(|param| param.display(s));
    let mut trait_params_str = String::new();
    write_joined_non_empty_list!(trait_params_str, "<{}>", trait_params, ", ").unwrap();
    as_display(move |f| write!(f, "{}{}", trait_name.display(s), trait_params_str))
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
        match &self.alias {
            AliasTy::Projection(projection_ty) => {
                let (assoc_ty_datum, trait_params, assoc_type_params) =
                    s.db.split_projection(&projection_ty);
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
            AliasTy::Opaque(_) => todo!("opaque types"),
        }
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
        let s = &s.add_debrujin_index(None);
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
        write!(f, "type {}", self.id.display(s))?;
        write_joined_non_empty_list!(f, "<{}>", assoc_ty_params, ", ")?;

        let datum_bounds = &self.binders.skip_binders();

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
            let where_s = &s.add_indent();
            let where_clauses = datum_bounds.where_clauses.display(where_s);
            write!(f, "\n{}where\n{}", s.indent(), where_clauses)?;
        }
        write!(f, ";")?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for TraitDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index(Some(0));
        let value = self.binders.skip_binders();

        macro_rules! trait_flags {
            ($($n:ident),*) => {
                $(if self.flags.$n {
                    write!(f,"#[{}]\n",stringify!($n))?;
                })*
            }
        }

        trait_flags!(
            auto,
            marker,
            upstream,
            fundamental,
            non_enumerable,
            coinductive
        );
        let binders = s.binder_var_display(&self.binders.binders).skip(1);
        write!(f, "trait {}", self.id.display(s))?;
        write_joined_non_empty_list!(f, "<{}>", binders, ", ")?;
        if !value.where_clauses.is_empty() {
            let s = &s.add_indent();
            write!(f, "\nwhere\n{}\n", value.where_clauses.display(s))?;
        } else {
            write!(f, " ")?;
        }
        write!(f, "{{")?;
        let s = &s.add_indent();
        write_joined_non_empty_list!(
            f,
            "\n{}\n",
            self.associated_ty_ids.iter().map(|assoc_ty_id| {
                let assoc_ty_data = s.db.associated_ty_data(*assoc_ty_id);
                format!("{}{}", s.indent(), assoc_ty_data.display(s))
            }),
            "\n"
        )?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for StructDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // When support for Self in structs is added, self_binding should be
        // changed to Some(0)
        let s = &s.add_debrujin_index(None);
        let value = self.binders.skip_binders();
        write!(f, "struct {}", self.id.display(s),)?;
        write_joined_non_empty_list!(f, "<{}>", s.binder_var_display(&self.binders.binders), ", ")?;
        if !value.where_clauses.is_empty() {
            let s = &s.add_indent();
            write!(f, "\nwhere\n{}\n", value.where_clauses.display(s))?;
        } else {
            write!(f, " ")?;
        }
        write!(f, "{{")?;
        let s = &s.add_indent();
        write_joined_non_empty_list!(
            f,
            "\n{}\n",
            value.fields.iter().enumerate().map(|(idx, field)| {
                format!("{}field_{}: {}", s.indent(), idx, field.display(s))
            }),
            ",\n"
        )?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for OpaqueTyDatum<I> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        let s = &s.add_debrujin_index(None);
        let bounds = self.bound.skip_binders();
        write!(f, "opaque type {}", self.opaque_ty_id.display(s))?;
        write_joined_non_empty_list!(f, "<{}>", s.binder_var_display(&self.bound.binders), ", ")?;
        {
            let s = &s.add_debrujin_index(Some(0));
            let clauses = bounds.bounds.skip_binders();
            write!(
                f,
                ": {} = ",
                display_self_where_clauses_as_bounds(s, clauses)
            )?;
        }
        write!(f, "{};", bounds.hidden_ty.display(s))?;
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
    within_idx: IndexWithinBinding,
}

impl Display for InvertedBoundVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "_{}_{}", self.inverted_debrujin_idx, self.within_idx)
    }
}

#[derive(Clone, Debug)]
pub struct WriterState<'a, I: Interner> {
    db: &'a dyn RustIrDatabase<I>,
    indent_level: usize,
    debrujin_indices_deep: u32,
    // lowered_(inverted_debrujin_idx, index) -> src_correct_(inverted_debrujin_idx, index)
    remapping: Rc<BTreeMap<InvertedBoundVar, InvertedBoundVar>>,
    // the inverted_bound_var which maps to "Self"
    self_mapping: Option<InvertedBoundVar>,
}
type IndexWithinBinding = usize;
impl<'a, I: Interner> WriterState<'a, I> {
    pub fn new(db: &'a dyn RustIrDatabase<I>) -> Self {
        WriterState {
            db,
            indent_level: 0,
            debrujin_indices_deep: 0,
            remapping: Rc::new(BTreeMap::new()),
            self_mapping: None,
        }
    }

    fn add_indent(&self) -> Self {
        WriterState {
            indent_level: self.indent_level + 1,
            ..self.clone()
        }
    }

    fn indent(&self) -> impl Display {
        std::iter::repeat("  ").take(self.indent_level).format("")
    }

    /// Adds a level of debrujin index, and possibly a "Self" parameter.
    ///
    /// This should be called whenever recursing into the value within a
    /// [`Binders`].
    ///
    /// If `self_binding` is `Some`, then it will introduce a new variable named
    /// `Self` with the within-debrujin index given within and the innermost
    /// debrujian index after increasing debrujin index.  
    #[must_use = "this returns a new `WriterState`, and does not modify the existing one"]
    fn add_debrujin_index(&self, self_binding: Option<IndexWithinBinding>) -> Self {
        let mut new_state = self.clone();
        new_state.debrujin_indices_deep += 1;
        new_state.self_mapping = self_binding
            .map(|idx| new_state.indices_for_introduced_bound_var(idx))
            .or(self.self_mapping);
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
            remapping: Rc::new(remapping),
            ..*self
        }
    }

    /// Inverts the debrujin index so as to create a canonical name we can
    /// anywhere for each bound variable.
    ///
    /// See [`InvertedBoundVar`][InvertedBoundVar].
    fn invert_debrujin_idx(
        &self,
        debrujin_idx: u32,
        index: IndexWithinBinding,
    ) -> InvertedBoundVar {
        InvertedBoundVar {
            inverted_debrujin_idx: (self.debrujin_indices_deep as i64) - (debrujin_idx as i64),
            within_idx: index,
        }
    }

    fn apply_mappings(&self, b: InvertedBoundVar) -> impl Display {
        let remapped = self.remapping.get(&b).copied().unwrap_or(b);
        if self.self_mapping == Some(remapped) {
            "Self".to_owned()
        } else {
            remapped.to_string()
        }
    }

    fn indices_for_bound_var(&self, b: &BoundVar) -> InvertedBoundVar {
        self.invert_debrujin_idx(b.debruijn.depth(), b.index)
    }

    fn indices_for_introduced_bound_var(&self, idx: IndexWithinBinding) -> InvertedBoundVar {
        // freshly introduced bound vars will always have debrujin index of 0,
        // they're always "innermost".
        self.invert_debrujin_idx(0, idx)
    }

    fn display_bound_var(&self, b: &BoundVar) -> impl Display {
        self.apply_mappings(self.indices_for_bound_var(b))
    }

    fn name_for_introduced_bound_var(&self, idx: IndexWithinBinding) -> impl Display {
        self.apply_mappings(self.indices_for_introduced_bound_var(idx))
    }

    fn binder_var_indices<'b>(
        &'b self,
        binders: &'b ParameterKinds<I>,
    ) -> impl Iterator<Item = InvertedBoundVar> + 'b {
        binders
            .iter(self.db.interner())
            .enumerate()
            .map(move |(idx, _param)| self.indices_for_introduced_bound_var(idx))
    }

    fn binder_var_display<'b>(
        &'b self,
        binders: &'b ParameterKinds<I>,
    ) -> impl Iterator<Item = String> + 'b {
        binders
            .iter(self.db.interner())
            .zip(self.binder_var_indices(binders))
            .map(move |(parameter, var)| match parameter {
                ParameterKind::Ty(_) => format!("{}", self.apply_mappings(var)),
                ParameterKind::Lifetime(_) => format!("'{}", self.apply_mappings(var)),
            })
    }
}
