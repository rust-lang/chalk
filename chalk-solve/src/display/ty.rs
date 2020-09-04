//! Writer logic for types.
//!
//! Contains the highly-recursive logic for writing `TyData` and its variants.
use std::fmt::{Formatter, Result};

use crate::split::Split;
use chalk_ir::{interner::Interner, *};
use itertools::Itertools;

use super::{
    display_self_where_clauses_as_bounds, display_type_with_generics, render_trait::RenderAsRust,
    state::InternalWriterState,
};

impl<I: Interner> RenderAsRust<I> for TyData<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        match self {
            TyData::Dyn(dyn_ty) => {
                // the lifetime needs to be outside of the bounds, so we
                // introduce a new scope for the bounds
                {
                    let s = &s.add_debrujin_index(None);
                    // dyn_ty.bounds.binders creates a Self binding for the trait
                    let bounds = dyn_ty.bounds.skip_binders();

                    write!(
                        f,
                        "dyn {}",
                        display_self_where_clauses_as_bounds(s, bounds.as_slice(interner)),
                    )?;
                }

                write!(f, " + {}", dyn_ty.lifetime.display(s))?;
                Ok(())
            }
            TyData::BoundVar(bound_var) => write!(f, "{}", s.display_bound_var(bound_var)),
            TyData::InferenceVar(_, _) => write!(f, "_"),
            TyData::Alias(alias_ty) => alias_ty.fmt(s, f),
            TyData::Apply(apply_ty) => apply_ty.fmt(s, f),
            TyData::Function(func) => func.fmt(s, f),
            TyData::Placeholder(_) => write!(f, "<placeholder>"),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for AliasTy<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            AliasTy::Projection(projection_ty) => projection_ty.fmt(s, f),
            AliasTy::Opaque(opaque_ty) => opaque_ty.fmt(s, f),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for ProjectionTy<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // <X as Y<A1, A2, A3>>::Z<B1, B2, B3>

        // Now, we split out A*, Y/Z and B*:
        // trait_params is X, A1, A2, A3,
        // assoc_type_params is B1, B2, B3,
        // assoc_ty_datum stores info about Y and Z.
        let (assoc_ty_datum, trait_params, assoc_type_params) = s.db().split_projection(&self);
        write!(
            f,
            "<{} as {}>::{}",
            trait_params[0].display(s),
            display_type_with_generics(s, assoc_ty_datum.trait_id, &trait_params[1..]),
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
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        write!(
            f,
            "{}",
            display_type_with_generics(s, self.opaque_ty_id, self.substitution.as_slice(interner),)
        )
    }
}

impl<I: Interner> RenderAsRust<I> for FnPointer<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        let s = &s.add_debrujin_index(None);
        if self.num_binders > 0 {
            write!(
                f,
                "for<{}> ",
                (0..self.num_binders)
                    .map(|n| format!("'{}", s.name_for_introduced_bound_var(n)))
                    .format(", ")
            )?;
        }
        let parameters = self.substitution.as_slice(interner);
        write!(
            f,
            "fn({}) -> {}",
            parameters[..parameters.len() - 1]
                .iter()
                .map(|param| param.display(s))
                .format(", "),
            parameters[parameters.len() - 1].display(s),
        )
    }
}

impl<I: Interner> RenderAsRust<I> for ApplicationTy<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        let interner = s.db().interner();
        match self.name {
            TypeName::Adt(sid) => {
                write!(f, "{}", sid.display(s))?;
                let parameters = self.substitution.as_slice(interner);
                let parameters = parameters.iter().map(|param| param.display(s));
                write_joined_non_empty_list!(f, "<{}>", parameters, ", ")?;
            }
            TypeName::AssociatedType(assoc_type_id) => {
                // (Iterator::Item)(x)
                // should be written in Rust as <X as Iterator>::Item
                let datum = s.db().associated_ty_data(assoc_type_id);
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
                let params = self.substitution.as_slice(interner);
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
                        .as_slice(interner)
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
            TypeName::OpaqueType(opaque_ty_id) => {
                write!(
                    f,
                    "{}",
                    display_type_with_generics(
                        s,
                        opaque_ty_id,
                        self.substitution.as_slice(interner)
                    )
                )?;
            }
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
            TypeName::Never => write!(f, "!")?,

            // FIXME: write out valid types for these variants
            TypeName::FnDef(_) => write!(f, "<fn_def>")?,
            TypeName::Closure(..) => write!(f, "<closure>")?,
            TypeName::Foreign(_) => write!(f, "<foreign>")?,

            TypeName::Array => write!(
                f,
                "[{}; {}]",
                self.first_type_parameter(interner).unwrap().display(s),
                self.substitution.at(interner, 1).display(s)
            )?,
        }
        Ok(())
    }
}

impl<I: Interner> RenderAsRust<I> for Scalar {
    fn fmt(&self, _s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
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
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            LifetimeData::BoundVar(v) => write!(f, "'{}", s.display_bound_var(v)),
            LifetimeData::InferenceVar(_) => write!(f, "'_"),
            LifetimeData::Placeholder(ix) => {
                write!(f, "'_placeholder_{}_{}", ix.ui.counter, ix.idx)
            }
            // Matching the void ensures at compile time that this code is
            // unreachable
            LifetimeData::Phantom(void, _) => match *void {},
        }
    }
}

impl<I: Interner> RenderAsRust<I> for ConstData<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value.display(s))
    }
}

impl<I: Interner> RenderAsRust<I> for ConstValue<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        match self {
            ConstValue::BoundVar(v) => write!(f, "{}", s.display_bound_var(v)),
            ConstValue::InferenceVar(_) => write!(f, "_"),
            ConstValue::Placeholder(_) => write!(f, "<const placeholder>"),
            ConstValue::Concrete(_value) => unimplemented!("const values"),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for GenericArgData<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        match self {
            GenericArgData::Ty(ty) => write!(f, "{}", ty.display(s)),
            GenericArgData::Lifetime(lt) => write!(f, "{}", lt.display(s)),
            GenericArgData::Const(const_ty) => write!(f, "{}", const_ty.display(s)),
        }
    }
}

impl<I: Interner> RenderAsRust<I> for Ty<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to TyData
        self.data(s.db().interner()).fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Lifetime<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to LifetimeData
        self.data(s.db().interner()).fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for Const<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result {
        self.data(s.db().interner()).fmt(s, f)
    }
}

impl<I: Interner> RenderAsRust<I> for GenericArg<I> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &'_ mut Formatter<'_>) -> Result {
        // delegate to GenericArgData
        self.data(s.db().interner()).fmt(s, f)
    }
}
