//! Contains the definition for the "Rust IR" -- this is basically a "lowered"
//! version of the AST, roughly corresponding to [the HIR] in the Rust
//! compiler.

use chalk_derive::{Fold, HasTypeFamily};
use chalk_ir::cast::Cast;
use chalk_ir::family::{HasTypeFamily, TargetTypeFamily, TypeFamily};
use chalk_ir::fold::{shift::Shift, Fold, Folder};
use chalk_ir::{
    AliasEq, AliasTy, AssocTypeId, Binders, Identifier, ImplId, LifetimeData, Parameter,
    ParameterKind, QuantifiedWhereClause, RawId, StructId, Substitution, TraitId, TraitRef, Ty,
    TyData, TypeName, WhereClause,
};
use std::iter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LangItem {}

/// Identifier for an "associated type value" found in some impl.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssociatedTyValueId(pub RawId);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum<TF: TypeFamily> {
    pub polarity: Polarity,
    pub binders: Binders<ImplDatumBound<TF>>,
    pub impl_type: ImplType,
    pub associated_ty_value_ids: Vec<AssociatedTyValueId>,
}

impl<TF: TypeFamily> ImplDatum<TF> {
    pub fn is_positive(&self) -> bool {
        self.polarity.is_positive()
    }

    pub fn trait_id(&self) -> TraitId<TF> {
        self.binders.value.trait_ref.trait_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatumBound<TF: TypeFamily> {
    pub trait_ref: TraitRef<TF>,
    pub where_clauses: Vec<QuantifiedWhereClause<TF>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImplType {
    Local,
    External,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatum<TF: TypeFamily> {
    pub binders: Binders<DefaultImplDatumBound<TF>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatumBound<TF: TypeFamily> {
    pub trait_ref: TraitRef<TF>,
    pub accessible_tys: Vec<Ty<TF>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum<TF: TypeFamily> {
    pub binders: Binders<StructDatumBound<TF>>,
    pub id: StructId<TF>,
    pub flags: StructFlags,
}

impl<TF: TypeFamily> StructDatum<TF> {
    pub fn name(&self) -> TypeName<TF> {
        self.id.cast()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound<TF: TypeFamily> {
    pub fields: Vec<Ty<TF>>,
    pub where_clauses: Vec<QuantifiedWhereClause<TF>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructFlags {
    pub upstream: bool,
    pub fundamental: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum<TF: TypeFamily> {
    pub id: TraitId<TF>,

    pub binders: Binders<TraitDatumBound<TF>>,

    /// "Flags" indicate special kinds of traits, like auto traits.
    /// In Rust syntax these are represented in different ways, but in
    /// chalk we add annotations like `#[auto]`.
    pub flags: TraitFlags,

    /// The id of each associated type defined in the trait.
    pub associated_ty_ids: Vec<AssocTypeId<TF>>,
}

impl<TF: TypeFamily> TraitDatum<TF> {
    pub fn is_auto_trait(&self) -> bool {
        self.flags.auto
    }

    pub fn is_non_enumerable_trait(&self) -> bool {
        self.flags.non_enumerable
    }

    pub fn is_coinductive_trait(&self) -> bool {
        self.flags.coinductive
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatumBound<TF: TypeFamily> {
    /// Where clauses defined on the trait:
    ///
    /// ```ignore
    /// trait Foo<T> where T: Debug { }
    ///              ^^^^^^^^^^^^^^
    /// ```
    pub where_clauses: Vec<QuantifiedWhereClause<TF>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
    pub upstream: bool,
    pub fundamental: bool,
    pub non_enumerable: bool,
    pub coinductive: bool,
}

/// An inline bound, e.g. `: Foo<K>` in `impl<K, T: Foo<K>> SomeType<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasTypeFamily)]
pub enum InlineBound<TF: TypeFamily> {
    TraitBound(TraitBound<TF>),
    AliasEqBound(AliasEqBound<TF>),
}

#[allow(type_alias_bounds)]
pub type QuantifiedInlineBound<TF: TypeFamily> = Binders<InlineBound<TF>>;

pub trait IntoWhereClauses<TF: TypeFamily> {
    type Output;

    fn into_where_clauses(&self, self_ty: Ty<TF>) -> Vec<Self::Output>;
}

impl<TF: TypeFamily> IntoWhereClauses<TF> for InlineBound<TF> {
    type Output = WhereClause<TF>;

    /// Applies the `InlineBound` to `self_ty` and lowers to a
    /// [`chalk_ir::DomainGoal`].
    ///
    /// Because an `InlineBound` does not know anything about what it's binding,
    /// you must provide that type as `self_ty`.
    fn into_where_clauses(&self, self_ty: Ty<TF>) -> Vec<WhereClause<TF>> {
        match self {
            InlineBound::TraitBound(b) => b.into_where_clauses(self_ty),
            InlineBound::AliasEqBound(b) => b.into_where_clauses(self_ty),
        }
    }
}

impl<TF: TypeFamily> IntoWhereClauses<TF> for QuantifiedInlineBound<TF> {
    type Output = QuantifiedWhereClause<TF>;

    fn into_where_clauses(&self, self_ty: Ty<TF>) -> Vec<QuantifiedWhereClause<TF>> {
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
pub struct TraitBound<TF: TypeFamily> {
    pub trait_id: TraitId<TF>,
    pub args_no_self: Vec<Parameter<TF>>,
}

impl<TF: TypeFamily> TraitBound<TF> {
    fn into_where_clauses(&self, self_ty: Ty<TF>) -> Vec<WhereClause<TF>> {
        let trait_ref = self.as_trait_ref(self_ty);
        vec![WhereClause::Implemented(trait_ref)]
    }

    pub fn as_trait_ref(&self, self_ty: Ty<TF>) -> TraitRef<TF> {
        TraitRef {
            trait_id: self.trait_id,
            substitution: Substitution::from(
                iter::once(self_ty.cast()).chain(self.args_no_self.iter().cloned()),
            ),
        }
    }
}

/// Represents an alias equality bound on e.g. a type or type parameter.
/// Does not know anything about what it's binding.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
pub struct AliasEqBound<TF: TypeFamily> {
    pub trait_bound: TraitBound<TF>,
    pub associated_ty_id: AssocTypeId<TF>,
    /// Does not include trait parameters.
    pub parameters: Vec<Parameter<TF>>,
    pub value: Ty<TF>,
}

impl<TF: TypeFamily> AliasEqBound<TF> {
    fn into_where_clauses(&self, self_ty: Ty<TF>) -> Vec<WhereClause<TF>> {
        let trait_ref = self.trait_bound.as_trait_ref(self_ty);

        let substitution = Substitution::from(
            self.parameters
                .iter()
                .cloned()
                .chain(trait_ref.substitution.iter().cloned()),
        );

        vec![
            WhereClause::Implemented(trait_ref),
            WhereClause::AliasEq(AliasEq {
                alias: AliasTy {
                    associated_ty_id: self.associated_ty_id,
                    substitution,
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
    fn to_parameter<TF: TypeFamily>(&self) -> Parameter<TF>;
}

impl<'a> ToParameter for (&'a ParameterKind<()>, usize) {
    fn to_parameter<TF: TypeFamily>(&self) -> Parameter<TF> {
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
pub struct AssociatedTyDatum<TF: TypeFamily> {
    /// The trait this associated type is defined in.
    pub trait_id: TraitId<TF>,

    /// The ID of this associated type
    pub id: AssocTypeId<TF>,

    /// Name of this associated type.
    pub name: Identifier,

    /// These binders represent the `P0...Pm` variables.  The binders
    /// are in the order `[Pn..Pm; P0..Pn]`. That is, the variables
    /// from `Bar` come first (corresponding to the de bruijn concept
    /// that "inner" binders are lower indices, although within a
    /// given binder we do not have an ordering).
    pub binders: Binders<AssociatedTyDatumBound<TF>>,
}

/// Encodes the parts of `AssociatedTyDatum` where the parameters
/// `P0..Pm` are in scope (`bounds` and `where_clauses`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasTypeFamily)]
pub struct AssociatedTyDatumBound<TF: TypeFamily> {
    /// Bounds on the associated type itself.
    ///
    /// These must be proven by the implementer, for all possible parameters that
    /// would result in a well-formed projection.
    pub bounds: Vec<QuantifiedInlineBound<TF>>,

    /// Where clauses that must hold for the projection to be well-formed.
    pub where_clauses: Vec<QuantifiedWhereClause<TF>>,
}

impl<TF: TypeFamily> AssociatedTyDatum<TF> {
    /// Returns the associated ty's bounds applied to the projection type, e.g.:
    ///
    /// ```notrust
    /// Implemented(<?0 as Foo>::Item<?1>: Sized)
    /// ```
    ///
    /// these quantified where clauses are in the scope of the
    /// `binders` field.
    pub fn bounds_on_self(&self) -> Vec<QuantifiedWhereClause<TF>> {
        let Binders { binders, value } = &self.binders;

        // Create a list `P0...Pn` of references to the binders in
        // scope for this associated type:
        let substitution = Substitution::from(binders.iter().zip(0..).map(|p| p.to_parameter()));

        // The self type will be `<P0 as Foo<P1..Pn>>::Item<Pn..Pm>` etc
        let self_ty = TyData::Alias(AliasTy {
            associated_ty_id: self.id,
            substitution,
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
pub struct AssociatedTyValue<TF: TypeFamily> {
    /// Impl in which this associated type value is found.  You might
    /// need to look at this to find the generic parameters defined on
    /// the impl, for example.
    ///
    /// ```ignore
    /// impl Iterator for Foo { // <-- refers to this impl
    ///     type Item = XXX; // <-- (where this is self)
    /// }
    /// ```
    pub impl_id: ImplId<TF>,

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
    pub associated_ty_id: AssocTypeId<TF>,

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
    pub value: Binders<AssociatedTyValueBound<TF>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasTypeFamily)]
pub struct AssociatedTyValueBound<TF: TypeFamily> {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    pub ty: Ty<TF>,
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
