//! Traits for transforming bits of IR.

use crate::cast::Cast;
use crate::*;
use chalk_engine::context::Context;
use chalk_engine::{DelayedLiteral, ExClause, FlounderedSubgoal, Literal};
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
pub trait Folder<TF: TypeFamily>:
    FreeVarFolder<TF> + InferenceFolder<TF> + PlaceholderFolder<TF> + TypeFolder<TF>
{
    /// Returns a "dynamic" version of this trait. There is no
    /// **particular** reason to require this, except that I didn't
    /// feel like making `super_fold_ty` generic for no reason.
    fn to_dyn(&mut self) -> &mut dyn Folder<TF>;
}

pub trait TypeFolder<TF: TypeFamily> {
    fn fold_ty(&mut self, ty: &TF::Type, binders: usize) -> Fallible<TF::Type>;
    fn fold_lifetime(&mut self, lifetime: &TF::Lifetime, binders: usize) -> Fallible<TF::Lifetime>;
}

impl<T, TF> Folder<TF> for T
where
    T: FreeVarFolder<TF> + InferenceFolder<TF> + PlaceholderFolder<TF> + TypeFolder<TF>,
    TF: TypeFamily,
{
    fn to_dyn(&mut self) -> &mut dyn Folder<TF> {
        self
    }
}

/// A convenience trait that indicates that this folder doesn't take
/// any action on types in particular, but just recursively folds
/// their contents (note that free variables that are encountered in
/// that process may still be substituted). The vast majority of
/// folders implement this trait.
pub trait DefaultTypeFolder {}

impl<T, TF> TypeFolder<TF> for T
where
    T: FreeVarFolder<TF> + InferenceFolder<TF> + PlaceholderFolder<TF> + DefaultTypeFolder,
    TF: TypeFamily,
{
    fn fold_ty(&mut self, ty: &TF::Type, binders: usize) -> Fallible<TF::Type> {
        super_fold_ty(self.to_dyn(), ty, binders)
    }

    fn fold_lifetime(&mut self, lifetime: &TF::Lifetime, binders: usize) -> Fallible<TF::Lifetime> {
        super_fold_lifetime(self.to_dyn(), lifetime, binders)
    }
}

/// The methods for folding **free variables**. These are `BoundVar`
/// instances where the binder is not something we folded over.  This
/// is used when you are instantiating previously bound things with some
/// replacement.
pub trait FreeVarFolder<TF: TypeFamily> {
    /// Invoked for `Ty::BoundVar` instances that are not bound within the type being folded
    /// over:
    ///
    /// - `depth` is the depth of the `Ty::BoundVar`; this has been adjusted to account for binders
    ///   in scope.
    /// - `binders` is the number of binders in scope.
    ///
    /// This should return a type suitable for a context with `binders` in scope.
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<TF::Type>;

    /// As `fold_free_var_ty`, but for lifetimes.
    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<TF::Lifetime>;
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

impl<T: DefaultFreeVarFolder, TF: TypeFamily> FreeVarFolder<TF> for T {
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<TF::Type> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(Ty::<TF>::BoundVar(depth + binders).intern())
        }
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<TF::Lifetime> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(Lifetime::<TF>::BoundVar(depth + binders).intern())
        }
    }
}

pub trait PlaceholderFolder<TF: TypeFamily> {
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
    ) -> Fallible<TF::Type>;

    /// As with `fold_free_placeholder_ty`, but for lifetimes.
    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        binders: usize,
    ) -> Fallible<TF::Lifetime>;
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

impl<T: DefaultPlaceholderFolder, TF: TypeFamily> PlaceholderFolder<TF> for T {
    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<TF::Type> {
        if T::forbid() {
            panic!("unexpected placeholder type `{:?}`", universe)
        } else {
            Ok(universe.to_ty::<TF>())
        }
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<TF::Lifetime> {
        if T::forbid() {
            panic!("unexpected placeholder lifetime `{:?}`", universe)
        } else {
            Ok(universe.to_lifetime::<TF>())
        }
    }
}

pub trait InferenceFolder<TF: TypeFamily> {
    /// Invoked for each occurrence of a inference type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    fn fold_inference_ty(&mut self, var: InferenceVar, binders: usize) -> Fallible<TF::Type>;

    /// As with `fold_free_inference_ty`, but for lifetimes.
    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        binders: usize,
    ) -> Fallible<TF::Lifetime>;
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

impl<T: DefaultInferenceFolder, TF: TypeFamily> InferenceFolder<TF> for T {
    fn fold_inference_ty(&mut self, var: InferenceVar, _binders: usize) -> Fallible<TF::Type> {
        if T::forbid() {
            panic!("unexpected inference type `{:?}`", var)
        } else {
            Ok(var.to_ty::<TF>())
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        _binders: usize,
    ) -> Fallible<TF::Lifetime> {
        if T::forbid() {
            panic!("unexpected inference lifetime `'{:?}`", var)
        } else {
            Ok(var.to_lifetime::<TF>())
        }
    }
}

pub trait ReflexiveFold<TF: TypeFamily>: Fold<TF, Result = Self> + Sized {}

impl<T, TF> ReflexiveFold<TF> for T
where
    T: Fold<TF, Result = Self>,
    TF: TypeFamily,
{
}

/// Applies the given folder to a value.
pub trait Fold<TF: TypeFamily>: Debug {
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
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result>;
}

impl<'a, T: Fold<TF>, TF: TypeFamily> Fold<TF> for &'a T {
    type Result = T::Result;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold<TF>, TF: TypeFamily> Fold<TF> for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold<TF>, TF: TypeFamily> Fold<TF> for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold<TF>, TF: TypeFamily> Fold<TF> for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold<TF>,)* TF: TypeFamily> Fold<TF> for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
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

impl<T: Fold<TF>, TF: TypeFamily> Fold<TF> for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}

pub fn super_fold_ty<TF>(
    folder: &mut dyn Folder<TF>,
    ty: &TF::Type,
    binders: usize,
) -> Fallible<TF::Type>
where
    TF: TypeFamily,
{
    match ty.lookup_ref() {
        Ty::BoundVar(depth) => {
            if *depth >= binders {
                folder.fold_free_var_ty(*depth - binders, binders)
            } else {
                Ok(Ty::<TF>::BoundVar(*depth).intern())
            }
        }
        Ty::Dyn(clauses) => Ok(TF::intern_ty(Ty::Dyn(clauses.fold_with(folder, binders)?))),
        Ty::Opaque(clauses) => Ok(TF::intern_ty(Ty::Opaque(
            clauses.fold_with(folder, binders)?,
        ))),
        Ty::InferenceVar(var) => folder.fold_inference_ty(*var, binders),
        Ty::Apply(apply) => {
            let ApplicationTy {
                name,
                ref parameters,
            } = *apply;
            match name {
                TypeName::Placeholder(ui) => {
                    assert!(
                        parameters.is_empty(),
                        "type {:?} with parameters {:?}",
                        ty,
                        parameters
                    );
                    folder.fold_free_placeholder_ty(ui, binders)
                }

                TypeName::TypeKindId(_) | TypeName::AssociatedType(_) => {
                    let parameters = parameters.fold_with(folder, binders)?;
                    Ok(ApplicationTy { name, parameters }.cast().intern())
                }
            }
        }
        Ty::Projection(proj) => Ok(Ty::Projection(proj.fold_with(folder, binders)?).intern()),
        Ty::ForAll(quantified_ty) => {
            Ok(Ty::ForAll(quantified_ty.fold_with(folder, binders)?).intern())
        }
    }
}

impl<TF: TypeFamily> Fold<TF> for QuantifiedTy<TF> {
    type Result = Self;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
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

impl<T, TF: TypeFamily> Fold<TF> for Binders<T>
where
    T: Fold<TF>,
    TF: TypeFamily,
{
    type Result = Binders<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
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

impl<T, TF> Fold<TF> for Canonical<T>
where
    T: Fold<TF>,
    TF: TypeFamily,
{
    type Result = Canonical<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
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

pub fn super_fold_lifetime<TF: TypeFamily>(
    folder: &mut dyn Folder<TF>,
    lifetime: &TF::Lifetime,
    binders: usize,
) -> Fallible<TF::Lifetime> {
    match lifetime.lookup_ref() {
        Lifetime::BoundVar(depth) => {
            if *depth >= binders {
                folder.fold_free_var_lifetime(depth - binders, binders)
            } else {
                Ok(Lifetime::<TF>::BoundVar(*depth).intern())
            }
        }
        Lifetime::InferenceVar(var) => folder.fold_inference_lifetime(*var, binders),
        Lifetime::Placeholder(universe) => {
            folder.fold_free_placeholder_lifetime(*universe, binders)
        }
        Lifetime::Phantom(..) => unreachable!(),
    }
}

impl<TF: TypeFamily> Fold<TF> for Substitution<TF> {
    type Result = Substitution<TF>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        let parameters = self.parameters.fold_with(folder, binders)?;
        Ok(Substitution { parameters })
    }
}

impl<TF: TypeFamily> Fold<TF> for Parameter<TF> {
    type Result = Parameter<TF>;
    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        let inner = self.0.fold_with(folder, binders)?;
        Ok(Parameter(inner))
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($TF:ident => $t:ty) => {
        impl<$TF: TypeFamily> $crate::fold::Fold<$TF> for $t {
            type Result = Self;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder<$TF>),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(*self)
            }
        }
    };
}

copy_fold!(TF => Identifier);
copy_fold!(TF => UniverseIndex);
copy_fold!(TF => ItemId);
copy_fold!(TF => ImplId);
copy_fold!(TF => StructId);
copy_fold!(TF => TraitId);
copy_fold!(TF => TypeId);
copy_fold!(TF => TypeKindId);
copy_fold!(TF => usize);
copy_fold!(TF => QuantifierKind);
copy_fold!(TF => chalk_engine::TableIndex);
copy_fold!(TF => chalk_engine::TimeStamp);
// copy_fold!(TypeName); -- intentionally omitted! This is folded via `fold_ap`
copy_fold!(TF => ());
copy_fold!(TF => PhantomData<TF>);

impl<TF: TypeFamily, T, L> Fold<TF> for ParameterKind<T, L>
where
    T: Fold<TF>,
    L: Fold<TF>,
{
    type Result = ParameterKind<T::Result, L::Result>;

    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        match self {
            ParameterKind::Ty(a) => Ok(ParameterKind::Ty(a.fold_with(folder, binders)?)),
            ParameterKind::Lifetime(a) => {
                Ok(ParameterKind::Lifetime(a.fold_with(folder, binders)?))
            }
        }
    }
}

impl<C: Context, TF: TypeFamily> Fold<TF> for ExClause<C>
where
    C: Context,
    C::Substitution: Fold<TF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, Result = C::GoalInEnvironment>,
{
    type Result = ExClause<C>;

    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        let ExClause {
            subst,
            delayed_literals,
            constraints,
            subgoals,
            current_time,
            floundered_subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, binders)?,
            delayed_literals: delayed_literals.fold_with(folder, binders)?,
            constraints: constraints.fold_with(folder, binders)?,
            subgoals: subgoals.fold_with(folder, binders)?,
            current_time: current_time.fold_with(folder, binders)?,
            floundered_subgoals: floundered_subgoals.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context, TF: TypeFamily> Fold<TF> for FlounderedSubgoal<C>
where
    C: Context,
    C::Substitution: Fold<TF, Result = C::Substitution>,
    C::RegionConstraint: Fold<TF, Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<TF, Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<TF, Result = C::GoalInEnvironment>,
{
    type Result = FlounderedSubgoal<C>;

    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
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

impl<C: Context, TF: TypeFamily> Fold<TF> for DelayedLiteral<C>
where
    C: Context,
    C::CanonicalConstrainedSubst: Fold<TF, Result = C::CanonicalConstrainedSubst>,
{
    type Result = DelayedLiteral<C>;

    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        match self {
            DelayedLiteral::CannotProve(()) => Ok(DelayedLiteral::CannotProve(())),
            DelayedLiteral::Negative(table_index) => Ok(DelayedLiteral::Negative(
                table_index.fold_with(folder, binders)?,
            )),
            DelayedLiteral::Positive(table_index, subst) => Ok(DelayedLiteral::Positive(
                table_index.fold_with(folder, binders)?,
                subst.fold_with(folder, binders)?,
            )),
        }
    }
}

impl<C: Context, TF: TypeFamily> Fold<TF> for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Fold<TF, Result = C::GoalInEnvironment>,
{
    type Result = Literal<C>;

    fn fold_with(&self, folder: &mut dyn Folder<TF>, binders: usize) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
