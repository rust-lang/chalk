//! Contains the definition for the "Rust IR" -- this is basically a "lowered"
//! version of the AST, roughly corresponding to [the HIR] in the Rust
//! compiler.

#[macro_use]
extern crate chalk_ir;

use chalk_ir::cast::Cast;
use chalk_ir::fold::shift::Shift;
use chalk_ir::{
    ApplicationTy, Binders, Identifier, ImplId, Lifetime, Parameter, ParameterKind, ProjectionEq,
    ProjectionTy, QuantifiedWhereClause, TraitId, TraitRef, Ty, TypeId, WhereClause,
};
use std::iter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LangItem {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum {
    pub binders: Binders<ImplDatumBound>,
}

impl ImplDatum {
    pub fn is_positive(&self) -> bool {
        match self.binders.value.trait_ref {
            PolarizedTraitRef::Positive(_) => true,
            PolarizedTraitRef::Negative(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatumBound {
    pub trait_ref: PolarizedTraitRef,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub associated_ty_values: Vec<AssociatedTyValue>,
    pub impl_type: ImplType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImplType {
    Local,
    External,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatum {
    pub binders: Binders<DefaultImplDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatumBound {
    pub trait_ref: TraitRef,
    pub accessible_tys: Vec<Ty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    pub binders: Binders<StructDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound {
    pub self_ty: ApplicationTy,
    pub fields: Vec<Ty>,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub flags: StructFlags,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructFlags {
    pub upstream: bool,
    pub fundamental: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum {
    pub binders: Binders<TraitDatumBound>,
}

impl TraitDatum {
    pub fn is_auto_trait(&self) -> bool {
        self.binders.value.flags.auto
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatumBound {
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub flags: TraitFlags,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
    pub upstream: bool,
    pub fundamental: bool,
}

/// An inline bound, e.g. `: Foo<K>` in `impl<K, T: Foo<K>> SomeType<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InlineBound {
    TraitBound(TraitBound),
    ProjectionEqBound(ProjectionEqBound),
}

enum_fold!(InlineBound[] { TraitBound(a), ProjectionEqBound(a) });

pub type QuantifiedInlineBound = Binders<InlineBound>;

pub trait IntoWhereClauses {
    type Output;

    fn into_where_clauses(&self, self_ty: Ty) -> Vec<Self::Output>;
}

impl IntoWhereClauses for InlineBound {
    type Output = WhereClause;

    /// Applies the `InlineBound` to `self_ty` and lowers to a
    /// [`chalk_ir::DomainGoal`].
    ///
    /// Because an `InlineBound` does not know anything about what it's binding,
    /// you must provide that type as `self_ty`.
    fn into_where_clauses(&self, self_ty: Ty) -> Vec<WhereClause> {
        match self {
            InlineBound::TraitBound(b) => b.into_where_clauses(self_ty),
            InlineBound::ProjectionEqBound(b) => b.into_where_clauses(self_ty),
        }
    }
}

impl IntoWhereClauses for QuantifiedInlineBound {
    type Output = QuantifiedWhereClause;

    fn into_where_clauses(&self, self_ty: Ty) -> Vec<QuantifiedWhereClause> {
        let self_ty = self_ty.shifted_in(self.binders.len());
        self.value
            .into_where_clauses(self_ty)
            .into_iter()
            .map(|wc| Binders {
                binders: self.binders.clone(),
                value: wc,
            })
            .collect()
    }
}

/// Represents a trait bound on e.g. a type or type parameter.
/// Does not know anything about what it's binding.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitBound {
    pub trait_id: TraitId,
    pub args_no_self: Vec<Parameter>,
}

struct_fold!(TraitBound {
    trait_id,
    args_no_self,
});

impl TraitBound {
    fn into_where_clauses(&self, self_ty: Ty) -> Vec<WhereClause> {
        let trait_ref = self.as_trait_ref(self_ty);
        vec![WhereClause::Implemented(trait_ref)]
    }

    pub fn as_trait_ref(&self, self_ty: Ty) -> TraitRef {
        TraitRef {
            trait_id: self.trait_id,
            parameters: iter::once(self_ty.cast())
                .chain(self.args_no_self.iter().cloned())
                .collect(),
        }
    }
}
/// Represents a projection equality bound on e.g. a type or type parameter.
/// Does not know anything about what it's binding.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectionEqBound {
    pub trait_bound: TraitBound,
    pub associated_ty_id: TypeId,
    /// Does not include trait parameters.
    pub parameters: Vec<Parameter>,
    pub value: Ty,
}

struct_fold!(ProjectionEqBound {
    trait_bound,
    associated_ty_id,
    parameters,
    value,
});

impl ProjectionEqBound {
    fn into_where_clauses(&self, self_ty: Ty) -> Vec<WhereClause> {
        let trait_ref = self.trait_bound.as_trait_ref(self_ty);

        let mut parameters = self.parameters.clone();
        parameters.extend(trait_ref.parameters.clone());

        vec![
            WhereClause::Implemented(trait_ref),
            WhereClause::ProjectionEq(ProjectionEq {
                projection: ProjectionTy {
                    associated_ty_id: self.associated_ty_id,
                    parameters: parameters,
                },
                ty: self.value.clone(),
            }),
        ]
    }
}

pub trait Anonymize {
    /// Utility function that converts from a list of generic parameters
    /// which *have* names (`ParameterKind<Identifier>`) to a list of
    /// "anonymous" generic parameters that just preserves their
    /// kinds (`ParameterKind<()>`). Often convenient in lowering.
    fn anonymize(&self) -> Vec<ParameterKind<()>>;
}

impl Anonymize for [ParameterKind<Identifier>] {
    fn anonymize(&self) -> Vec<ParameterKind<()>> {
        self.iter().map(|pk| pk.map(|_| ())).collect()
    }
}

pub trait ToParameter {
    /// Utility for converting a list of all the binders into scope
    /// into references to those binders. Simply pair the binders with
    /// the indices, and invoke `to_parameter()` on the `(binder,
    /// index)` pair. The result will be a reference to a bound
    /// variable of appropriate kind at the corresponding index.
    fn to_parameter(&self) -> Parameter;
}

impl<'a> ToParameter for (&'a ParameterKind<()>, usize) {
    fn to_parameter(&self) -> Parameter {
        let &(binder, index) = self;
        match *binder {
            ParameterKind::Lifetime(_) => Lifetime::BoundVar(index).cast(),
            ParameterKind::Ty(_) => Ty::BoundVar(index).cast(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyDatum {
    /// The trait this associated type is defined in.
    pub trait_id: TraitId,

    /// The ID of this associated type
    pub id: TypeId,

    /// Name of this associated type.
    pub name: Identifier,

    /// Parameters on this associated type, beginning with those from the trait,
    /// but possibly including more.
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,

    /// Bounds on the associated type itself.
    ///
    /// These must be proven by the implementer, for all possible parameters that
    /// would result in a well-formed projection.
    pub bounds: Vec<QuantifiedInlineBound>,

    /// Where clauses that must hold for the projection to be well-formed.
    pub where_clauses: Vec<QuantifiedWhereClause>,
}

impl AssociatedTyDatum {
    /// Returns the associated ty's bounds applied to the projection type, e.g.:
    ///
    /// ```notrust
    /// Implemented(<?0 as Foo>::Item<?1>: Sized)
    /// ```
    pub fn bounds_on_self(&self) -> Vec<QuantifiedWhereClause> {
        let parameters = self
            .parameter_kinds
            .anonymize()
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter())
            .collect();
        let self_ty = Ty::Projection(ProjectionTy {
            associated_ty_id: self.id,
            parameters,
        });
        self.bounds
            .iter()
            .flat_map(|b| b.into_where_clauses(self_ty.clone()))
            .collect()
    }
}

/// Represents the *value* of an associated type that is assigned
/// from within some impl.
///
/// ```ignore
/// impl Iterator for Foo {
///     type Item = XXX; // <-- represents this line!
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValue {
    /// Impl in which this associated type value is found.  You might
    /// need to look at this to find the generic parameters defined on
    /// the impl, for example.
    ///
    /// ```ignore
    /// impl Iterator for Foo { // <-- refers to this impl
    ///     type Item = XXX; // <-- (where this is self)
    /// }
    /// ```
    pub impl_id: ImplId,

    /// Associated type being defined.
    ///
    /// ```ignore
    /// impl Iterator for Foo {
    ///     type Item = XXX; // <-- (where this is self)
    /// }
    /// ...
    /// trait Iterator {
    ///     type Item; // <-- refers to this declaration here!
    /// }
    /// ```
    pub associated_ty_id: TypeId,

    /// Additional binders declared on the associated type itself,
    /// beyond thos from the impl. This would be empty for normal
    /// associated types, but non-empty for generic associated types.
    ///
    /// ```ignore
    /// impl<T> Iterable for Vec<T> {
    ///     type Iter<'a> = vec::Iter<'a, T>;
    ///           // ^^^^ refers to these generics here
    /// }
    /// ```
    pub value: Binders<AssociatedTyValueBound>,
}

struct_fold!(AssociatedTyValue {
    impl_id,
    associated_ty_id,
    value,
});

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValueBound {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    pub ty: Ty,
}

struct_fold!(AssociatedTyValueBound { ty });

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub binders: Binders<()>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum PolarizedTraitRef {
    Positive(TraitRef),
    Negative(TraitRef),
}

enum_fold!(PolarizedTraitRef[] { Positive(a), Negative(a) });

impl PolarizedTraitRef {
    pub fn is_positive(&self) -> bool {
        match *self {
            PolarizedTraitRef::Positive(_) => true,
            PolarizedTraitRef::Negative(_) => false,
        }
    }

    pub fn trait_ref(&self) -> &TraitRef {
        match *self {
            PolarizedTraitRef::Positive(ref tr) | PolarizedTraitRef::Negative(ref tr) => tr,
        }
    }
}
