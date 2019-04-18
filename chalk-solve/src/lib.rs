use chalk_ir::*;
use chalk_rust_ir::*;
use std::fmt::Debug;
use std::sync::Arc;

#[macro_use]
extern crate chalk_macros;
#[macro_use]
extern crate failure;

pub mod clauses;
pub mod coherence;
mod coinductive_goal;
pub mod ext;
mod infer;
mod solve;
pub mod wf;

pub trait RustIrDatabase {
    /// Returns the datum for the associated type with the given id.
    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum>;

    /// Returns the datum for the impl with the given id.
    fn trait_datum(&self, trait_id: TraitId) -> Arc<TraitDatum>;

    /// Returns the datum for the impl with the given id.
    fn struct_datum(&self, struct_id: StructId) -> Arc<StructDatum>;

    /// Returns the datum for the impl with the given id.
    fn impl_datum(&self, impl_id: ImplId) -> Arc<ImplDatum>;

    /// Returns all the impls for a given trait.
    fn impls_for_trait(&self, trait_id: TraitId) -> Vec<ImplId>;

    /// Returns true if there is an explicit impl of the auto trait
    /// `auto_trait_id` for the struct `struct_id`. This is part of
    /// the auto trait handling -- if there is no explicit impl given
    /// by the user for the struct, then we provide default impls
    /// based on the field types (otherwise, we rely on the impls the
    /// user gave).
    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool;

    /// Returns the name for the type with the given id.
    fn type_name(&self, id: TypeKindId) -> Identifier;

    /// Given a projection of an associated type, splits the type
    /// parameters into two parts: those that come from the trait, and
    /// those that come from the associated type itself.
    ///
    /// e.g. given a projection `<Foo as Iterable>::Item<'x>`, where `Iterable` is defined like so:
    ///
    /// ```ignore
    /// trait Iterable { type Item<'a>; }
    /// ```
    ///
    /// we would split into the type parameter lists `[Foo]` (from the
    /// trait) and `['x]` (from the type).
    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy,
    ) -> (Arc<AssociatedTyDatum>, &'p [Parameter], &'p [Parameter]);
}

/// The trait for defining the program clauses that are in scope when
/// solving a goal.
pub trait ChalkSolveDatabase: RustIrDatabase + Debug {
    /// Returns a set of program clauses that could possibly match
    /// `goal`. This can be any superset of the correct set, but the
    /// more precise you can make it, the more efficient solving will
    /// be.
    fn program_clauses_that_could_match(&self, goal: &DomainGoal, vec: &mut Vec<ProgramClause>);
}

pub use solve::Solution;
pub use solve::Solver;
pub use solve::SolverChoice;
pub use solve::TestSolver;
