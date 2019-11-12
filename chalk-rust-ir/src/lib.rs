//! Contains the definition for the "Rust IR" -- this is basically a "lowered"
//! version of the AST, roughly corresponding to [the HIR] in the Rust
//! compiler.

use chalk_derive::Fold;
use chalk_ir::cast::Cast;
use chalk_ir::family::{ChalkIr, HasTypeFamily};
use chalk_ir::fold::{shift::Shift, Fold, Folder};
use chalk_ir::{
    Binders, Identifier, ImplId, LifetimeData, Parameter, ParameterKind, ProjectionEq,
    ProjectionTy, QuantifiedWhereClause, RawId, StructId, TraitId, TraitRef, Ty, TyData, TypeId,
    TypeName, WhereClause,
};
use std::iter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LangItem {}

/// Identifier for an "associated type value" found in some impl.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssociatedTyValueId(pub RawId);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum {
    pub polarity: Polarity,
    pub binders: Binders<ImplDatumBound>,
    pub impl_type: ImplType,
    pub associated_ty_value_ids: Vec<AssociatedTyValueId>,
}

impl ImplDatum {
    pub fn is_positive(&self) -> bool {
        self.polarity.is_positive()
    }

    pub fn trait_id(&self) -> TraitId {
        self.binders.value.trait_ref.trait_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatumBound {
    pub trait_ref: TraitRef<ChalkIr>,
    pub where_clauses: Vec<QuantifiedWhereClause<ChalkIr>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
    pub trait_ref: TraitRef<ChalkIr>,
    pub accessible_tys: Vec<Ty<ChalkIr>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    pub binders: Binders<StructDatumBound>,
    pub id: StructId,
    pub flags: StructFlags,
}

impl StructDatum {
    pub fn name(&self) -> TypeName {
        self.id.cast()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound {
    pub fields: Vec<Ty<ChalkIr>>,
    pub where_clauses: Vec<QuantifiedWhereClause<ChalkIr>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructFlags {
    pub upstream: bool,
    pub fundamental: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum {
    pub id: TraitId,

    pub binders: Binders<TraitDatumBound>,

    /// "Flags" indicate special kinds of traits, like auto traits.
    /// In Rust syntax these are represented in different ways, but in
    /// chalk we add annotations like `#[auto]`.
    pub flags: TraitFlags,

    /// The id of each associated type defined in the trait.
    pub associated_ty_ids: Vec<TypeId>,
}

impl TraitDatum {
    pub fn is_auto_trait(&self) -> bool {
        self.flags.auto
    }

    pub fn is_non_enumerable_trait(&self) -> bool {
        self.flags.non_enumerable
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatumBound {
    /// Where clauses defined on the trait:
    ///
    /// ```ignore
    /// trait Foo<T> where T: Debug { }
    ///              ^^^^^^^^^^^^^^
    /// ```
    pub where_clauses: Vec<QuantifiedWhereClause<ChalkIr>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
    pub upstream: bool,
    pub fundamental: bool,
    pub non_enumerable: bool,
}

/// An inline bound, e.g. `: Foo<K>` in `impl<K, T: Foo<K>> SomeType<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
pub enum InlineBound {
    TraitBound(TraitBound),
    ProjectionEqBound(ProjectionEqBound),
}

impl HasTypeFamily for InlineBound {
    type TypeFamily = ChalkIr;
}

pub type QuantifiedInlineBound = Binders<InlineBound>;

pub trait IntoWhereClauses {
    type Output;

    fn into_where_clauses(&self, self_ty: Ty<ChalkIr>) -> Vec<Self::Output>;
}

impl IntoWhereClauses for InlineBound {
    type Output = WhereClause<ChalkIr>;

    /// Applies the `InlineBound` to `self_ty` and lowers to a
    /// [`chalk_ir::DomainGoal`].
    ///
    /// Because an `InlineBound` does not know anything about what it's binding,
    /// you must provide that type as `self_ty`.
    fn into_where_clauses(&self, self_ty: Ty<ChalkIr>) -> Vec<WhereClause<ChalkIr>> {
        match self {
            InlineBound::TraitBound(b) => b.into_where_clauses(self_ty),
            InlineBound::ProjectionEqBound(b) => b.into_where_clauses(self_ty),
        }
    }
}

impl IntoWhereClauses for QuantifiedInlineBound {
    type Output = QuantifiedWhereClause<ChalkIr>;

    fn into_where_clauses(&self, self_ty: Ty<ChalkIr>) -> Vec<QuantifiedWhereClause<ChalkIr>> {
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
pub struct TraitBound {
    pub trait_id: TraitId,
    pub args_no_self: Vec<Parameter<ChalkIr>>,
}

impl TraitBound {
    fn into_where_clauses(&self, self_ty: Ty<ChalkIr>) -> Vec<WhereClause<ChalkIr>> {
        let trait_ref = self.as_trait_ref(self_ty);
        vec![WhereClause::Implemented(trait_ref)]
    }

    pub fn as_trait_ref(&self, self_ty: Ty<ChalkIr>) -> TraitRef<ChalkIr> {
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
pub struct ProjectionEqBound {
    pub trait_bound: TraitBound,
    pub associated_ty_id: TypeId,
    /// Does not include trait parameters.
    pub parameters: Vec<Parameter<ChalkIr>>,
    pub value: Ty<ChalkIr>,
}

impl ProjectionEqBound {
    fn into_where_clauses(&self, self_ty: Ty<ChalkIr>) -> Vec<WhereClause<ChalkIr>> {
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
    fn to_parameter(&self) -> Parameter<ChalkIr>;
}

impl<'a> ToParameter for (&'a ParameterKind<()>, usize) {
    fn to_parameter(&self) -> Parameter<ChalkIr> {
        let &(binder, index) = self;
        match *binder {
            ParameterKind::Lifetime(_) => LifetimeData::BoundVar(index).intern().cast(),
            ParameterKind::Ty(_) => TyData::BoundVar(index).intern().cast(),
        }
    }
}

/// Represents an associated type declaration found inside of a trait:
///
/// ```notrust
/// trait Foo<P1..Pn> { // P0 is Self
///     type Bar<Pn..Pm>: [bounds]
///     where
///         [where_clauses];
/// }
/// ```
///
/// The meaning of each of these parts:
///
/// * The *parameters* `P0...Pm` are all in scope for this associated type.
/// * The *bounds* `bounds` are things that the impl must prove to be true.
/// * The *where clauses* `where_clauses` are things that the impl can *assume* to be true
///   (but which projectors must prove).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyDatum {
    /// The trait this associated type is defined in.
    pub trait_id: TraitId,

    /// The ID of this associated type
    pub id: TypeId,

    /// Name of this associated type.
    pub name: Identifier,

    /// These binders represent the `P0...Pm` variables.  The binders
    /// are in the order `[Pn..Pm; P0..Pn]`. That is, the variables
    /// from `Bar` come first (corresponding to the de bruijn concept
    /// that "inner" binders are lower indices, although within a
    /// given binder we do not have an ordering).
    pub binders: Binders<AssociatedTyDatumBound>,
}

/// Encodes the parts of `AssociatedTyDatum` where the parameters
/// `P0..Pm` are in scope (`bounds` and `where_clauses`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
pub struct AssociatedTyDatumBound {
    /// Bounds on the associated type itself.
    ///
    /// These must be proven by the implementer, for all possible parameters that
    /// would result in a well-formed projection.
    pub bounds: Vec<QuantifiedInlineBound>,

    /// Where clauses that must hold for the projection to be well-formed.
    pub where_clauses: Vec<QuantifiedWhereClause<ChalkIr>>,
}

impl HasTypeFamily for AssociatedTyDatumBound {
    type TypeFamily = ChalkIr;
}

impl AssociatedTyDatum {
    /// Returns the associated ty's bounds applied to the projection type, e.g.:
    ///
    /// ```notrust
    /// Implemented(<?0 as Foo>::Item<?1>: Sized)
    /// ```
    ///
    /// these quantified where clauses are in the scope of the
    /// `binders` field.
    pub fn bounds_on_self(&self) -> Vec<QuantifiedWhereClause<ChalkIr>> {
        let Binders { binders, value } = &self.binders;

        // Create a list `P0...Pn` of references to the binders in
        // scope for this associated type:
        let parameters = binders.iter().zip(0..).map(|p| p.to_parameter()).collect();

        // The self type will be `<P0 as Foo<P1..Pn>>::Item<Pn..Pm>` etc
        let self_ty = TyData::Projection(ProjectionTy {
            associated_ty_id: self.id,
            parameters,
        })
        .intern();

        // Now use that as the self type for the bounds, transforming
        // something like `type Bar<Pn..Pm>: Debug` into
        //
        // ```
        // <P0 as Foo<P1..Pn>>::Item<Pn..Pm>: Debug
        // ```
        value
            .bounds
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
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
    /// beyond those from the impl. This would be empty for normal
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
#[has_type_family(ChalkIr)]
pub struct AssociatedTyValueBound {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    pub ty: Ty<ChalkIr>,
}

impl HasTypeFamily for AssociatedTyValueBound {
    type TypeFamily = ChalkIr;
}

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Polarity {
    Positive,
    Negative,
}

impl Polarity {
    pub fn is_positive(&self) -> bool {
        match *self {
            Polarity::Positive => true,
            Polarity::Negative => false,
        }
    }
}
