//! Traits for visiting bits of IR.
use std::fmt::Debug;

use crate::{
    BoundVar, Const, ConstValue, DebruijnIndex, DomainGoal, Goal, InferenceVar, Interner, Lifetime,
    LifetimeData, PlaceholderIndex, ProgramClause, Ty, TyData, WhereClause,
};

mod binder_impls;
mod boring_impls;
pub mod visitors;

pub use visitors::VisitExt;

/// A "result type" that can be returned from a visitor. Visitors pick
/// an appropriate result type depending on what sort of operation they
/// are doing. A common choice is `FindAny`, which indicates that the visitor
/// is searching for something and that the visitor should stop once it is found.
pub trait VisitResult: Sized {
    /// Creates a new visitor result.
    fn new() -> Self;

    /// Returns true if this result is "complete" and we can stop visiting any
    /// further parts of the term. This is used by `FindAny`, for example, to
    /// stop the search after a match has been found.
    fn return_early(&self) -> bool;

    /// Combines two visitor results.
    fn combine(self, other: Self) -> Self;

    /// Convenience helper for use in writing `Visitor` impls. Returns `self`
    /// if `return_early()` is true, but otherwise combines `self` with the
    /// result of invoking `op`.
    ///
    /// If you have a struct with two fields, `foo` and `bar`, you can
    /// thus write code like
    ///
    /// ```rust,ignore
    /// self.foo.visit_with(visitor, outer_binder)
    ///     .and_then(|| self.bar.visit_with(visitor, outer_binder))
    /// ```
    ///
    /// and `bar` will only be visited if necessary.
    fn and_then(self, op: impl FnOnce() -> Self) -> Self {
        if self.return_early() {
            self
        } else {
            self.combine(op())
        }
    }
}

/// Unit type for a visitor indicates a "side-effecting" visitor that
/// should visit an entire term.
impl VisitResult for () {
    fn new() -> Self {}

    fn return_early(&self) -> bool {
        false
    }
    fn combine(self, _other: Self) {}
}

/// A "visitor" recursively folds some term -- that is, some bit of IR,
/// such as a `Goal`, and computes a value as a result.
///
///
/// To **apply** a visitor, use the `Visit::visit_with` method, like so
///
/// ```rust,ignore
/// let result = x.visit_with(&mut visitor, 0);
/// ```
pub trait Visitor<'i, I: Interner>
where
    I: 'i,
{
    /// The type of result that this visitor produces.
    type Result: VisitResult;

    /// Creates a `dyn` value from this visitor. Unfortunately, this
    /// must be added manually to each impl of visitor; it permits the
    /// default implements below to create a `&mut dyn Visitor` from
    /// `Self` without knowing what `Self` is (by invoking this
    /// method). Effectively, this limits impls of `visitor` to types
    /// for which we are able to create a dyn value (i.e., not `[T]`
    /// types).
    fn as_dyn(&mut self) -> &mut dyn Visitor<'i, I, Result = Self::Result>;

    /// Top-level callback: invoked for each `Ty<I>` that is
    /// encountered when visiting. By default, invokes
    /// `super_visit_with`, which will in turn invoke the more
    /// specialized visiting methods below, like `visit_free_var`.
    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Self::Result {
        ty.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// Top-level callback: invoked for each `Lifetime<I>` that is
    /// encountered when visiting. By default, invokes
    /// `super_visit_with`, which will in turn invoke the more
    /// specialized visiting methods below, like `visit_free_var`.
    fn visit_lifetime(
        &mut self,
        lifetime: &Lifetime<I>,
        outer_binder: DebruijnIndex,
    ) -> Self::Result {
        lifetime.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// Top-level callback: invoked for each `Const<I>` that is
    /// encountered when visiting. By default, invokes
    /// `super_visit_with`, which will in turn invoke the more
    /// specialized visiting methods below, like `visit_free_var`.
    fn visit_const(&mut self, constant: &Const<I>, outer_binder: DebruijnIndex) -> Self::Result {
        constant.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// Invoked for every program clause. By default, recursively visits the goals contents.
    fn visit_program_clause(
        &mut self,
        clause: &ProgramClause<I>,
        outer_binder: DebruijnIndex,
    ) -> Self::Result {
        clause.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// Invoked for every goal. By default, recursively visits the goals contents.
    fn visit_goal(&mut self, goal: &Goal<I>, outer_binder: DebruijnIndex) -> Self::Result {
        goal.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// Invoked for each domain goal.
    fn visit_domain_goal(
        &mut self,
        domain_goal: &DomainGoal<I>,
        outer_binder: DebruijnIndex,
    ) -> Self::Result {
        domain_goal.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// If overridden to return true, then visiting will panic if a
    /// free variable is encountered. This should be done if free
    /// type/lifetime/const variables are not expected.
    fn forbid_free_vars(&self) -> bool {
        false
    }

    /// Invoked for `BoundVar` instances that are not bound
    /// within the type being visited over:
    fn visit_free_var(&mut self, bound_var: BoundVar, outer_binder: DebruijnIndex) -> Self::Result {
        if self.forbid_free_vars() {
            panic!(
                "unexpected free variable `{:?}` with outer binder {:?}",
                bound_var, outer_binder
            )
        } else {
            Self::Result::new()
        }
    }

    /// If overridden to return true, we will panic when a free
    /// placeholder type/lifetime is encountered.
    fn forbid_free_placeholders(&self) -> bool {
        false
    }

    /// Invoked for each occurrence of a placeholder type; these are
    /// used when we instantiate binders universally.
    fn visit_free_placeholder(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        if self.forbid_free_placeholders() {
            panic!("unexpected placeholder type `{:?}`", universe)
        } else {
            Self::Result::new()
        }
    }

    /// Invoked for each where clause.
    fn visit_where_clause(
        &mut self,
        where_clause: &WhereClause<I>,
        outer_binder: DebruijnIndex,
    ) -> Self::Result {
        where_clause.super_visit_with(self.as_dyn(), outer_binder)
    }

    /// If overridden to return true, inference variables will trigger
    /// panics when visited. Used when inference variables are
    /// unexpected.
    fn forbid_inference_vars(&self) -> bool {
        false
    }

    /// Invoked for each occurrence of a inference type; these are
    /// used when we instantiate binders universally.
    fn visit_inference_var(
        &mut self,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Self::Result {
        if self.forbid_inference_vars() {
            panic!("unexpected inference type `{:?}`", var)
        } else {
            Self::Result::new()
        }
    }

    /// Gets the visitor's interner.
    fn interner(&self) -> &'i I;
}

/// Applies the given `visitor` to a value, producing a visited result
/// of type `Visitor::Result`.
pub trait Visit<I: Interner>: Debug {
    /// Apply the given visitor `visitor` to `self`; `binders` is the
    /// number of binders that are in scope when beginning the
    /// visitor. Typically `binders` starts as 0, but is adjusted when
    /// we encounter `Binders<T>` in the IR or other similar
    /// constructs.
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i;
}

/// For types where "visit" invokes a callback on the `visitor`, the
/// `SuperVisit` trait captures the recursive behavior that visits all
/// the contents of the type.
pub trait SuperVisit<I: Interner>: Visit<I> {
    /// Recursively visits the type contents.
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i;
}

/// "visiting" a type invokes the `visit_ty` method on the visitor; this
/// usually (in turn) invokes `super_visit_ty` to visit the individual
/// parts.
impl<I: Interner> Visit<I> for Ty<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_ty(self, outer_binder)
    }
}

/// "Super visit" for a type invokes te more detailed callbacks on the type
impl<I> SuperVisit<I> for Ty<I>
where
    I: Interner,
{
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        match self.data(interner) {
            TyData::BoundVar(bound_var) => {
                if let Some(_) = bound_var.shifted_out_to(outer_binder) {
                    visitor.visit_free_var(*bound_var, outer_binder)
                } else {
                    R::new()
                }
            }
            TyData::Dyn(clauses) => clauses.visit_with(visitor, outer_binder),
            TyData::InferenceVar(var, _) => visitor.visit_inference_var(*var, outer_binder),
            TyData::Apply(apply) => apply.visit_with(visitor, outer_binder),
            TyData::Placeholder(ui) => visitor.visit_free_placeholder(*ui, outer_binder),
            TyData::Alias(proj) => proj.visit_with(visitor, outer_binder),
            TyData::Function(fun) => fun.visit_with(visitor, outer_binder),
        }
    }
}

impl<I: Interner> Visit<I> for Lifetime<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_lifetime(self, outer_binder)
    }
}

impl<I: Interner> SuperVisit<I> for Lifetime<I> {
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        match self.data(interner) {
            LifetimeData::BoundVar(bound_var) => {
                if let Some(_) = bound_var.shifted_out_to(outer_binder) {
                    visitor.visit_free_var(*bound_var, outer_binder)
                } else {
                    R::new()
                }
            }
            LifetimeData::InferenceVar(var) => visitor.visit_inference_var(*var, outer_binder),
            LifetimeData::Placeholder(universe) => {
                visitor.visit_free_placeholder(*universe, outer_binder)
            }
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

impl<I: Interner> Visit<I> for Const<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_const(self, outer_binder)
    }
}

impl<I: Interner> SuperVisit<I> for Const<I> {
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        match &self.data(interner).value {
            ConstValue::BoundVar(bound_var) => {
                if let Some(_) = bound_var.shifted_out_to(outer_binder) {
                    visitor.visit_free_var(*bound_var, outer_binder)
                } else {
                    R::new()
                }
            }
            ConstValue::InferenceVar(var) => visitor.visit_inference_var(*var, outer_binder),
            ConstValue::Placeholder(universe) => {
                visitor.visit_free_placeholder(*universe, outer_binder)
            }
            ConstValue::Concrete(_) => R::new(),
        }
    }
}

impl<I: Interner> Visit<I> for Goal<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_goal(self, outer_binder)
    }
}

impl<I: Interner> SuperVisit<I> for Goal<I> {
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        self.data(interner).visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for ProgramClause<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_program_clause(self, outer_binder)
    }
}

impl<I: Interner> Visit<I> for WhereClause<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_where_clause(self, outer_binder)
    }
}

impl<I: Interner> Visit<I> for DomainGoal<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visitor.visit_domain_goal(self, outer_binder)
    }
}
