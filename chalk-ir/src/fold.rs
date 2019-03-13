//! Traits for transforming bits of IR.

use crate::cast::Cast;
use crate::*;
use chalk_engine::context::Context;
use chalk_engine::{DelayedLiteral, ExClause, Literal};
use std::fmt::Debug;
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
pub trait Folder: FreeVarFolder + InferenceFolder + PlaceholderFolder + TypeFolder {
    /// Returns a "dynamic" version of this trait. There is no
    /// **particular** reason to require this, except that I didn't
    /// feel like making `super_fold_ty` generic for no reason.
    fn to_dyn(&mut self) -> &mut dyn Folder;
}

pub trait TypeFolder {
    fn fold_ty(&mut self, ty: &Ty, binders: usize) -> Fallible<Ty>;
    fn fold_lifetime(&mut self, lifetime: &Lifetime, binders: usize) -> Fallible<Lifetime>;
}

impl<T> Folder for T
where
    T: FreeVarFolder + InferenceFolder + PlaceholderFolder + TypeFolder,
{
    fn to_dyn(&mut self) -> &mut dyn Folder {
        self
    }
}

/// A convenience trait that indicates that this folder doesn't take
/// any action on types in particular, but just recursively folds
/// their contents (note that free variables that are encountered in
/// that process may still be substituted). The vast majority of
/// folders implement this trait.
pub trait DefaultTypeFolder {}

impl<T> TypeFolder for T
where
    T: FreeVarFolder + InferenceFolder + PlaceholderFolder + DefaultTypeFolder,
{
    fn fold_ty(&mut self, ty: &Ty, binders: usize) -> Fallible<Ty> {
        super_fold_ty(self.to_dyn(), ty, binders)
    }

    fn fold_lifetime(&mut self, lifetime: &Lifetime, binders: usize) -> Fallible<Lifetime> {
        super_fold_lifetime(self.to_dyn(), lifetime, binders)
    }
}

/// The methods for folding **free variables**. These are `BoundVar`
/// instances where the binder is not something we folded over.  This
/// is used when you are instanting previously bound things with some
/// replacement.
pub trait FreeVarFolder {
    /// Invoked for `Ty::BoundVar` instances that are not bound within the type being folded
    /// over:
    ///
    /// - `depth` is the depth of the `Ty::BoundVar`; this has been adjusted to account for binders
    ///   in scope.
    /// - `binders` is the number of binders in scope.
    ///
    /// This should return a type suitable for a context with `binders` in scope.
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty>;

    /// As `fold_free_var_ty`, but for lifetimes.
    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime>;
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

impl<T: DefaultFreeVarFolder> FreeVarFolder for T {
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(Ty::BoundVar(depth + binders))
        }
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime> {
        if T::forbid() {
            panic!("unexpected free variable with depth `{:?}`", depth)
        } else {
            Ok(Lifetime::BoundVar(depth + binders))
        }
    }
}

pub trait PlaceholderFolder {
    /// Invoked for each occurence of a placeholder type; these are
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
    ) -> Fallible<Ty>;

    /// As with `fold_free_placeholder_ty`, but for lifetimes.
    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        binders: usize,
    ) -> Fallible<Lifetime>;
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

impl<T: DefaultPlaceholderFolder> PlaceholderFolder for T {
    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<Ty> {
        if T::forbid() {
            panic!("unexpected placeholder type `{:?}`", universe)
        } else {
            Ok(universe.to_ty())
        }
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe: PlaceholderIndex,
        _binders: usize,
    ) -> Fallible<Lifetime> {
        if T::forbid() {
            panic!("unexpected placeholder lifetime `{:?}`", universe)
        } else {
            Ok(universe.to_lifetime())
        }
    }
}

pub trait InferenceFolder {
    /// Invoked for each occurence of a inference type; these are
    /// used when we instantiate binders universally. Returns a type
    /// to use instead, which should be suitably shifted to account
    /// for `binders`.
    ///
    /// - `universe` is the universe of the `TypeName::ForAll` that was found
    /// - `binders` is the number of binders in scope
    fn fold_inference_ty(&mut self, var: InferenceVar, binders: usize) -> Fallible<Ty>;

    /// As with `fold_free_inference_ty`, but for lifetimes.
    fn fold_inference_lifetime(&mut self, var: InferenceVar, binders: usize) -> Fallible<Lifetime>;
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

impl<T: DefaultInferenceFolder> InferenceFolder for T {
    fn fold_inference_ty(&mut self, var: InferenceVar, _binders: usize) -> Fallible<Ty> {
        if T::forbid() {
            panic!("unexpected inference type `{:?}`", var)
        } else {
            Ok(var.to_ty())
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        _binders: usize,
    ) -> Fallible<Lifetime> {
        if T::forbid() {
            panic!("unexpected inference lifetime `'{:?}`", var)
        } else {
            Ok(var.to_lifetime())
        }
    }
}

/// Applies the given folder to a value.
pub trait Fold: Debug {
    /// The type of value that will be produced once folding is done.
    /// Typically this is `Self`, unless `Self` contains borrowed
    /// values, in which case owned values are produced (for example,
    /// one can fold over a `&T` value where `T: Fold`, in which case
    /// you get back a `T`, not a `&T`).
    type Result: Fold;

    /// Apply the given folder `folder` to `self`; `binders` is the
    /// number of binders that are in scope when beginning the
    /// folder. Typically `binders` starts as 0, but is adjusted when
    /// we encounter `Binders<T>` in the IR or other similar
    /// constructs.
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result>;
}

impl<'a, T: Fold> Fold for &'a T {
    type Result = T::Result;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        (**self).fold_with(folder, binders)
    }
}

impl<T: Fold> Fold for Vec<T> {
    type Result = Vec<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        self.iter().map(|e| e.fold_with(folder, binders)).collect()
    }
}

impl<T: Fold> Fold for Box<T> {
    type Result = Box<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        Ok(Box::new((**self).fold_with(folder, binders)?))
    }
}

impl<T: Fold> Fold for Arc<T> {
    type Result = Arc<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        Ok(Arc::new((**self).fold_with(folder, binders)?))
    }
}

macro_rules! tuple_fold {
    ($($n:ident),*) => {
        impl<$($n: Fold,)*> Fold for ($($n,)*) {
            type Result = ($($n::Result,)*);
            fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
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

impl<T: Fold> Fold for Option<T> {
    type Result = Option<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        match self {
            None => Ok(None),
            Some(e) => Ok(Some(e.fold_with(folder, binders)?)),
        }
    }
}

impl Fold for Ty {
    type Result = Self;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        folder.fold_ty(self, binders)
    }
}

pub fn super_fold_ty(folder: &mut dyn Folder, ty: &Ty, binders: usize) -> Fallible<Ty> {
    match *ty {
        Ty::BoundVar(depth) => {
            if depth >= binders {
                folder.fold_free_var_ty(depth - binders, binders)
            } else {
                Ok(Ty::BoundVar(depth))
            }
        }
        Ty::InferenceVar(var) => folder.fold_inference_ty(var, binders),
        Ty::Apply(ref apply) => {
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

                TypeName::ItemId(_) | TypeName::AssociatedType(_) => {
                    let parameters = parameters.fold_with(folder, binders)?;
                    Ok(ApplicationTy { name, parameters }.cast())
                }
            }
        }
        Ty::Projection(ref proj) => Ok(Ty::Projection(proj.fold_with(folder, binders)?)),
        Ty::UnselectedProjection(ref proj) => {
            Ok(Ty::UnselectedProjection(proj.fold_with(folder, binders)?))
        }
        Ty::ForAll(ref quantified_ty) => Ok(Ty::ForAll(quantified_ty.fold_with(folder, binders)?)),
    }
}

impl Fold for QuantifiedTy {
    type Result = Self;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
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

impl<T> Fold for Binders<T>
where
    T: Fold,
{
    type Result = Binders<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
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

impl<T> Fold for Canonical<T>
where
    T: Fold,
{
    type Result = Canonical<T::Result>;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
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

impl Fold for Lifetime {
    type Result = Self;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        folder.fold_lifetime(self, binders)
    }
}

pub fn super_fold_lifetime(
    folder: &mut dyn Folder,
    lifetime: &Lifetime,
    binders: usize,
) -> Fallible<Lifetime> {
    match *lifetime {
        Lifetime::BoundVar(depth) => {
            if depth >= binders {
                folder.fold_free_var_lifetime(depth - binders, binders)
            } else {
                Ok(Lifetime::BoundVar(depth))
            }
        }
        Lifetime::InferenceVar(var) => folder.fold_inference_lifetime(var, binders),
        Lifetime::Placeholder(universe) => folder.fold_free_placeholder_lifetime(universe, binders),
    }
}

impl Fold for Substitution {
    type Result = Substitution;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        let parameters = self.parameters.fold_with(folder, binders)?;
        Ok(Substitution { parameters })
    }
}

impl Fold for Parameter {
    type Result = Parameter;
    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        let inner = self.0.fold_with(folder, binders)?;
        Ok(Parameter(inner))
    }
}

#[macro_export]
macro_rules! copy_fold {
    ($t:ty) => {
        impl $crate::fold::Fold for $t {
            type Result = Self;
            fn fold_with(
                &self,
                _folder: &mut dyn ($crate::fold::Folder),
                _binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(*self)
            }
        }
    };
}

copy_fold!(Identifier);
copy_fold!(UniverseIndex);
copy_fold!(ItemId);
copy_fold!(usize);
copy_fold!(QuantifierKind);
copy_fold!(chalk_engine::TableIndex);
// copy_fold!(TypeName); -- intentionally omitted! This is folded via `fold_ap`
copy_fold!(());

#[macro_export]
macro_rules! enum_fold {
    ($s:ident [$($n:ident),*] { $($variant:ident($($name:ident),*)),* } $($w:tt)*) => {
        impl<$($n),*> $crate::fold::Fold for $s<$($n),*> $($w)* {
            type Result = $s<$($n :: Result),*>;
            fn fold_with(&self,
                         folder: &mut dyn ($crate::fold::Folder),
                         binders: usize)
                         -> ::chalk_engine::fallible::Fallible<Self::Result> {
                match self {
                    $(
                        $s::$variant( $($name),* ) => {
                            Ok($s::$variant( $($name.fold_with(folder, binders)?),* ))
                        }
                    )*
                }
            }
        }
    };

    // Hacky variant for use in slg::context::implementation
    ($s:ty { $p:ident :: { $($variant:ident($($name:ident),*)),* } }) => {
        impl $crate::fold::Fold for $s {
            type Result = $s;
            fn fold_with(&self,
                         folder: &mut dyn ($crate::fold::Folder),
                         binders: usize)
                         -> ::chalk_engine::fallible::Fallible<Self::Result> {
                match self {
                    $(
                        $p::$variant( $($name),* ) => {
                            Ok($p::$variant( $($name.fold_with(folder, binders)?),* ))
                        }
                    )*
                }
            }
        }
    }
}

enum_fold!(ParameterKind[T,L] { Ty(a), Lifetime(a) } where T: Fold, L: Fold);
enum_fold!(WhereClause[] { Implemented(a), ProjectionEq(a) });
enum_fold!(WellFormed[] { Trait(a), Ty(a) });
enum_fold!(FromEnv[] { Trait(a), Ty(a) });
enum_fold!(DomainGoal[] { Holds(a), WellFormed(a), FromEnv(a), Normalize(a), UnselectedNormalize(a),
                          InScope(a), Derefs(a), IsLocal(a), IsUpstream(a), IsFullyVisible(a),
                          LocalImplAllowed(a), Compatible(a), DownstreamType(a) });
enum_fold!(LeafGoal[] { EqGoal(a), DomainGoal(a) });
enum_fold!(Constraint[] { LifetimeEq(a, b) });
enum_fold!(Goal[] { Quantified(qkind, subgoal), Implies(wc, subgoal), And(g1, g2), Not(g),
                    Leaf(wc), CannotProve(a) });
enum_fold!(ProgramClause[] { Implies(a), ForAll(a) });

#[macro_export]
macro_rules! struct_fold {
    ($s:ident $([$($tt_args:tt)*])* { $($name:ident),* $(,)* } $($w:tt)*) => {
        struct_fold! {
            @parse_tt_args($($($tt_args)*)*)
                struct_name($s)
                parameters()
                self_args()
                result_args()
                field_names($($name),*)
                where_clauses($($w)*)
        }
    };

    (
        @parse_tt_args()
            struct_name($s:ident)
            parameters($($parameters:tt)*)
            self_args($($self_args:tt)*)
            result_args($($result_args:tt)*)
            field_names($($field_names:tt)*)
        where_clauses($($where_clauses:tt)*)
    ) => {
        struct_fold! {
            @parsed_tt_args
                struct_name($s)
                parameters($($parameters)*)
                self_ty($s < $($self_args)* >)
                result_ty($s < $($result_args)* >)
                field_names($($field_names)*)
                where_clauses($($where_clauses)*)
        }
    };

    (
        @parse_tt_args(, $($input:tt)*)
            struct_name($s:ident)
            parameters($($parameters:tt)*)
            self_args($($self_args:tt)*)
            result_args($($result_args:tt)*)
            field_names($($field_names:tt)*)
        where_clauses($($where_clauses:tt)*)
    ) => {
        struct_fold! {
            @parse_tt_args($($input)*)
                struct_name($s)
                parameters($($parameters)*,)
                self_args($($self_args)*,)
                result_args($($result_args)*,)
                field_names($($field_names)*)
            where_clauses($($where_clauses)*)
        }
    };

    (
        @parse_tt_args(- $n:ident $($input:tt)*)
            struct_name($s:ident)
            parameters($($parameters:tt)*)
            self_args($($self_args:tt)*)
            result_args($($result_args:tt)*)
            field_names($($field_names:tt)*)
        where_clauses($($where_clauses:tt)*)
    ) => {
        struct_fold! {
            @parse_tt_args($($input)*)
                struct_name($s)
                parameters($($parameters)* $n)
                self_args($($self_args)* $n)
                result_args($($result_args)* $n)
                field_names($($field_names)*)
            where_clauses($($where_clauses)*)
        }
    };

    (
        @parse_tt_args($n:ident $($input:tt)*)
            struct_name($s:ident)
            parameters($($parameters:tt)*)
            self_args($($self_args:tt)*)
            result_args($($result_args:tt)*)
            field_names($($field_names:tt)*)
        where_clauses($($where_clauses:tt)*)
    ) => {
        struct_fold! {
            @parse_tt_args($($input)*)
                struct_name($s)
                parameters($($parameters)* $n)
                self_args($($self_args)* $n)
                result_args($($result_args)* $n :: Result)
                field_names($($field_names)*)
            where_clauses($($where_clauses)*)
        }
    };

    (
        @parsed_tt_args
            struct_name($s:ident)
            parameters($($parameters:tt)*)
            self_ty($self_ty:ty)
            result_ty($result_ty:ty)
            field_names($($field_name:ident),*)
        where_clauses($($where_clauses:tt)*)
    ) => {
        impl<$($parameters)*> $crate::fold::Fold for $self_ty $($where_clauses)* {
            type Result = $result_ty;
            fn fold_with(&self,
                         folder: &mut dyn ($crate::fold::Folder),
                         binders: usize)
                         -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok($s {
                    $($field_name: self.$field_name.fold_with(folder, binders)?),*
                })
            }
        }
    };
}

struct_fold!(ProjectionTy {
    associated_ty_id,
    parameters,
});
struct_fold!(UnselectedProjectionTy {
    type_name,
    parameters,
});
struct_fold!(TraitRef {
    trait_id,
    parameters,
});
struct_fold!(Normalize { projection, ty });
struct_fold!(ProjectionEq { projection, ty });
struct_fold!(UnselectedNormalize { projection, ty });
struct_fold!(Environment { clauses });
struct_fold!(InEnvironment[F] { environment, goal } where F: Fold<Result = F>);
struct_fold!(EqGoal { a, b });
struct_fold!(Derefs { source, target });
struct_fold!(ProgramClauseImplication {
    consequence,
    conditions,
});

struct_fold!(ConstrainedSubst {
    subst, /* NB: The `is_trivial` routine relies on the fact that `subst` is folded first. */
    constraints,
});

// struct_fold!(ApplicationTy { name, parameters }); -- intentionally omitted, folded through Ty

impl<C: Context> Fold for ExClause<C>
where
    C: Context,
    C::Substitution: Fold<Result = C::Substitution>,
    C::RegionConstraint: Fold<Result = C::RegionConstraint>,
    C::CanonicalConstrainedSubst: Fold<Result = C::CanonicalConstrainedSubst>,
    C::GoalInEnvironment: Fold<Result = C::GoalInEnvironment>,
{
    type Result = ExClause<C>;

    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        let ExClause {
            subst,
            delayed_literals,
            constraints,
            subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, binders)?,
            delayed_literals: delayed_literals.fold_with(folder, binders)?,
            constraints: constraints.fold_with(folder, binders)?,
            subgoals: subgoals.fold_with(folder, binders)?,
        })
    }
}

impl<C: Context> Fold for DelayedLiteral<C>
where
    C: Context,
    C::CanonicalConstrainedSubst: Fold<Result = C::CanonicalConstrainedSubst>,
{
    type Result = DelayedLiteral<C>;

    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
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

impl<C: Context> Fold for Literal<C>
where
    C: Context,
    C::GoalInEnvironment: Fold<Result = C::GoalInEnvironment>,
{
    type Result = Literal<C>;

    fn fold_with(&self, folder: &mut dyn Folder, binders: usize) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
