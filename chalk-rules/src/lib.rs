#[macro_use]
extern crate chalk_macros;
#[macro_use]
extern crate failure;

use chalk_ir::*;
use chalk_rust_ir::*;
use std::sync::Arc;

pub mod clauses;
pub mod coherence;
pub mod wf;

pub trait RustIrSource {
    /// Convert to a dyn trait value representing `self`. This is a
    /// workaround for the lack of proper upcasting in Rust.
    fn as_dyn(&self) -> &dyn RustIrSource;

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
