//! Traits for transforming bits of IR.

use crate::cast::Cast;
use crate::*;
use chalk_engine::context::Context;
use chalk_engine::{ExClause, FlounderedSubgoal, Literal};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

pub mod shift;
mod subst;

pub use self::subst::Subst;

/// A "folder" is a transformer that can be used to make a copy of
/// some term -- that is, some bit of IR, such as a `Goal` -- with
/// certain changes applied. The idea is that it contains methods that
/// let you swap types/lifetimes for new types/lifetimes; meanwhile,
/// each bit of IR implements the `Fold` trait which, given a
/// `Folder`, will reconstruct itself, invoking the folder's methods
/// to transform each of the types/lifetimes embedded within.
///
/// # Type families
///
/// The Folder trait has two type parameters, `TF` and `TTF`:
///
/// * `TF` is the "source type family" that we are folding *from*
/// * `TTF` is the "target type family" that we are folding *into*
///
/// Often, both are the same.
///
/// # Usage patterns
///
/// ## Substituting for free variables
///
/// Most of the time, though, we are not interested in adjust
/// arbitrary types/lifetimes, but rather just free variables (even
/// more often, just free existential variables) that appear within
/// the term.
///
/// For this reason, the `Folder` trait extends two other traits that
/// contain methods that are invoked when just those particular
///
/// In particular, folders can intercept references to free variables
/// (either existentially or universally quantified) and replace them
/// with other types/lifetimes as appropriate.
///
/// To create a folder `F`, one never implements `Folder` directly, but instead
/// implements one of each of these three sub-traits:
///
/// - `FreeVarFolder` -- folds `BoundVar` instances that appear free
///   in the term being folded (use `DefaultFreeVarFolder` to
///   ignore/forbid these altogether)
/// - `InferenceFolder` -- folds existential `InferenceVar` instances
///   that appear in the term being folded (use
///   `DefaultInferenceFolder` to ignore/forbid these altogether)
/// - `PlaceholderFolder` -- folds universal `Placeholder` instances
///   that appear in the term being folded (use
///   `DefaultPlaceholderFolder` to ignore/forbid these altogether)
///
/// To **apply** a folder, use the `Fold::fold_with` method, like so
///
/// ```rust,ignore
/// let x = x.fold_with(&mut folder, 0);
/// ```
pub trait Folder<TF: TypeFamily, TTF: TypeFamily>:
    FreeVarFolder<TTF> + InferenceFolder<TTF> + PlaceholderFolder<TTF> + TypeFolder<TF, TTF>
{
}

pub trait TypeFolder<TF: TypeFamily, TTF: TypeFamily = TF> {
    fn fold_ty(&mut self, ty: &Ty<TF>, binders: usize) -> Fallible<Ty<TTF>>;
    fn fold_lifetime(&mut self, lifetime: &Lifetime<TF>, binders: usize)
        -> Fallible<Lifetime<TTF>>;
}

impl<T, TF, TTF> Folder<TF, TTF> for T
where
    T: FreeVarFolder<TTF> + InferenceFolder<TTF> + PlaceholderFolder<TTF> + TypeFolder<TF, TTF>,
    TF: TypeFamily,
    TTF: TypeFamily,
{
}

/// A convenience trait that indicates that this folder doesn't take
/// any action on types in particular, but just recursively folds
/// their contents (note that free variables that are encountered in
/// that process may still be substituted). The vast majority of
/// folders implement this trait.
pub trait DefaultTypeFolder {}

impl<T, TF, TTF> TypeFolder<TF, TTF> for T
where
    T: FreeVarFolder<TTF> + InferenceFolder<TTF> + PlaceholderFolder<TTF> + DefaultTypeFolder,
    TF: TypeFamily,
    TTF: TypeFamily,
{
    fn fold_ty(&mut self, ty: &Ty<TF>, binders: usize) -> Fallible<Ty<TTF>> {
        super_fold_ty(self, ty, binders)
    }

    fn fold_lifetime(
        &mut self,
        lifetime: &Lifetime<TF>,
        binders: usize,
    ) -> Fallible<Lifetime<TTF>> {
        super_fold_lifetime(self, lifetime, binders)
    }
}

/// The methods for folding **free variables**. These are `BoundVar`
/// instances where the binder is not something we folded over.  This
/// is used when you are instantiating previously bound things with some
/// replacement.
pub trait FreeVarFolder<TTF: TypeFamily> {
    /// Invoked for `TyData::BoundVar` instances that are not bound within the type being folded
    /// over:
    ///
    /// - `depth` is the depth of the `TyData::BoundVar`; this has been adjusted to account for binders
    ///   in scope.
    /// - `binders` is the number of binders in scope.
    ///
    /// This should return a type suitable for a context with `binders` in scope.
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty<TTF>>;

    /// As `fold_free_var_ty`, but for lifetimes.
    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime<TTF>>;
}

/// A convenience trait. If you implement this, you get an
/// implementation of `FreeVarFolder` for free that simply ignores
/// free values (that is, it replaces them with themselves).
///
/// You can make it panic if a free-variable is found by overriding
/// `forbid` to return true.
pub trait DefaultFreeVarFolder {
    fn forbid() -> bool {
        false
    }
}

impl<T: DefaultFreeVarFolder, TTF: TypeFamily> FreeVarFolder<TTF> for T {
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty<TTF>> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(TyData::<TTF>::BoundVar(depth + binders).intern())
        }
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime<TTF>> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(LifetimeData::<TTF>::BoundVar(depth + binders).intern())
        }
    }
}

pub trait PlaceholderFolder<TTF: TypeFamily> {
    /// Invoked for each occurrence of a placeholder type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        binders: usize,
    ) -> Fallible<Ty<TTF>>;

    /// As with `fold_free_placeholder_ty`, but for lifetimes.
    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        binders: usize,
    ) -> Fallible<Lifetime<TTF>>;
}

/// A convenience trait. If you implement this, you get an
/// implementation of `PlaceholderFolder` for free that simply ignores
/// placeholder values (that is, it replaces them with themselves).
///
/// You can make it panic if a free-variable is found by overriding
/// `forbid` to return true.
pub trait DefaultPlaceholderFolder {
    fn forbid() -> bool {
        false
    }
}

impl<T: DefaultPlaceholderFolder, TTF: TypeFamily> PlaceholderFolder<TTF> for T {
    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<Ty<TTF>> {
        if T::forbid() {
            panic!("unexpected placeholder type `{:?}`", universe)
        } else {
            Ok(universe.to_ty::<TTF>())
        }
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<Lifetime<TTF>> {
        if T::forbid() {
            panic!("unexpected placeholder lifetime `{:?}`", universe)
        } else {
            Ok(universe.to_lifetime::<TTF>())
        }
    }
}

pub trait InferenceFolder<TTF: TypeFamily> {
    /// Invoked for each occurrence of a inference type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    fn fold_inference_ty(&mut self, var: InferenceVar, binders: usize) -> Fallible<Ty<TTF>>;

    /// As with `fold_free_inference_ty`, but for lifetimes.
    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        binders: usize,
    ) -> Fallible<Lifetime<TTF>>;
}

/// A convenience trait. If you implement this, you get an
/// implementation of `InferenceFolder` for free that simply ignores
/// inference values (that is, it replaces them with themselves).
///
/// You can make it panic if a free-variable is found by overriding
/// `forbid` to return true.
pub trait DefaultInferenceFolder {
    fn forbid() -> bool {
        false
    }
}

impl<T: DefaultInferenceFolder, TTF: TypeFamily> InferenceFolder<TTF> for T {
    fn fold_inference_ty(&mut self, var: InferenceVar, _binders: usize) -> Fallible<Ty<TTF>> {
        if T::forbid() {
            panic!("unexpected inference type `{:?}`", var)
        } else {
            Ok(var.to_ty::<TTF>())
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        _binders: usize,
    ) -> Fallible<Lifetime<TTF>> {
        if T::forbid() {
            panic!("unexpected inference lifetime `'{:?}`", var)
        } else {
            Ok(var.to_lifetime::<TTF>())
        }
    }
}

pub trait ReflexiveFold<TF: TypeFamily>: Fold<TF, TF, Result = Self> + Sized {}

impl<T, TF> ReflexiveFold<TF> for T
where
    T: Fold<TF, TF, Result = Self>,
    TF: TypeFamily,
{
}

/// Applies the given `Folder` to a value, producing a folded result
/// of type `Self::Result`. The result is in the type family
/// `TTF`. The result type is typically the same as the source type
/// (modulo type family), but in some cases we convert from borrowed
/// to owned as well (e.g., the folder for `&T` will fold to a fresh
/// `T`; well, actually `T::Result`).
///
/// # Type families
///
/// The `Fold` trait has two type parameters, `TF` and `TTF`:
///
/// * `TF` is the "source type family" that we are folding *from*
/// * `TTF` is the "target type family" that we are folding *into*
///
/// Often, both are the same.
pub trait Fold<TF: TypeFamily, TTF: TypeFamily = TF>: Debug {
    /// The type of value that will be produced once folding is done.
    /// Typically this is `Self`, unless `Self` contains borrowed
    /// values, in which case owned values are produced (for example,
    /// one can fold over a `&T` value where `T: Fold`, in which case
    /// you get back a `T`, not a `&T`).
    type Result;

    /// Apply the given folder `folder` to `self`; `binders` is the
    /// number of binders that are in scope when beginning the
    /// folder. Typically `binders` starts as 0, but is adjusted when
    /// we encounter `Binders<T>` in the IR or other similar
    /// constructs.
    fn fold_with(&self, folder: &mut dyn Folder<TF, TTF>, binders: usize)
        -> Fallible<Self::Result>;
}

impl<'a, T: Fold<TF, TTF>, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for &'a T {
    type Result = T::Result;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold<TF, TTF>,)* TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with(&self, folder: &mut dyn Folder<TF, TTF>, binders: usize) -> Fallible<Self::Result> {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                Ok(($($n.fold_with(folder, binders)?,)*))
            }
        }
    }
}

tuple_fold!(A, B);
tuple_fold!(A, B, C);
tuple_fold!(A, B, C, D);
tuple_fold!(A, B, C, D, E);

impl<T: Fold<TF, TTF>, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}

pub fn super_fold_ty<TF, TTF>(
    folder: &mut dyn Folder<TF, TTF>,
    ty: &Ty<TF>,
    binders: usize,
) -> Fallible<Ty<TTF>>
where
    TF: TypeFamily,
    TTF: TypeFamily,
{
    match ty.data() {
        TyData::BoundVar(depth) => {
            if *depth >= binders {
                folder.fold_free_var_ty(*depth - binders, binders)
            } else {
                Ok(TyData::<TTF>::BoundVar(*depth).intern())
            }
        }
        TyData::Dyn(clauses) => Ok(TyData::Dyn(clauses.fold_with(folder, binders)?).intern()),
        TyData::Opaque(clauses) => Ok(TyData::Opaque(clauses.fold_with(folder, binders)?).intern()),
        TyData::InferenceVar(var) => folder.fold_inference_ty(*var, binders),
        TyData::Apply(apply) => Ok(apply.fold_with(folder, binders)?),
        TyData::Projection(proj) => {
            Ok(TyData::Projection(proj.fold_with(folder, binders)?).intern())
        }
        TyData::ForAll(quantified_ty) => {
            Ok(TyData::ForAll(quantified_ty.fold_with(folder, binders)?).intern())
        }
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Ty<TF> {
    type Result = Ty<TTF>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        folder.fold_ty(self, binders)
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Lifetime<TF> {
    type Result = Lifetime<TTF>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        folder.fold_lifetime(self, binders)
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for ApplicationTy<TF> {
    type Result = Ty<TTF>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let ApplicationTy { name, parameters } = self;
        let name = *name;
        match name {
            TypeName::Placeholder(ui) => {
                assert!(
                    parameters.is_empty(),
                    "placeholder type {:?} with parameters {:?}",
                    self,
                    parameters
                );
                folder.fold_free_placeholder_ty(ui, binders)
            }

            TypeName::TypeKindId(_) | TypeName::AssociatedType(_) | TypeName::Error => {
                let parameters = parameters.fold_with(folder, binders)?;
                Ok(ApplicationTy { name, parameters }.cast().intern())
            }
        }
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for QuantifiedTy<TF> {
    type Result = QuantifiedTy<TTF>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let QuantifiedTy {
            num_binders,
            ref ty,
        } = *self;
        Ok(QuantifiedTy {
            num_binders,
            ty: ty.fold_with(folder, binders + num_binders)?,
        })
    }
}

impl<T, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Binders<T>
where
    T: Fold<TF, TTF>,
    TF: TypeFamily,
{
    type Result = Binders<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let Binders {
            binders: ref self_binders,
            value: ref self_value,
        } = *self;
        let value = self_value.fold_with(folder, binders + self_binders.len())?;
        Ok(Binders {
            binders: self_binders.clone(),
            value: value,
        })
    }
}

impl<T, TF, TTF> Fold<TF, TTF> for Canonical<T>
where
    T: Fold<TF, TTF>,
    TF: TypeFamily,
    TTF: TypeFamily,
{
    type Result = Canonical<T::Result>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let Canonical {
            binders: ref self_binders,
            value: ref self_value,
        } = *self;
        let value = self_value.fold_with(folder, binders + self_binders.len())?;
        Ok(Canonical {
            binders: self_binders.clone(),
            value: value,
        })
    }
}

pub fn super_fold_lifetime<TF: TypeFamily, TTF: TypeFamily>(
    folder: &mut dyn Folder<TF, TTF>,
    lifetime: &Lifetime<TF>,
    binders: usize,
) -> Fallible<Lifetime<TTF>> {
    match lifetime.data() {
        LifetimeData::BoundVar(depth) => {
            if *depth >= binders {
                folder.fold_free_var_lifetime(depth - binders, binders)
            } else {
                Ok(LifetimeData::<TTF>::BoundVar(*depth).intern())
            }
        }
        LifetimeData::InferenceVar(var) => folder.fold_inference_lifetime(*var, binders),
        LifetimeData::Placeholder(universe) => {
            folder.fold_free_placeholder_lifetime(*universe, binders)
        }
        LifetimeData::Phantom(..) => unreachable!(),
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Substitution<TF> {
    type Result = Substitution<TTF>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let parameters = self.parameters.fold_with(folder, binders)?;
        Ok(Substitution { parameters })
    }
}

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Parameter<TF> {
    type Result = Parameter<TTF>;
    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let inner = self.0.fold_with(folder, binders)?;
        Ok(Parameter(inner))
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl<TF: TypeFamily, TTF: TypeFamily> $crate::fold::Fold<TF, TTF> for $t {
            type Result = Self;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<TF, TTF>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(*self)
            }
        }
    };
}

copy_fold!(UniverseIndex);
copy_fold!(ImplId);
copy_fold!(StructId);
copy_fold!(TraitId);
copy_fold!(TypeId);
copy_fold!(TypeKindId);
copy_fold!(usize);
copy_fold!(QuantifierKind);
copy_fold!(chalk_engine::TableIndex);
copy_fold!(chalk_engine::TimeStamp);
copy_fold!(());

impl<TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for PhantomData<TF> {
    type Result = PhantomData<TTF>;

    fn fold_with(
        &self,
        _folder: &mut dyn Folder<TF, TTF>,
        _binders: usize,
    ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
        Ok(PhantomData)
    }
}

impl<TF: TypeFamily, TTF: TypeFamily, T, L> Fold<TF, TTF> for ParameterKind<T, L>
where
    T: Fold<TF, TTF>,
    L: Fold<TF, TTF>,
{
    type Result = ParameterKind<T::Result, L::Result>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            ParameterKind::Ty(a) => Ok(ParameterKind::Ty(a.fold_with(folder, binders)?)),
            ParameterKind::Lifetime(a) => {
                Ok(ParameterKind::Lifetime(a.fold_with(folder, binders)?))
            }
        }
    }
}

impl<C: Context, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for ExClause<C>
where
    C: Context,
    C::Substitution: Fold<TF, TTF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, TTF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, TTF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = ExClause<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let ExClause {
            subst,
            ambiguous,
            constraints,
            subgoals,
            current_time,
            floundered_subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, binders)?,
            ambiguous: *ambiguous,
            constraints: constraints.fold_with(folder, binders)?,
            subgoals: subgoals.fold_with(folder, binders)?,
            current_time: current_time.fold_with(folder, binders)?,
            floundered_subgoals: floundered_subgoals.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for FlounderedSubgoal<C>
where
    C: Context,
    C::Substitution: Fold<TF, TTF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, TTF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, TTF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = FlounderedSubgoal<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        let FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        } = self;
        Ok(FlounderedSubgoal {
            floundered_literal: floundered_literal.fold_with(folder, binders)?,
            floundered_time: floundered_time.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, TF: TypeFamily, TTF: TypeFamily> Fold<TF, TTF> for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Fold<TF, TTF, Result = C::GoalInEnvironment>,
{
    type Result = Literal<C>;

    fn fold_with(
        &self,
        folder: &mut dyn Folder<TF, TTF>,
        binders: usize,
    ) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
