//! Traits for transforming bits of IR.

use crate::interner::TargetInterner;
use crate::*;
use std::fmt::Debug;

mod binder_impls;
mod boring_impls;
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
/// # Interners
///
/// The Folder trait has two type parameters, `I` and `TI`:
///
/// * `I` is the "source interner" that we are folding *from*
/// * `TI` is the "target interner" that we are folding *into*
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
pub trait Folder<'i, I: Interner, TI: TargetInterner<I> = I>
where
    I: 'i,
    TI: 'i,
{
    /// Creates a `dyn` value from this folder. Unfortunately, this
    /// must be added manually to each impl of Folder; it permits the
    /// default implements below to create a `&mut dyn Folder` from
    /// `Self` without knowing what `Self` is (by invoking this
    /// method). Effectively, this limits impls of `Folder` to types
    /// for which we are able to create a dyn value (i.e., not `[T]`
    /// types).
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I, TI>;

    /// Top-level callback: invoked for each `Ty<I>` that is
    /// encountered when folding. By default, invokes
    /// `super_fold_with`, which will in turn invoke the more
    /// specialized folding methods below, like `fold_free_var_ty`.
    fn fold_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Fallible<Ty<TI>> {
        ty.super_fold_with(self.as_dyn(), outer_binder)
    }

    /// Top-level callback: invoked for each `Lifetime<I>` that is
    /// encountered when folding. By default, invokes
    /// `super_fold_with`, which will in turn invoke the more
    /// specialized folding methods below, like `fold_free_lifetime_ty`.
    fn fold_lifetime(
        &mut self,
        lifetime: &Lifetime<I>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<TI>> {
        lifetime.super_fold_with(self.as_dyn(), outer_binder)
    }

    /// Invoked for every program clause. By default, recursively folds the goals contents.
    fn fold_program_clause(
        &mut self,
        clause: &ProgramClause<I>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<ProgramClause<TI>> {
        clause.super_fold_with(self.as_dyn(), outer_binder)
    }

    /// Invoked for every goal. By default, recursively folds the goals contents.
    fn fold_goal(&mut self, goal: &Goal<I>, outer_binder: DebruijnIndex) -> Fallible<Goal<TI>> {
        goal.super_fold_with(self.as_dyn(), outer_binder)
    }

    /// If overridden to return true, then folding will panic if a
    /// free variable is encountered. This should be done if free
    /// type/lifetime variables are not expected.
    fn forbid_free_vars(&self) -> bool {
        false
    }

    /// Invoked for `TyData::BoundVar` instances that are not bound
    /// within the type being folded over:
    ///
    /// - `depth` is the depth of the `TyData::BoundVar`; this has
    ///   been adjusted to account for binders in scope.
    /// - `binders` is the number of binders in scope.
    ///
    /// This should return a type suitable for a context with
    /// `binders` in scope.
    fn fold_free_var_ty(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<TI>> {
        if self.forbid_free_vars() {
            panic!(
                "unexpected free variable with depth `{:?}` with outer binder {:?}",
                bound_var, outer_binder
            )
        } else {
            let bound_var = bound_var.shifted_in_from(outer_binder);
            Ok(TyData::<TI>::BoundVar(bound_var).intern(self.target_interner()))
        }
    }

    /// As `fold_free_var_ty`, but for lifetimes.
    fn fold_free_var_lifetime(
        &mut self,
        bound_var: BoundVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<TI>> {
        if self.forbid_free_vars() {
            panic!(
                "unexpected free variable with depth `{:?}` with outer binder {:?}",
                bound_var, outer_binder
            )
        } else {
            let bound_var = bound_var.shifted_in_from(outer_binder);
            Ok(LifetimeData::<TI>::BoundVar(bound_var).intern(self.target_interner()))
        }
    }

    /// If overriden to return true, we will panic when a free
    /// placeholder type/lifetime is encountered.
    fn forbid_free_placeholders(&self) -> bool {
        false
    }

    /// Invoked for each occurrence of a placeholder type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    #[allow(unused_variables)]
    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<TI>> {
        if self.forbid_free_placeholders() {
            panic!("unexpected placeholder type `{:?}`", universe)
        } else {
            Ok(universe.to_ty::<TI>(self.target_interner()))
        }
    }

    /// As with `fold_free_placeholder_ty`, but for lifetimes.
    #[allow(unused_variables)]
    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<TI>> {
        if self.forbid_free_placeholders() {
            panic!("unexpected placeholder lifetime `{:?}`", universe)
        } else {
            Ok(universe.to_lifetime(self.target_interner()))
        }
    }

    /// If overridden to return true, inference variables will trigger
    /// panics when folded. Used when inference variables are
    /// unexpected.
    fn forbid_inference_vars(&self) -> bool {
        false
    }

    /// Invoked for each occurrence of a inference type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    #[allow(unused_variables)]
    fn fold_inference_ty(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<TI>> {
        if self.forbid_inference_vars() {
            panic!("unexpected inference type `{:?}`", var)
        } else {
            Ok(var.to_ty(self.target_interner()))
        }
    }

    /// As with `fold_free_inference_ty`, but for lifetimes.
    #[allow(unused_variables)]
    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<TI>> {
        if self.forbid_inference_vars() {
            panic!("unexpected inference lifetime `'{:?}`", var)
        } else {
            Ok(var.to_lifetime(self.target_interner()))
        }
    }

    fn interner(&self) -> &'i I;

    fn target_interner(&self) -> &'i TI;
}

/// Applies the given `Folder` to a value, producing a folded result
/// of type `Self::Result`. The result is in the interner
/// `TI`. The result type is typically the same as the source type
/// (modulo interner), but in some cases we convert from borrowed
/// to owned as well (e.g., the folder for `&T` will fold to a fresh
/// `T`; well, actually `T::Result`).
///
/// # Interners
///
/// The `Fold` trait has two type parameters, `I` and `TI`:
///
/// * `I` is the "source interner" that we are folding *from*
/// * `TI` is the "target interner" that we are folding *into*
///
/// Often, both are the same.
pub trait Fold<I: Interner, TI: TargetInterner<I> = I>: Debug {
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
    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i;
}

/// For types where "fold" invokes a callback on the `Folder`, the
/// `SuperFold` trait captures the recursive behavior that folds all
/// the contents of the type.
pub trait SuperFold<I: Interner, TI: TargetInterner<I> = I>: Fold<I, TI> {
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i;
}

/// "Folding" a type invokes the `fold_ty` method on the folder; this
/// usually (in turn) invokes `super_fold_ty` to fold the individual
/// parts.
impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Ty<I> {
    type Result = Ty<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        folder.fold_ty(self, outer_binder)
    }
}

/// "Super fold" for a type invokes te more detailed callbacks on the type
impl<I, TI> SuperFold<I, TI> for Ty<I>
where
    I: Interner,
    TI: TargetInterner<I>,
{
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<TI>>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        match self.data(interner) {
            TyData::BoundVar(bound_var) => {
                if let Some(bound_var1) = bound_var.shifted_out_to(outer_binder) {
                    // This variable was bound outside of the binders
                    // that we have traversed during folding;
                    // therefore, it is free. Let the folder have a
                    // crack at it.
                    folder.fold_free_var_ty(bound_var1, outer_binder)
                } else {
                    // This variable was bound within the binders that
                    // we folded over, so just return a bound
                    // variable.
                    Ok(TyData::<TI>::BoundVar(*bound_var).intern(folder.target_interner()))
                }
            }
            TyData::Dyn(clauses) => Ok(TyData::Dyn(clauses.fold_with(folder, outer_binder)?)
                .intern(folder.target_interner())),
            TyData::InferenceVar(var) => folder.fold_inference_ty(*var, outer_binder),
            TyData::Apply(apply) => Ok(TyData::Apply(apply.fold_with(folder, outer_binder)?)
                .intern(folder.target_interner())),
            TyData::Placeholder(ui) => Ok(folder.fold_free_placeholder_ty(*ui, outer_binder)?),
            TyData::Alias(proj) => Ok(TyData::Alias(proj.fold_with(folder, outer_binder)?)
                .intern(folder.target_interner())),
            TyData::Function(fun) => Ok(TyData::Function(fun.fold_with(folder, outer_binder)?)
                .intern(folder.target_interner())),
        }
    }
}

/// "Folding" a lifetime invokes the `fold_lifetime` method on the folder; this
/// usually (in turn) invokes `super_fold_lifetime` to fold the individual
/// parts.
impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Lifetime<I> {
    type Result = Lifetime<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        folder.fold_lifetime(self, outer_binder)
    }
}

impl<I, TI> SuperFold<I, TI> for Lifetime<I>
where
    I: Interner,
    TI: TargetInterner<I>,
{
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<TI>>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        match self.data(interner) {
            LifetimeData::BoundVar(bound_var) => {
                if let Some(bound_var1) = bound_var.shifted_out_to(outer_binder) {
                    // This variable was bound outside of the binders
                    // that we have traversed during folding;
                    // therefore, it is free. Let the folder have a
                    // crack at it.
                    folder.fold_free_var_lifetime(bound_var1, outer_binder)
                } else {
                    // This variable was bound within the binders that
                    // we folded over, so just return a bound
                    // variable.
                    Ok(LifetimeData::<TI>::BoundVar(*bound_var).intern(folder.target_interner()))
                }
            }
            LifetimeData::InferenceVar(var) => folder.fold_inference_lifetime(*var, outer_binder),
            LifetimeData::Placeholder(universe) => {
                folder.fold_free_placeholder_lifetime(*universe, outer_binder)
            }
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

/// Folding a goal invokes the `fold_goal` callback (which will, by
/// default, invoke super-fold).
impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Goal<I> {
    type Result = Goal<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        folder.fold_goal(self, outer_binder)
    }
}

/// Superfold folds recursively.
impl<I: Interner, TI: TargetInterner<I>> SuperFold<I, TI> for Goal<I> {
    fn super_fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let interner = folder.interner();
        let target_interner = folder.target_interner();
        Ok(Goal::new(
            target_interner,
            self.data(interner).fold_with(folder, outer_binder)?,
        ))
    }
}

/// Folding a program clause invokes the `fold_program_clause`
/// callback on the folder (which will, by default, invoke the
/// `super_fold_with` method on the program clause).
impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ProgramClause<I> {
    type Result = ProgramClause<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        folder.fold_program_clause(self, outer_binder)
    }
}
