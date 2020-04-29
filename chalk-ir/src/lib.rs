#![deny(rust_2018_idioms)]

// Allows macros to refer to this crate as `::chalk_ir`
extern crate self as chalk_ir;

use crate::cast::{Cast, CastTo};
use crate::fold::shift::Shift;
use crate::fold::{Fold, Folder, Subst, SuperFold};
use crate::visit::{SuperVisit, Visit, VisitExt, VisitResult, Visitor};
use chalk_derive::{Fold, HasInterner, SuperVisit, Visit, Zip};
use chalk_engine::fallible::*;
use std::iter;
use std::marker::PhantomData;

pub use crate::debug::SeparatorTraitRef;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Void {}

macro_rules! impl_debugs {
    ($($id:ident), *) => {
        $(
            impl<I: Interner> std::fmt::Debug for $id<I> {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    write!(fmt, "{}({:?})", stringify!($id), self.0)
                }
            }
        )*
    };
}

#[macro_use]
mod macros;

#[macro_use]
pub mod zip;

#[macro_use]
pub mod fold;

#[macro_use]
pub mod visit;

pub mod cast;

pub mod interner;
use interner::{HasInterner, Interner};

pub mod could_match;
pub mod debug;
#[cfg(any(test, feature = "default-interner"))]
pub mod tls;

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner)]
/// The set of assumptions we've made so far, and the current number of
/// universal (forall) quantifiers we're within.
pub struct Environment<I: Interner> {
    pub clauses: ProgramClauses<I>,
}

impl<I: Interner> Environment<I> {
    pub fn new(interner: &I) -> Self {
        Environment {
            clauses: ProgramClauses::new(interner),
        }
    }

    pub fn add_clauses<II>(&self, interner: &I, clauses: II) -> Self
    where
        II: IntoIterator<Item = ProgramClause<I>>,
    {
        let mut env = self.clone();
        env.clauses =
            ProgramClauses::from(interner, env.clauses.iter(interner).cloned().chain(clauses));
        env
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, Visit)]
pub struct InEnvironment<G: HasInterner> {
    pub environment: Environment<G::Interner>,
    pub goal: G,
}

impl<G: HasInterner> InEnvironment<G> {
    pub fn new(environment: &Environment<G::Interner>, goal: G) -> Self {
        InEnvironment {
            environment: environment.clone(),
            goal,
        }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
    where
        OP: FnOnce(G) -> H,
        H: HasInterner<Interner = G::Interner>,
    {
        InEnvironment {
            environment: self.environment,
            goal: op(self.goal),
        }
    }
}

impl<G: HasInterner> HasInterner for InEnvironment<G> {
    type Interner = G::Interner;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntTy {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UintTy {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatTy {
    F32,
    F64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fold, Visit)]
pub enum TypeName<I: Interner> {
    /// a type like `Vec<T>`
    Struct(StructId<I>),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(AssocTypeId<I>),

    /// a scalar type like `bool` or `u32`
    Scalar(Scalar),

    /// a tuple of the given arity
    Tuple(usize),

    /// a placeholder for opaque types like `impl Trait`
    OpaqueType(OpaqueTyId<I>),

    /// This can be used to represent an error, e.g. during name resolution of a type.
    /// Chalk itself will not produce this, just pass it through when given.
    Error,
}

impl<I: Interner> HasInterner for TypeName<I> {
    type Interner = I;
}

/// An universe index is how a universally quantified parameter is
/// represented when it's binder is moved into the environment.
/// An example chain of transformations would be:
/// `forall<T> { Goal(T) }` (syntactical representation)
/// `forall { Goal(?0) }` (used a DeBruijn index)
/// `Goal(!U1)` (the quantifier was moved to the environment and replaced with a universe index)
/// See https://rustc-dev-guide.rust-lang.org/borrow_check/region_inference.html#placeholders-and-universes for more.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseIndex {
    pub counter: usize,
}

impl UniverseIndex {
    pub const ROOT: UniverseIndex = UniverseIndex { counter: 0 };

    pub fn root() -> UniverseIndex {
        Self::ROOT
    }

    pub fn can_see(self, ui: UniverseIndex) -> bool {
        self.counter >= ui.counter
    }

    pub fn next(self) -> UniverseIndex {
        UniverseIndex {
            counter: self.counter + 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructId<I: Interner>(pub I::DefId);

/// The id of a trait definition; could be used to load the trait datum by
/// invoking the [`trait_datum`] method.
///
/// [`trait_datum`]: ../chalk_solve/trait.RustIrDatabase.html#tymethod.trait_datum
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImplId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClauseId<I: Interner>(pub I::DefId);

/// The id for the associated type member of a trait. The details of the type
/// can be found by invoking the [`associated_ty_data`] method.
///
/// [`associated_ty_data`]: ../chalk_solve/trait.RustIrDatabase.html#tymethod.associated_ty_data
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssocTypeId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpaqueTyId<I: Interner>(pub I::DefId);

impl_debugs!(ImplId, ClauseId);

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Ty<I: Interner> {
    interned: I::InternedType,
}

impl<I: Interner> Ty<I> {
    pub fn new(interner: &I, data: impl CastTo<TyData<I>>) -> Self {
        Ty {
            interned: I::intern_ty(interner, data.cast(interner)),
        }
    }

    pub fn interned(&self) -> &I::InternedType {
        &self.interned
    }

    pub fn data(&self, interner: &I) -> &TyData<I> {
        I::ty_data(interner, &self.interned)
    }

    pub fn from_env(&self) -> FromEnv<I> {
        FromEnv::Ty(self.clone())
    }

    pub fn well_formed(&self) -> WellFormed<I> {
        WellFormed::Ty(self.clone())
    }

    /// Creates a domain goal `FromEnv(T)` where `T` is this type.
    pub fn into_from_env_goal(self, interner: &I) -> DomainGoal<I> {
        self.from_env().cast(interner)
    }

    /// If this is a `TyData::BoundVar(d)`, returns `Some(d)` else `None`.
    pub fn bound(&self, interner: &I) -> Option<BoundVar> {
        if let TyData::BoundVar(bv) = self.data(interner) {
            Some(*bv)
        } else {
            None
        }
    }

    /// If this is a `TyData::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self, interner: &I) -> Option<InferenceVar> {
        if let TyData::InferenceVar(depth) = self.data(interner) {
            Some(*depth)
        } else {
            None
        }
    }

    /// Returns true if this is a `BoundVar` or `InferenceVar`.
    pub fn is_var(&self, interner: &I) -> bool {
        match self.data(interner) {
            TyData::BoundVar(_) | TyData::InferenceVar(_) => true,
            _ => false,
        }
    }

    pub fn is_alias(&self, interner: &I) -> bool {
        match self.data(interner) {
            TyData::Alias(..) => true,
            _ => false,
        }
    }

    /// True if this type contains "bound" types/lifetimes, and hence
    /// needs to be shifted across binders. This is a very inefficient
    /// check, intended only for debug assertions, because I am lazy.
    pub fn needs_shift(&self, interner: &I) -> bool {
        self.has_free_vars(interner)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
pub enum TyData<I: Interner> {
    /// An "application" type is one that applies the set of type
    /// arguments to some base type. For example, `Vec<u32>` would be
    /// "applying" the parameters `[u32]` to the code type `Vec`.
    /// This type is also used for base types like `u32` (which just apply
    /// an empty list).
    Apply(ApplicationTy<I>),

    /// instantiated form a universally quantified type, e.g., from
    /// `forall<T> { .. }`. Stands in as a representative of "some
    /// unknown type".
    Placeholder(PlaceholderIndex),

    /// A "dyn" type is a trait object type created via the "dyn Trait" syntax.
    /// In the chalk parser, the traits that the object represents is parsed as
    /// a QuantifiedInlineBound, and is then changed to a list of where clauses
    /// during lowering.
    ///
    /// See the `Opaque` variant for a discussion about the use of
    /// binders here.
    Dyn(DynTy<I>),

    /// An "alias" type represents some form of type alias, such as:
    /// - An associated type projection like `<T as Iterator>::Item`
    /// - `impl Trait` types
    /// - Named type aliases like `type Foo<X> = Vec<X>`
    Alias(AliasTy<I>),

    /// A function type such as `for<'a> fn(&'a u32)`.
    /// Note that "higher-ranked" types (starting with `for<>`) are either
    /// function types or dyn types, and do not appear otherwise in Rust
    /// surface syntax.
    Function(Fn<I>),

    /// References the binding at the given depth. The index is a [de
    /// Bruijn index], so it counts back through the in-scope binders.
    BoundVar(BoundVar),

    /// Inference variable defined in the current inference context.
    InferenceVar(InferenceVar),
}

impl<I: Interner> TyData<I> {
    pub fn intern(self, interner: &I) -> Ty<I> {
        Ty::new(interner, self)
    }
}

/// Identifies a particular bound variable within a binder.
/// Variables are identified by the combination of a [`DebruijnIndex`],
/// which identifies the *binder*, and an index within that binder.
///
/// Consider this case:
///
/// ```ignore
/// forall<'a, 'b> { forall<'c, 'd> { ... } }
/// ```
///
/// Within the `...` term:
///
/// * the variable `'a` have a debruijn index of 1 and index 0
/// * the variable `'b` have a debruijn index of 1 and index 1
/// * the variable `'c` have a debruijn index of 0 and index 0
/// * the variable `'d` have a debruijn index of 0 and index 1
///
/// The variables `'a` and `'b` both have debruijn index of 1 because,
/// counting out, they are the 2nd binder enclosing `...`. The indices
/// identify the location *within* that binder.
///
/// The variables `'c` and `'d` both have debruijn index of 0 because
/// they appear in the *innermost* binder enclosing the `...`. The
/// indices identify the location *within* that binder.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundVar {
    pub debruijn: DebruijnIndex,
    pub index: usize,
}

impl BoundVar {
    pub fn new(debruijn: DebruijnIndex, index: usize) -> Self {
        Self { debruijn, index }
    }

    pub fn to_ty<I: Interner>(self, interner: &I) -> Ty<I> {
        TyData::<I>::BoundVar(self).intern(interner)
    }

    pub fn to_lifetime<I: Interner>(self, interner: &I) -> Lifetime<I> {
        LifetimeData::<I>::BoundVar(self).intern(interner)
    }

    /// True if this variable is bound within the `amount` innermost binders.
    pub fn bound_within(self, outer_binder: DebruijnIndex) -> bool {
        self.debruijn.within(outer_binder)
    }

    /// Adjusts the debruijn index (see [`DebruijnIndex::shifted_in`]).
    #[must_use]
    pub fn shifted_in(self) -> Self {
        BoundVar::new(self.debruijn.shifted_in(), self.index)
    }

    /// Adjusts the debruijn index (see [`DebruijnIndex::shifted_in`]).
    #[must_use]
    pub fn shifted_in_from(self, outer_binder: DebruijnIndex) -> Self {
        BoundVar::new(self.debruijn.shifted_in_from(outer_binder), self.index)
    }

    /// Adjusts the debruijn index (see [`DebruijnIndex::shifted_in`]).
    #[must_use]
    pub fn shifted_out(self) -> Option<Self> {
        self.debruijn
            .shifted_out()
            .map(|db| BoundVar::new(db, self.index))
    }

    /// Adjusts the debruijn index (see [`DebruijnIndex::shifted_in`]).
    #[must_use]
    pub fn shifted_out_to(self, outer_binder: DebruijnIndex) -> Option<Self> {
        self.debruijn
            .shifted_out_to(outer_binder)
            .map(|db| BoundVar::new(db, self.index))
    }

    /// Return the index of the bound variable, but only if it is bound
    /// at the innermost binder. Otherwise, returns `None`.
    pub fn index_if_innermost(self) -> Option<usize> {
        self.index_if_bound_at(DebruijnIndex::INNERMOST)
    }

    /// Return the index of the bound variable, but only if it is bound
    /// at the innermost binder. Otherwise, returns `None`.
    pub fn index_if_bound_at(self, debruijn: DebruijnIndex) -> Option<usize> {
        if self.debruijn == debruijn {
            Some(self.index)
        } else {
            None
        }
    }
}

/// References the binder at the given depth. The index is a [de
/// Bruijn index], so it counts back through the in-scope binders,
/// with 0 being the innermost binder. This is used in impls and
/// the like. For example, if we had a rule like `for<T> { (T:
/// Clone) :- (T: Copy) }`, then `T` would be represented as a
/// `BoundVar(0)` (as the `for` is the innermost binder).
///
/// [de Bruijn index]: https://en.wikipedia.org/wiki/De_Bruijn_index
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DebruijnIndex {
    depth: u32,
}

impl DebruijnIndex {
    pub const INNERMOST: DebruijnIndex = DebruijnIndex { depth: 0 };
    pub const ONE: DebruijnIndex = DebruijnIndex { depth: 1 };

    pub fn new(depth: u32) -> Self {
        DebruijnIndex { depth }
    }

    pub fn depth(self) -> u32 {
        self.depth
    }

    /// True if the binder identified by this index is within the
    /// binder identified by the index `outer_binder`.
    ///
    /// # Example
    ///
    /// Imagine you have the following binders in scope
    ///
    /// ```ignore
    /// forall<a> forall<b> forall<c>
    /// ```
    ///
    /// then the Debruijn index for `c` would be `0`, the index for
    /// `b` would be 1, and so on. Now consider the following calls:
    ///
    /// * `c.within(a) = true`
    /// * `b.within(a) = true`
    /// * `a.within(a) = false`
    /// * `a.within(c) = false`
    pub fn within(self, outer_binder: DebruijnIndex) -> bool {
        self < outer_binder
    }

    /// Returns the resulting index when this value is moved into
    /// through one binder.
    #[must_use]
    pub fn shifted_in(self) -> DebruijnIndex {
        self.shifted_in_from(DebruijnIndex::ONE)
    }

    /// Update this index in place by shifting it "in" through
    /// `amount` number of binders.
    pub fn shift_in(&mut self) {
        *self = self.shifted_in();
    }

    /// Adds `outer_binder` levels to the `self` index. Intuitively, this
    /// shifts the `self` index, which was valid at the outer binder,
    /// so that it is valid at the innermost binder.
    ///
    /// Example: Assume that the following binders are in scope:
    ///
    /// ```ignore
    /// for<A> for<B> for<C> for<D>
    ///            ^ outer binder
    /// ```
    ///
    /// Assume further that the `outer_binder` argument is 2,
    /// which means that it is referring to the `for<B>` binder
    /// (since `D` would be the innermost binder).
    ///
    /// This means that `self` is relative to the binder `B` -- so
    /// if `self` is 0 (`INNERMOST`), then it refers to `B`,
    /// and if `self` is 1, then it refers to `A`.
    ///
    /// We will return as follows:
    ///
    /// * `0.shifted_in_from(2) = 2` -- i.e., `B`, when shifted in to the binding level `D`, has index 2
    /// * `1.shifted_in_from(2) = 3` -- i.e., `A`, when shifted in to the binding level `D`, has index 3
    /// * `2.shifted_in_from(1) = 3` -- here, we changed the `outer_binder`  to refer to `C`.
    ///   Therefore `2` (relative to `C`) refers to `A`, so the result is still 3 (since `A`, relative to the
    ///   innermost binder, has index 3).
    #[must_use]
    pub fn shifted_in_from(self, outer_binder: DebruijnIndex) -> DebruijnIndex {
        DebruijnIndex::new(self.depth() + outer_binder.depth())
    }

    /// Returns the resulting index when this value is moved out from
    /// `amount` number of new binders.
    #[must_use]
    pub fn shifted_out(self) -> Option<DebruijnIndex> {
        self.shifted_out_to(DebruijnIndex::ONE)
    }

    /// Update in place by shifting out from `amount` binders.
    pub fn shift_out(&mut self) {
        *self = self.shifted_out().unwrap();
    }

    /// Subtracts `outer_binder` levels from the `self` index. Intuitively, this
    /// shifts the `self` index, which was valid at the innermost
    /// binder, to one that is valid at the binder `outer_binder`.
    ///
    /// This will return `None` if the `self` index is internal to the
    /// outer binder (i.e., if `self < outer_binder`).
    ///
    /// Example: Assume that the following binders are in scope:
    ///
    /// ```ignore
    /// for<A> for<B> for<C> for<D>
    ///            ^ outer binder
    /// ```
    ///
    /// Assume further that the `outer_binder` argument is 2,
    /// which means that it is referring to the `for<B>` binder
    /// (since `D` would be the innermost binder).
    ///
    /// This means that the result is relative to the binder `B` -- so
    /// if `self` is 0 (`INNERMOST`), then it refers to `B`,
    /// and if `self` is 1, then it refers to `A`.
    ///
    /// We will return as follows:
    ///
    /// * `1.shifted_out_to(2) = None` -- i.e., the binder for `C` can't be named from the binding level `B`
    /// * `3.shifted_out_to(2) = Some(1)` -- i.e., `A`, when shifted out to the binding level `B`, has index 1
    pub fn shifted_out_to(self, outer_binder: DebruijnIndex) -> Option<DebruijnIndex> {
        if self.within(outer_binder) {
            None
        } else {
            Some(DebruijnIndex::new(self.depth() - outer_binder.depth()))
        }
    }
}

/// A "DynTy" could be either a `dyn Trait` or an (opaque) `impl
/// Trait`. Both of them are conceptually very related to a
/// "existential type" of the form `exists<T> { T: Trait }`. The
/// `DynTy` type represents those bounds.
///
/// The "binder" here represents the unknown self type. So, a type like
/// `impl for<'a> Fn(&'a u32)` would be represented with two-levels of
/// binder, as "depicted" here:
///
/// ```notrust
/// exists<type> {
///    vec![
///        // A QuantifiedWhereClause:
///        forall<region> { ^1: Fn(&^0 u32) }
///    ]
/// }
/// ```
///
/// The outer `exists<type>` binder indicates that there exists
/// some type that meets the criteria within, but that type is not
/// known. It is referenced within the type using `^1`, indicating
/// a bound type with debruijn index 1 (i.e., skipping through one
/// level of binder).
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct DynTy<I: Interner> {
    pub bounds: Binders<QuantifiedWhereClauses<I>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InferenceVar {
    index: u32,
}

impl From<u32> for InferenceVar {
    fn from(index: u32) -> InferenceVar {
        InferenceVar { index }
    }
}

impl InferenceVar {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn to_ty<I: Interner>(self, interner: &I) -> Ty<I> {
        TyData::<I>::InferenceVar(self).intern(interner)
    }

    pub fn to_lifetime<I: Interner>(self, interner: &I) -> Lifetime<I> {
        LifetimeData::<I>::InferenceVar(self).intern(interner)
    }
}

/// for<'a...'z> X -- all binders are instantiated at once,
/// and we use deBruijn indices within `self.ty`
#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
pub struct Fn<I: Interner> {
    pub num_binders: usize,
    pub substitution: Substitution<I>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Lifetime<I: Interner> {
    interned: I::InternedLifetime,
}

impl<I: Interner> Lifetime<I> {
    pub fn new(interner: &I, data: impl CastTo<LifetimeData<I>>) -> Self {
        Lifetime {
            interned: I::intern_lifetime(interner, data.cast(interner)),
        }
    }

    pub fn interned(&self) -> &I::InternedLifetime {
        &self.interned
    }

    pub fn data(&self, interner: &I) -> &LifetimeData<I> {
        I::lifetime_data(interner, &self.interned)
    }

    /// If this is a `Lifetime::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self, interner: &I) -> Option<InferenceVar> {
        if let LifetimeData::InferenceVar(depth) = self.data(interner) {
            Some(*depth)
        } else {
            None
        }
    }

    /// True if this lifetime is a "bound" lifetime, and hence
    /// needs to be shifted across binders. Meant for debug assertions.
    pub fn needs_shift(&self, interner: &I) -> bool {
        match self.data(interner) {
            LifetimeData::BoundVar(_) => true,
            LifetimeData::InferenceVar(_) => false,
            LifetimeData::Placeholder(_) => false,
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub enum LifetimeData<I: Interner> {
    /// See TyData::Var(_).
    BoundVar(BoundVar),
    InferenceVar(InferenceVar),
    Placeholder(PlaceholderIndex),
    Phantom(Void, PhantomData<I>),
}

impl<I: Interner> LifetimeData<I> {
    pub fn intern(self, interner: &I) -> Lifetime<I> {
        Lifetime::new(interner, self)
    }
}

/// Index of an universally quantified parameter in the environment.
/// Two indexes are required, the one of the universe itself
/// and the relative index inside the universe.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaceholderIndex {
    /// Index *of* the universe.
    pub ui: UniverseIndex,
    /// Index *in* the universe.
    pub idx: usize,
}

impl PlaceholderIndex {
    pub fn to_lifetime<I: Interner>(self, interner: &I) -> Lifetime<I> {
        LifetimeData::<I>::Placeholder(self).intern(interner)
    }

    pub fn to_ty<I: Interner>(self, interner: &I) -> Ty<I> {
        let data: TyData<I> = TyData::Placeholder(self);
        data.intern(interner)
    }
}

// Fold derive intentionally omitted, folded through Ty
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct ApplicationTy<I: Interner> {
    pub name: TypeName<I>,
    pub substitution: Substitution<I>,
}

impl<I: Interner> ApplicationTy<I> {
    pub fn intern(self, interner: &I) -> Ty<I> {
        Ty::new(interner, self)
    }

    pub fn type_parameters<'a>(&'a self, interner: &'a I) -> impl Iterator<Item = Ty<I>> + 'a {
        self.substitution
            .iter(interner)
            .filter_map(move |p| p.ty(interner))
            .cloned()
    }

    pub fn first_type_parameter(&self, interner: &I) -> Option<Ty<I>> {
        self.type_parameters(interner).next()
    }

    pub fn len_type_parameters(&self, interner: &I) -> usize {
        self.type_parameters(interner).count()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParameterKind<T, L = T> {
    Ty(T),
    Lifetime(L),
}

impl<T> ParameterKind<T> {
    pub fn into_inner(self) -> T {
        match self {
            ParameterKind::Ty(t) => t,
            ParameterKind::Lifetime(t) => t,
        }
    }

    pub fn map<OP, U>(self, op: OP) -> ParameterKind<U>
    where
        OP: FnOnce(T) -> U,
    {
        match self {
            ParameterKind::Ty(t) => ParameterKind::Ty(op(t)),
            ParameterKind::Lifetime(t) => ParameterKind::Lifetime(op(t)),
        }
    }

    pub fn map_ref<OP, U>(&self, op: OP) -> ParameterKind<U>
    where
        OP: FnOnce(&T) -> U,
    {
        match self {
            ParameterKind::Ty(t) => ParameterKind::Ty(op(t)),
            ParameterKind::Lifetime(t) => ParameterKind::Lifetime(op(t)),
        }
    }
}

impl<T, L> ParameterKind<T, L> {
    pub fn assert_ty_ref(&self) -> &T {
        self.as_ref().ty().unwrap()
    }

    pub fn assert_lifetime_ref(&self) -> &L {
        self.as_ref().lifetime().unwrap()
    }

    pub fn as_ref(&self) -> ParameterKind<&T, &L> {
        match *self {
            ParameterKind::Ty(ref t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(ref l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn is_ty(&self) -> bool {
        match self {
            ParameterKind::Ty(_) => true,
            ParameterKind::Lifetime(_) => false,
        }
    }

    pub fn ty(self) -> Option<T> {
        match self {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    pub fn lifetime(self) -> Option<L> {
        match self {
            ParameterKind::Lifetime(t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Parameter<I: Interner> {
    interned: I::InternedParameter,
}

impl<I: Interner> Parameter<I> {
    pub fn new(interner: &I, data: ParameterData<I>) -> Self {
        let interned = I::intern_parameter(interner, data);
        Parameter { interned }
    }

    pub fn interned(&self) -> &I::InternedParameter {
        &self.interned
    }

    pub fn data(&self, interner: &I) -> &ParameterData<I> {
        I::parameter_data(interner, &self.interned)
    }

    pub fn assert_ty_ref(&self, interner: &I) -> &Ty<I> {
        self.as_ref(interner).ty().unwrap()
    }

    pub fn assert_lifetime_ref(&self, interner: &I) -> &Lifetime<I> {
        self.as_ref(interner).lifetime().unwrap()
    }

    pub fn as_ref(&self, interner: &I) -> ParameterKind<&Ty<I>, &Lifetime<I>> {
        match self.data(interner) {
            ParameterKind::Ty(t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn is_ty(&self, interner: &I) -> bool {
        match self.data(interner) {
            ParameterKind::Ty(_) => true,
            ParameterKind::Lifetime(_) => false,
        }
    }

    pub fn ty(&self, interner: &I) -> Option<&Ty<I>> {
        match self.data(interner) {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    pub fn lifetime(&self, interner: &I) -> Option<&Lifetime<I>> {
        match self.data(interner) {
            ParameterKind::Lifetime(t) => Some(t),
            _ => None,
        }
    }
}

#[allow(type_alias_bounds)]
pub type ParameterData<I: Interner> = ParameterKind<Ty<I>, Lifetime<I>>;

impl<I: Interner> ParameterData<I> {
    pub fn intern(self, interner: &I) -> Parameter<I> {
        Parameter::new(interner, self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub enum AliasTy<I: Interner> {
    Projection(ProjectionTy<I>),
    Opaque(OpaqueTy<I>),
}

impl<I: Interner> AliasTy<I> {
    pub fn intern(self, interner: &I) -> Ty<I> {
        Ty::new(interner, self)
    }

    pub fn self_type_parameter(&self, interner: &I) -> Ty<I> {
        match self {
            AliasTy::Projection(projection_ty) => projection_ty
                .substitution
                .iter(interner)
                .find_map(move |p| p.ty(interner))
                .unwrap()
                .clone(),
            _ => todo!(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct ProjectionTy<I: Interner> {
    pub associated_ty_id: AssocTypeId<I>,
    pub substitution: Substitution<I>,
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct OpaqueTy<I: Interner> {
    pub opaque_ty_id: OpaqueTyId<I>,
    pub substitution: Substitution<I>,
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct TraitRef<I: Interner> {
    pub trait_id: TraitId<I>,
    pub substitution: Substitution<I>,
}

impl<I: Interner> TraitRef<I> {
    pub fn type_parameters<'a>(&'a self, interner: &'a I) -> impl Iterator<Item = Ty<I>> + 'a {
        self.substitution
            .iter(interner)
            .filter_map(move |p| p.ty(interner))
            .cloned()
    }

    pub fn self_type_parameter(&self, interner: &I) -> Ty<I> {
        self.type_parameters(interner).next().unwrap()
    }

    pub fn from_env(self) -> FromEnv<I> {
        FromEnv::Trait(self)
    }

    pub fn well_formed(self) -> WellFormed<I> {
        WellFormed::Trait(self)
    }
}

/// Where clauses that can be written by a Rust programmer.
#[derive(Clone, PartialEq, Eq, Hash, Fold, SuperVisit, HasInterner, Zip)]
pub enum WhereClause<I: Interner> {
    Implemented(TraitRef<I>),
    AliasEq(AliasEq<I>),
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub enum WellFormed<I: Interner> {
    /// A predicate which is true is some trait ref is well-formed.
    /// For example, given the following trait definitions:
    ///
    /// ```notrust
    /// trait Clone { ... }
    /// trait Copy where Self: Clone { ... }
    /// ```
    ///
    /// then we have the following rule:
    ///
    /// ```notrust
    /// WellFormed(?Self: Copy) :- ?Self: Copy, WellFormed(?Self: Clone)
    /// ```
    Trait(TraitRef<I>),

    /// A predicate which is true is some type is well-formed.
    /// For example, given the following type definition:
    ///
    /// ```notrust
    /// struct Set<K> where K: Hash {
    ///     ...
    /// }
    /// ```
    ///
    /// then we have the following rule: `WellFormedTy(Set<K>) :- Implemented(K: Hash)`.
    Ty(Ty<I>),
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub enum FromEnv<I: Interner> {
    /// A predicate which enables deriving everything which should be true if we *know* that
    /// some trait ref is well-formed. For example given the above trait definitions, we can use
    /// `FromEnv(T: Copy)` to derive that `T: Clone`, like in:
    ///
    /// ```notrust
    /// forall<T> {
    ///     if (FromEnv(T: Copy)) {
    ///         T: Clone
    ///     }
    /// }
    /// ```
    Trait(TraitRef<I>),

    /// A predicate which enables deriving everything which should be true if we *know* that
    /// some type is well-formed. For example given the above type definition, we can use
    /// `FromEnv(Set<K>)` to derive that `K: Hash`, like in:
    ///
    /// ```notrust
    /// forall<K> {
    ///     if (FromEnv(Set<K>)) {
    ///         K: Hash
    ///     }
    /// }
    /// ```
    Ty(Ty<I>),
}

/// A "domain goal" is a goal that is directly about Rust, rather than a pure
/// logical statement. As much as possible, the Chalk solver should avoid
/// decomposing this enum, and instead treat its values opaquely.
#[derive(Clone, PartialEq, Eq, Hash, Fold, SuperVisit, HasInterner, Zip)]
pub enum DomainGoal<I: Interner> {
    Holds(WhereClause<I>),

    WellFormed(WellFormed<I>),

    FromEnv(FromEnv<I>),

    Normalize(Normalize<I>),

    /// True if a type is considered to have been "defined" by the current crate. This is true for
    /// a `struct Foo { }` but false for a `#[upstream] struct Foo { }`. However, for fundamental types
    /// like `Box<T>`, it is true if `T` is local.
    IsLocal(Ty<I>),

    /// True if a type is *not* considered to have been "defined" by the current crate. This is
    /// false for a `struct Foo { }` but true for a `#[upstream] struct Foo { }`. However, for
    /// fundamental types like `Box<T>`, it is true if `T` is upstream.
    IsUpstream(Ty<I>),

    /// True if a type and its input types are fully visible, known types. That is, there are no
    /// unknown type parameters anywhere in this type.
    ///
    /// More formally, for each struct S<P0..Pn>:
    /// forall<P0..Pn> {
    ///     IsFullyVisible(S<P0...Pn>) :-
    ///         IsFullyVisible(P0),
    ///         ...
    ///         IsFullyVisible(Pn)
    /// }
    ///
    /// Note that any of these types can have lifetimes in their parameters too, but we only
    /// consider type parameters.
    IsFullyVisible(Ty<I>),

    /// Used to dictate when trait impls are allowed in the current (local) crate based on the
    /// orphan rules.
    ///
    /// `LocalImplAllowed(T: Trait)` is true if the type T is allowed to impl trait Trait in
    /// the current crate. Under the current rules, this is unconditionally true for all types if
    /// the Trait is considered to be "defined" in the current crate. If that is not the case, then
    /// `LocalImplAllowed(T: Trait)` can still be true if `IsLocal(T)` is true.
    LocalImplAllowed(TraitRef<I>),

    /// Used to activate the "compatible modality" rules. Rules that introduce predicates that have
    /// to do with "all compatible universes" should depend on this clause so that they only apply
    /// if this is present.
    ///
    /// (HACK: Having `()` makes some of our macros work better.)
    Compatible(()),

    /// Used to indicate that a given type is in a downstream crate. Downstream crates contain the
    /// current crate at some level of their dependencies.
    ///
    /// Since chalk does not actually see downstream types, this is usually introduced with
    /// implication on a fresh, universally quantified type.
    ///
    /// forall<T> { if (DownstreamType(T)) { /* ... */ } }
    ///
    /// This makes a new type `T` available and makes `DownstreamType(T)` provable for that type.
    DownstreamType(Ty<I>),

    /// Used to activate the "reveal mode", in which opaque (`impl Trait`) types can be equated
    /// to their actual type.
    Reveal(()),
}

pub type QuantifiedWhereClause<I> = Binders<WhereClause<I>>;

impl<I: Interner> WhereClause<I> {
    /// Turn a where clause into the WF version of it i.e.:
    /// * `Implemented(T: Trait)` maps to `WellFormed(T: Trait)`
    /// * `ProjectionEq(<T as Trait>::Item = Foo)` maps to `WellFormed(<T as Trait>::Item = Foo)`
    /// * any other clause maps to itself
    pub fn into_well_formed_goal(self, interner: &I) -> DomainGoal<I> {
        match self {
            WhereClause::Implemented(trait_ref) => WellFormed::Trait(trait_ref).cast(interner),
            wc => wc.cast(interner),
        }
    }

    /// Same as `into_well_formed_goal` but with the `FromEnv` predicate instead of `WellFormed`.
    pub fn into_from_env_goal(self, interner: &I) -> DomainGoal<I> {
        match self {
            WhereClause::Implemented(trait_ref) => FromEnv::Trait(trait_ref).cast(interner),
            wc => wc.cast(interner),
        }
    }
}

impl<I: Interner> QuantifiedWhereClause<I> {
    /// As with `WhereClause::into_well_formed_goal`, but for a
    /// quantified where clause. For example, `forall<T> {
    /// Implemented(T: Trait)}` would map to `forall<T> {
    /// WellFormed(T: Trait) }`.
    pub fn into_well_formed_goal(self, interner: &I) -> Binders<DomainGoal<I>> {
        self.map(|wc| wc.into_well_formed_goal(interner))
    }

    /// As with `WhereClause::into_from_env_goal`, but mapped over any
    /// binders. For example, `forall<T> {
    /// Implemented(T: Trait)}` would map to `forall<T> {
    /// FromEnv(T: Trait) }`.
    pub fn into_from_env_goal(self, interner: &I) -> Binders<DomainGoal<I>> {
        self.map(|wc| wc.into_from_env_goal(interner))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct QuantifiedWhereClauses<I: Interner> {
    interned: I::InternedQuantifiedWhereClauses,
}

impl<I: Interner> QuantifiedWhereClauses<I> {
    pub fn new(interner: &I) -> Self {
        Self::from(interner, None::<QuantifiedWhereClause<I>>)
    }

    pub fn interned(&self) -> &I::InternedQuantifiedWhereClauses {
        &self.interned
    }

    pub fn from(
        interner: &I,
        clauses: impl IntoIterator<Item = impl CastTo<QuantifiedWhereClause<I>>>,
    ) -> Self {
        Self::from_fallible(
            interner,
            clauses
                .into_iter()
                .map(|p| -> Result<QuantifiedWhereClause<I>, ()> { Ok(p.cast(interner)) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        clauses: impl IntoIterator<Item = Result<impl CastTo<QuantifiedWhereClause<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        Ok(QuantifiedWhereClauses {
            interned: I::intern_quantified_where_clauses(
                interner,
                clauses.into_iter().casted(interner),
            )?,
        })
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, QuantifiedWhereClause<I>> {
        self.as_slice(interner).iter()
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.as_slice(interner).is_empty()
    }

    pub fn len(&self, interner: &I) -> usize {
        self.as_slice(interner).len()
    }

    pub fn as_slice(&self, interner: &I) -> &[QuantifiedWhereClause<I>] {
        interner.quantified_where_clauses_data(&self.interned)
    }
}

impl<I: Interner> DomainGoal<I> {
    /// Convert `Implemented(...)` into `FromEnv(...)`, but leave other
    /// goals unchanged.
    pub fn into_from_env_goal(self, interner: &I) -> DomainGoal<I> {
        match self {
            DomainGoal::Holds(wc) => wc.into_from_env_goal(interner),
            goal => goal,
        }
    }

    pub fn inputs(&self, interner: &I) -> Vec<Parameter<I>> {
        match self {
            DomainGoal::Holds(WhereClause::AliasEq(alias_eq)) => {
                vec![ParameterKind::Ty(alias_eq.alias.clone().intern(interner)).intern(interner)]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, Zip)]
pub struct EqGoal<I: Interner> {
    pub a: Parameter<I>,
    pub b: Parameter<I>,
}

/// Proves that the given type alias **normalizes** to the given
/// type. A projection `T::Foo` normalizes to the type `U` if we can
/// **match it to an impl** and that impl has a `type Foo = V` where
/// `U = V`.
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, Zip)]
pub struct Normalize<I: Interner> {
    pub alias: AliasTy<I>,
    pub ty: Ty<I>,
}

/// Proves **equality** between an alias and a type.
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, Zip)]
pub struct AliasEq<I: Interner> {
    pub alias: AliasTy<I>,
    pub ty: Ty<I>,
}

impl<I: Interner> HasInterner for AliasEq<I> {
    type Interner = I;
}

/// Indicates that the `value` is universally quantified over `N`
/// parameters of the given kinds, where `N == self.binders.len()`. A
/// variable with depth `i < N` refers to the value at
/// `self.binders[i]`. Variables with depth `>= N` are free.
///
/// (IOW, we use deBruijn indices, where binders are introduced in reverse order
/// of `self.binders`.)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Binders<T: HasInterner> {
    pub binders: ParameterKinds<T::Interner>,
    value: T,
}

impl<T: HasInterner> HasInterner for Binders<T> {
    type Interner = T::Interner;
}

impl<T: HasInterner> Binders<T> {
    pub fn new(binders: ParameterKinds<T::Interner>, value: T) -> Self {
        Self { binders, value }
    }

    /// Skips the binder and returns the "bound" value. This is a
    /// risky thing to do because it's easy to get confused about
    /// De Bruijn indices and the like. `skip_binder` is only valid
    /// when you are either extracting data that has nothing to
    /// do with bound vars, or you are being very careful about
    /// your depth accounting.
    ///
    /// Some examples where `skip_binder` is reasonable:
    ///
    /// - extracting the `TraitId` from a TraitRef;
    /// - checking if there are any fields in a StructDatum
    pub fn skip_binders(&self) -> &T {
        &self.value
    }

    /// Converts `&Binders<T>` to `Binders<&T>`. Produces new `Binders`
    /// with cloned quantifiers containing a reference to the original
    /// value, leaving the original in place.
    pub fn as_ref(&self) -> Binders<&T> {
        Binders {
            binders: self.binders.clone(),
            value: &self.value,
        }
    }

    pub fn map<U, OP>(self, op: OP) -> Binders<U>
    where
        OP: FnOnce(T) -> U,
        U: HasInterner<Interner = T::Interner>,
    {
        let value = op(self.value);
        Binders {
            binders: self.binders,
            value,
        }
    }

    pub fn map_ref<'a, U, OP>(&'a self, op: OP) -> Binders<U>
    where
        OP: FnOnce(&'a T) -> U,
        U: HasInterner<Interner = T::Interner>,
    {
        self.as_ref().map(op)
    }

    /// Creates a fresh binders that contains a single type
    /// variable. The result of the closure will be embedded in this
    /// binder. Note that you should be careful with what you return
    /// from the closure to account for the binder that will be added.
    ///
    /// XXX FIXME -- this is potentially a pretty footgun-y function.
    pub fn with_fresh_type_var(
        interner: &T::Interner,
        op: impl FnOnce(Ty<T::Interner>) -> T,
    ) -> Binders<T> {
        // The new variable is at the front and everything afterwards is shifted up by 1
        let new_var = TyData::BoundVar(BoundVar::new(DebruijnIndex::INNERMOST, 0)).intern(interner);
        let value = op(new_var);
        let binders = ParameterKinds::from(interner, iter::once(ParameterKind::Ty(())));
        Binders { binders, value }
    }

    pub fn len(&self, interner: &T::Interner) -> usize {
        self.binders.len(interner)
    }
}

impl<T: HasInterner> From<Binders<T>> for (ParameterKinds<T::Interner>, T) {
    fn from(binders: Binders<T>) -> Self {
        (binders.binders, binders.value)
    }
}

impl<T, I> Binders<T>
where
    T: Fold<I, I> + HasInterner<Interner = I>,
    I: Interner,
{
    /// Substitute `parameters` for the variables introduced by these
    /// binders. So if the binders represent (e.g.) `<X, Y> { T }` and
    /// parameters is the slice `[A, B]`, then returns `[X => A, Y =>
    /// B] T`.
    pub fn substitute(
        &self,
        interner: &I,
        parameters: &(impl AsParameters<I> + ?Sized),
    ) -> T::Result {
        let parameters = parameters.as_parameters(interner);
        assert_eq!(self.binders.len(interner), parameters.len());
        Subst::apply(interner, parameters, &self.value)
    }
}

/// Allows iterating over a `&Binders<Vec<T>>`, for instance. Each
/// element will be a `Binders<&T>`.
impl<'a, V> IntoIterator for &'a Binders<V>
where
    V: HasInterner,
    &'a V: IntoIterator,
    <&'a V as IntoIterator>::Item: HasInterner<Interner = V::Interner>,
{
    type Item = Binders<<&'a V as IntoIterator>::Item>;
    type IntoIter = BindersIntoIterator<&'a V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map_ref(|r| r).into_iter()
    }
}

/// Allows iterating over a Binders<Vec<T>>, for instance.
/// Each element will include the same set of parameter bounds.
impl<V, U> IntoIterator for Binders<V>
where
    V: HasInterner + IntoIterator<Item = U>,
    U: HasInterner<Interner = V::Interner>,
{
    type Item = Binders<U>;
    type IntoIter = BindersIntoIterator<V>;

    fn into_iter(self) -> Self::IntoIter {
        BindersIntoIterator {
            iter: self.value.into_iter(),
            binders: self.binders,
        }
    }
}

pub struct BindersIntoIterator<V: HasInterner + IntoIterator> {
    iter: <V as IntoIterator>::IntoIter,
    binders: ParameterKinds<V::Interner>,
}

impl<V> Iterator for BindersIntoIterator<V>
where
    V: HasInterner + IntoIterator,
    <V as IntoIterator>::Item: HasInterner<Interner = V::Interner>,
{
    type Item = Binders<<V as IntoIterator>::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|v| Binders::new(self.binders.clone(), v))
    }
}

/// Represents one clause of the form `consequence :- conditions` where
/// `conditions = cond_1 && cond_2 && ...` is the conjunction of the individual
/// conditions.
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
pub struct ProgramClauseImplication<I: Interner> {
    pub consequence: DomainGoal<I>,
    pub conditions: Goals<I>,
    pub priority: ClausePriority,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClausePriority {
    High,
    Low,
}

impl std::ops::BitAnd for ClausePriority {
    type Output = ClausePriority;
    fn bitand(self, rhs: ClausePriority) -> Self::Output {
        match (self, rhs) {
            (ClausePriority::High, ClausePriority::High) => ClausePriority::High,
            _ => ClausePriority::Low,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner, Zip)]
pub enum ProgramClauseData<I: Interner> {
    Implies(ProgramClauseImplication<I>),
    ForAll(Binders<ProgramClauseImplication<I>>),
}

impl<I: Interner> ProgramClauseImplication<I> {
    pub fn into_from_env_clause(self, interner: &I) -> ProgramClauseImplication<I> {
        if self.conditions.is_empty(interner) {
            ProgramClauseImplication {
                consequence: self.consequence.into_from_env_goal(interner),
                conditions: self.conditions.clone(),
                priority: self.priority,
            }
        } else {
            self
        }
    }
}

impl<I: Interner> ProgramClauseData<I> {
    pub fn into_from_env_clause(self, interner: &I) -> ProgramClauseData<I> {
        match self {
            ProgramClauseData::Implies(implication) => {
                ProgramClauseData::Implies(implication.into_from_env_clause(interner))
            }
            ProgramClauseData::ForAll(binders_implication) => ProgramClauseData::ForAll(
                binders_implication.map(|i| i.into_from_env_clause(interner)),
            ),
        }
    }

    pub fn intern(self, interner: &I) -> ProgramClause<I> {
        ProgramClause {
            interned: interner.intern_program_clause(self),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct ProgramClause<I: Interner> {
    interned: I::InternedProgramClause,
}

impl<I: Interner> ProgramClause<I> {
    pub fn new(interner: &I, clause: ProgramClauseData<I>) -> Self {
        let interned = interner.intern_program_clause(clause);
        Self { interned }
    }

    pub fn into_from_env_clause(self, interner: &I) -> ProgramClause<I> {
        let program_clause_data = self.data(interner);
        let new_clause = program_clause_data.clone().into_from_env_clause(interner);
        Self::new(interner, new_clause)
    }

    pub fn interned(&self) -> &I::InternedProgramClause {
        &self.interned
    }

    pub fn data(&self, interner: &I) -> &ProgramClauseData<I> {
        interner.program_clause_data(&self.interned)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct ProgramClauses<I: Interner> {
    interned: I::InternedProgramClauses,
}

impl<I: Interner> ProgramClauses<I> {
    pub fn new(interner: &I) -> Self {
        Self::from(interner, None::<ProgramClause<I>>)
    }

    pub fn interned(&self) -> &I::InternedProgramClauses {
        &self.interned
    }

    pub fn from(
        interner: &I,
        clauses: impl IntoIterator<Item = impl CastTo<ProgramClause<I>>>,
    ) -> Self {
        Self::from_fallible(
            interner,
            clauses
                .into_iter()
                .map(|p| -> Result<ProgramClause<I>, ()> { Ok(p.cast(interner)) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        clauses: impl IntoIterator<Item = Result<impl CastTo<ProgramClause<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        Ok(ProgramClauses {
            interned: I::intern_program_clauses(interner, clauses.into_iter().casted(interner))?,
        })
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, ProgramClause<I>> {
        self.as_slice(interner).iter()
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.as_slice(interner).is_empty()
    }

    pub fn len(&self, interner: &I) -> usize {
        self.as_slice(interner).len()
    }

    pub fn as_slice(&self, interner: &I) -> &[ProgramClause<I>] {
        interner.program_clauses_data(&self.interned)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct ParameterKinds<I: Interner> {
    interned: I::InternedParameterKinds,
}

impl<I: Interner> ParameterKinds<I> {
    pub fn new(interner: &I) -> Self {
        Self::from(interner, None::<ParameterKind<()>>)
    }

    pub fn interned(&self) -> &I::InternedParameterKinds {
        &self.interned
    }

    pub fn from(
        interner: &I,
        parameter_kinds: impl IntoIterator<Item = ParameterKind<()>>,
    ) -> Self {
        Self::from_fallible(
            interner,
            parameter_kinds
                .into_iter()
                .map(|p| -> Result<ParameterKind<()>, ()> { Ok(p) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        parameter_kinds: impl IntoIterator<Item = Result<ParameterKind<()>, E>>,
    ) -> Result<Self, E> {
        Ok(ParameterKinds {
            interned: I::intern_parameter_kinds(interner, parameter_kinds.into_iter())?,
        })
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, ParameterKind<()>> {
        self.as_slice(interner).iter()
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.as_slice(interner).is_empty()
    }

    pub fn len(&self, interner: &I) -> usize {
        self.as_slice(interner).len()
    }

    pub fn as_slice(&self, interner: &I) -> &[ParameterKind<()>] {
        interner.parameter_kinds_data(&self.interned)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct CanonicalVarKinds<I: Interner> {
    interned: I::InternedCanonicalVarKinds,
}

impl<I: Interner> CanonicalVarKinds<I> {
    pub fn new(interner: &I) -> Self {
        Self::from(interner, None::<ParameterKind<UniverseIndex>>)
    }

    pub fn interned(&self) -> &I::InternedCanonicalVarKinds {
        &self.interned
    }

    pub fn from(
        interner: &I,
        parameter_kinds: impl IntoIterator<Item = ParameterKind<UniverseIndex>>,
    ) -> Self {
        Self::from_fallible(
            interner,
            parameter_kinds
                .into_iter()
                .map(|p| -> Result<ParameterKind<UniverseIndex>, ()> { Ok(p) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        parameter_kinds: impl IntoIterator<Item = Result<ParameterKind<UniverseIndex>, E>>,
    ) -> Result<Self, E> {
        Ok(CanonicalVarKinds {
            interned: I::intern_canonical_var_kinds(interner, parameter_kinds.into_iter())?,
        })
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, ParameterKind<UniverseIndex>> {
        self.as_slice(interner).iter()
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.as_slice(interner).is_empty()
    }

    pub fn len(&self, interner: &I) -> usize {
        self.as_slice(interner).len()
    }

    pub fn as_slice(&self, interner: &I) -> &[ParameterKind<UniverseIndex>] {
        interner.canonical_var_kinds_data(&self.interned)
    }
}

/// Wraps a "canonicalized item". Items are canonicalized as follows:
///
/// All unresolved existential variables are "renumbered" according to their
/// first appearance; the kind/universe of the variable is recorded in the
/// `binders` field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Canonical<T: HasInterner> {
    pub value: T,
    pub binders: CanonicalVarKinds<T::Interner>,
}

impl<T: HasInterner> HasInterner for Canonical<T> {
    type Interner = T::Interner;
}

/// A "universe canonical" value. This is a wrapper around a
/// `Canonical`, indicating that the universes within have been
/// "renumbered" to start from 0 and collapse unimportant
/// distinctions.
///
/// To produce one of these values, use the `u_canonicalize` method.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UCanonical<T: HasInterner> {
    pub canonical: Canonical<T>,
    pub universes: usize,
}

impl<T: HasInterner> UCanonical<T> {
    pub fn is_trivial_substitution(
        &self,
        interner: &T::Interner,
        canonical_subst: &Canonical<AnswerSubst<T::Interner>>,
    ) -> bool {
        let subst = &canonical_subst.value.subst;
        assert_eq!(
            self.canonical.binders.len(interner),
            subst.parameters(interner).len()
        );
        subst.is_identity_subst(interner)
    }

    pub fn trivial_substitution(&self, interner: &T::Interner) -> Substitution<T::Interner> {
        let binders = &self.canonical.binders;
        Substitution::from(
            interner,
            binders
                .iter(interner)
                .enumerate()
                .map(|(index, pk)| {
                    let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, index);
                    match pk {
                        ParameterKind::Ty(_) => {
                            ParameterKind::Ty(TyData::BoundVar(bound_var).intern(interner))
                                .intern(interner)
                        }
                        ParameterKind::Lifetime(_) => ParameterKind::Lifetime(
                            LifetimeData::BoundVar(bound_var).intern(interner),
                        )
                        .intern(interner),
                    }
                })
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, HasInterner)]
/// A list of goals.
pub struct Goals<I: Interner> {
    interned: I::InternedGoals,
}

impl<I: Interner> Goals<I> {
    pub fn new(interner: &I) -> Self {
        Self::from(interner, None::<Goal<I>>)
    }

    pub fn interned(&self) -> &I::InternedGoals {
        &self.interned
    }

    pub fn from(interner: &I, goals: impl IntoIterator<Item = impl CastTo<Goal<I>>>) -> Self {
        Self::from_fallible(
            interner,
            goals
                .into_iter()
                .map(|p| -> Result<Goal<I>, ()> { Ok(p.cast(interner)) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        goals: impl IntoIterator<Item = Result<impl CastTo<Goal<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        Ok(Goals {
            interned: I::intern_goals(interner, goals.into_iter().casted(interner))?,
        })
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, Goal<I>> {
        self.as_slice(interner).iter()
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.as_slice(interner).is_empty()
    }

    pub fn len(&self, interner: &I) -> usize {
        self.as_slice(interner).len()
    }

    pub fn as_slice(&self, interner: &I) -> &[Goal<I>] {
        interner.goals_data(&self.interned)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub struct Goal<I: Interner> {
    interned: I::InternedGoal,
}

impl<I: Interner> Goal<I> {
    pub fn new(interner: &I, interned: GoalData<I>) -> Self {
        let interned = I::intern_goal(interner, interned);
        Self { interned }
    }

    pub fn interned(&self) -> &I::InternedGoal {
        &self.interned
    }

    pub fn data(&self, interner: &I) -> &GoalData<I> {
        interner.goal_data(&self.interned)
    }

    pub fn quantify(
        self,
        interner: &I,
        kind: QuantifierKind,
        binders: ParameterKinds<I>,
    ) -> Goal<I> {
        GoalData::Quantified(kind, Binders::new(binders, self)).intern(interner)
    }

    /// Takes a goal `G` and turns it into `not { G }`
    pub fn negate(self, interner: &I) -> Self {
        GoalData::Not(self).intern(interner)
    }

    /// Takes a goal `G` and turns it into `compatible { G }`.
    pub fn compatible(self, interner: &I) -> Self {
        // compatible { G } desugars into: forall<T> { if (Compatible, DownstreamType(T)) { G } }
        // This activates the compatible modality rules and introduces an anonymous downstream type
        GoalData::Quantified(
            QuantifierKind::ForAll,
            Binders::with_fresh_type_var(interner, |ty| {
                GoalData::Implies(
                    ProgramClauses::from(
                        interner,
                        vec![DomainGoal::Compatible(()), DomainGoal::DownstreamType(ty)],
                    ),
                    self.shifted_in(interner),
                )
                .intern(interner)
            }),
        )
        .intern(interner)
    }

    pub fn implied_by(self, interner: &I, predicates: ProgramClauses<I>) -> Goal<I> {
        GoalData::Implies(predicates, self).intern(interner)
    }

    /// True if this goal is "trivially true" -- i.e., no work is
    /// required to prove it.
    pub fn is_trivially_true(&self, interner: &I) -> bool {
        match self.data(interner) {
            GoalData::All(goals) => goals.is_empty(interner),
            _ => false,
        }
    }
}

impl<I> Goal<I>
where
    I: Interner,
{
    pub fn all<II>(interner: &I, iter: II) -> Self
    where
        II: IntoIterator<Item = Goal<I>>,
    {
        let mut iter = iter.into_iter();
        if let Some(goal0) = iter.next() {
            if let Some(goal1) = iter.next() {
                // More than one goal to prove
                let goals = Goals::from(
                    interner,
                    Some(goal0).into_iter().chain(Some(goal1)).chain(iter),
                );
                GoalData::All(goals).intern(interner)
            } else {
                // One goal to prove
                goal0
            }
        } else {
            // No goals to prove, always true
            GoalData::All(Goals::new(interner)).intern(interner)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner, Zip)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum GoalData<I: Interner> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Goal<I>>),
    Implies(ProgramClauses<I>, Goal<I>),
    All(Goals<I>),
    Not(Goal<I>),

    /// Make two things equal; the rules for doing so are well known to the logic
    EqGoal(EqGoal<I>),

    /// A "domain goal" indicates some base sort of goal that can be
    /// proven via program clauses
    DomainGoal(DomainGoal<I>),

    /// Indicates something that cannot be proven to be true or false
    /// definitively. This can occur with overflow but also with
    /// unifications of skolemized variables like `forall<X,Y> { X = Y
    /// }`. Of course, that statement is false, as there exist types
    /// X, Y where `X = Y` is not true. But we treat it as "cannot
    /// prove" so that `forall<X,Y> { not { X = Y } }` also winds up
    /// as cannot prove.
    ///
    /// (TOTAL HACK: Having a unit result makes some of our macros work better.)
    CannotProve(()),
}

impl<I: Interner> GoalData<I> {
    pub fn intern(self, interner: &I) -> Goal<I> {
        Goal::new(interner, self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QuantifierKind {
    ForAll,
    Exists,
}

/// A constraint on lifetimes.
///
/// When we search for solutions within the trait system, we essentially ignore
/// lifetime constraints, instead gathering them up to return with our solution
/// for later checking. This allows for decoupling between type and region
/// checking in the compiler.
#[derive(Clone, PartialEq, Eq, Hash, Fold, Visit, HasInterner)]
pub enum Constraint<I: Interner> {
    LifetimeEq(Lifetime<I>, Lifetime<I>),
}

/// A mapping of inference variables to instantiations thereof.
#[derive(Copy, Clone, PartialEq, Eq, Hash, HasInterner)]
pub struct Substitution<I: Interner> {
    /// Map free variable with given index to the value with the same
    /// index. Naturally, the kind of the variable must agree with
    /// the kind of the value.
    interned: I::InternedSubstitution,
}

impl<I: Interner> Substitution<I> {
    pub fn from(
        interner: &I,
        parameters: impl IntoIterator<Item = impl CastTo<Parameter<I>>>,
    ) -> Self {
        Self::from_fallible(
            interner,
            parameters
                .into_iter()
                .map(|p| -> Result<Parameter<I>, ()> { Ok(p.cast(interner)) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        interner: &I,
        parameters: impl IntoIterator<Item = Result<impl CastTo<Parameter<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        Ok(Substitution {
            interned: I::intern_substitution(interner, parameters.into_iter().casted(interner))?,
        })
    }

    pub fn interned(&self) -> &I::InternedSubstitution {
        &self.interned
    }

    /// Index into the list of parameters
    pub fn at(&self, interner: &I, index: usize) -> &Parameter<I> {
        &self.parameters(interner)[index]
    }

    pub fn from1(interner: &I, parameter: impl CastTo<Parameter<I>>) -> Self {
        Self::from(interner, Some(parameter))
    }

    pub fn empty(interner: &I) -> Self {
        Self::from(interner, None::<Parameter<I>>)
    }

    pub fn is_empty(&self, interner: &I) -> bool {
        self.parameters(interner).is_empty()
    }

    pub fn iter(&self, interner: &I) -> std::slice::Iter<'_, Parameter<I>> {
        self.parameters(interner).iter()
    }

    pub fn parameters(&self, interner: &I) -> &[Parameter<I>] {
        interner.substitution_data(&self.interned)
    }

    pub fn len(&self, interner: &I) -> usize {
        self.parameters(interner).len()
    }

    /// A substitution is an **identity substitution** if it looks
    /// like this
    ///
    /// ```text
    /// ?0 := ?0
    /// ?1 := ?1
    /// ?2 := ?2
    /// ...
    /// ```
    ///
    /// Basically, each value is mapped to a type or lifetime with its
    /// same index.
    pub fn is_identity_subst(&self, interner: &I) -> bool {
        self.iter(interner).zip(0..).all(|(parameter, index)| {
            let index_db = BoundVar::new(DebruijnIndex::INNERMOST, index);
            match parameter.data(interner) {
                ParameterKind::Ty(ty) => match ty.data(interner) {
                    TyData::BoundVar(depth) => index_db == *depth,
                    _ => false,
                },
                ParameterKind::Lifetime(lifetime) => match lifetime.data(interner) {
                    LifetimeData::BoundVar(depth) => index_db == *depth,
                    _ => false,
                },
            }
        })
    }

    pub fn apply<T>(&self, value: &T, interner: &I) -> T::Result
    where
        T: Fold<I, I>,
    {
        value
            .fold_with(
                &mut &SubstFolder {
                    interner,
                    subst: self,
                },
                DebruijnIndex::INNERMOST,
            )
            .unwrap()
    }
}

struct SubstFolder<'i, I: Interner> {
    interner: &'i I,
    subst: &'i Substitution<I>,
}

impl<I: Interner> SubstFolder<'_, I> {
    /// Index into the list of parameters
    pub fn at(&self, index: usize) -> &Parameter<I> {
        let interner = self.interner;
        &self.subst.parameters(interner)[index]
    }
}

pub trait AsParameters<I: Interner> {
    fn as_parameters(&self, interner: &I) -> &[Parameter<I>];
}

impl<I: Interner> AsParameters<I> for Substitution<I> {
    #[allow(unreachable_code, unused_variables)]
    fn as_parameters(&self, interner: &I) -> &[Parameter<I>] {
        self.parameters(interner)
    }
}

impl<I: Interner> AsParameters<I> for [Parameter<I>] {
    fn as_parameters(&self, _interner: &I) -> &[Parameter<I>] {
        self
    }
}

impl<I: Interner> AsParameters<I> for [Parameter<I>; 1] {
    fn as_parameters(&self, _interner: &I) -> &[Parameter<I>] {
        self
    }
}

impl<I: Interner> AsParameters<I> for Vec<Parameter<I>> {
    fn as_parameters(&self, _interner: &I) -> &[Parameter<I>] {
        self
    }
}

impl<T, I: Interner> AsParameters<I> for &T
where
    T: ?Sized + AsParameters<I>,
{
    fn as_parameters(&self, interner: &I) -> &[Parameter<I>] {
        T::as_parameters(self, interner)
    }
}

impl<'i, I: Interner> Folder<'i, I> for &SubstFolder<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_var_ty(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        assert_eq!(bound_var.debruijn, DebruijnIndex::INNERMOST);
        let ty = self.at(bound_var.index);
        let ty = ty.assert_ty_ref(self.interner());
        Ok(ty.shifted_in_from(self.interner(), outer_binder))
    }

    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        assert_eq!(bound_var.debruijn, DebruijnIndex::INNERMOST);
        let l = self.at(bound_var.index);
        let l = l.assert_lifetime_ref(self.interner());
        Ok(l.shifted_in_from(self.interner(), outer_binder))
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}

/// Combines a substitution (`subst`) with a set of region constraints
/// (`constraints`). This represents the result of a query; the
/// substitution stores the values for the query's unknown variables,
/// and the constraints represents any region constraints that must
/// additionally be solved.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, Visit, HasInterner)]
pub struct ConstrainedSubst<I: Interner> {
    pub subst: Substitution<I>, /* NB: The `is_trivial` routine relies on the fact that `subst` is folded first. */
    pub constraints: Vec<InEnvironment<Constraint<I>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, Visit, HasInterner)]
pub struct AnswerSubst<I: Interner> {
    pub subst: Substitution<I>, /* NB: The `is_trivial` routine relies on the fact that `subst` is folded first. */
    pub constraints: Vec<InEnvironment<Constraint<I>>>,
    pub delayed_subgoals: Vec<InEnvironment<Goal<I>>>,
}
