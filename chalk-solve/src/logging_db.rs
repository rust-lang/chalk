//! Provides wrappers over `RustIrDatabase` which record used definitions and write
//! `.chalk` files containing those definitions.
use std::{
    borrow::Borrow,
    cmp::{Ord, Ordering},
    collections::BTreeSet,
    fmt::{self, Debug, Display},
    io::Write,
    marker::PhantomData,
    sync::Arc,
    sync::Mutex,
};

use crate::rust_ir::*;
use crate::{display, RustIrDatabase};
use chalk_ir::{interner::Interner, *};

/// Wraps another `RustIrDatabase` (`DB`) and records which definitions are
/// used.
///
/// A full .chalk file containing all used definitions can be recovered through
/// `LoggingRustIrDatabase`'s `Display` implementation.
///
/// Uses a separate type, `P`, for the database stored inside to account for
/// `Arc` or wrapping other storage mediums.
#[derive(Debug)]
pub struct LoggingRustIrDatabase<I, DB, P = DB>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    db: P,
    def_ids: Mutex<BTreeSet<RecordedItemId<I>>>,
    _phantom: PhantomData<DB>,
}

impl<I, DB, P> LoggingRustIrDatabase<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    pub fn new(db: P) -> Self {
        LoggingRustIrDatabase {
            db,
            def_ids: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<I, DB, P> Display for LoggingRustIrDatabase<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let def_ids = self.def_ids.lock().unwrap();
        display::write_items(f, self.db.borrow(), def_ids.iter().copied())
    }
}

impl<I, DB, P> LoggingRustIrDatabase<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    fn record(&self, id: impl Into<RecordedItemId<I>>) {
        self.def_ids.lock().unwrap().insert(id.into());
    }

    fn record_all<T, U>(&self, ids: T)
    where
        T: IntoIterator<Item = U>,
        U: Into<RecordedItemId<I>>,
    {
        self.def_ids
            .lock()
            .unwrap()
            .extend(ids.into_iter().map(Into::into));
    }
}

impl<I, DB, P> RustIrDatabase<I> for LoggingRustIrDatabase<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB> + Debug,
    I: Interner,
{
    fn custom_clauses(&self) -> Vec<chalk_ir::ProgramClause<I>> {
        self.db.borrow().custom_clauses()
    }

    fn associated_ty_data(
        &self,
        ty: chalk_ir::AssocTypeId<I>,
    ) -> Arc<crate::rust_ir::AssociatedTyDatum<I>> {
        let ty_datum = self.db.borrow().associated_ty_data(ty);
        self.record(ty_datum.trait_id);
        ty_datum
    }

    fn trait_datum(&self, trait_id: TraitId<I>) -> Arc<TraitDatum<I>> {
        self.record(trait_id);
        self.db.borrow().trait_datum(trait_id)
    }

    fn adt_datum(&self, adt_id: AdtId<I>) -> Arc<AdtDatum<I>> {
        self.record(adt_id);
        self.db.borrow().adt_datum(adt_id)
    }

    fn impl_datum(&self, impl_id: ImplId<I>) -> Arc<ImplDatum<I>> {
        self.record(impl_id);
        self.db.borrow().impl_datum(impl_id)
    }

    fn hidden_opaque_type(&self, id: OpaqueTyId<I>) -> Ty<I> {
        self.record(id);
        self.db.borrow().hidden_opaque_type(id)
    }

    fn associated_ty_value(
        &self,
        id: crate::rust_ir::AssociatedTyValueId<I>,
    ) -> Arc<crate::rust_ir::AssociatedTyValue<I>> {
        let value = self.db.borrow().associated_ty_value(id);
        self.record(value.impl_id);
        value
    }

    fn opaque_ty_data(&self, id: OpaqueTyId<I>) -> Arc<OpaqueTyDatum<I>> {
        self.record(id);
        self.db.borrow().opaque_ty_data(id)
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<I>,
        parameters: &[chalk_ir::GenericArg<I>],
    ) -> Vec<ImplId<I>> {
        self.record(trait_id);
        let impl_ids = self.db.borrow().impls_for_trait(trait_id, parameters);
        self.record_all(impl_ids.iter().copied());
        impl_ids
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<I>) -> Vec<ImplId<I>> {
        self.record(trait_id);
        self.db.borrow().local_impls_to_coherence_check(trait_id)
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId<I>, adt_id: AdtId<I>) -> bool {
        self.record(auto_trait_id);
        self.record(adt_id);
        self.db.borrow().impl_provided_for(auto_trait_id, adt_id)
    }

    fn well_known_trait_id(
        &self,
        well_known_trait: crate::rust_ir::WellKnownTrait,
    ) -> Option<TraitId<I>> {
        let trait_id = self.db.borrow().well_known_trait_id(well_known_trait);
        trait_id.map(|id| self.record(id));
        trait_id
    }

    fn program_clauses_for_env(
        &self,
        environment: &chalk_ir::Environment<I>,
    ) -> chalk_ir::ProgramClauses<I> {
        self.db.borrow().program_clauses_for_env(environment)
    }

    fn interner(&self) -> &I {
        self.db.borrow().interner()
    }

    fn trait_name(&self, trait_id: TraitId<I>) -> String {
        self.db.borrow().trait_name(trait_id)
    }

    fn adt_name(&self, adt_id: AdtId<I>) -> String {
        self.db.borrow().adt_name(adt_id)
    }

    fn assoc_type_name(&self, assoc_ty_id: AssocTypeId<I>) -> String {
        self.db.borrow().assoc_type_name(assoc_ty_id)
    }

    fn opaque_type_name(&self, opaque_ty_id: OpaqueTyId<I>) -> String {
        self.db.borrow().opaque_type_name(opaque_ty_id)
    }

    fn is_object_safe(&self, trait_id: TraitId<I>) -> bool {
        self.record(trait_id);
        self.db.borrow().is_object_safe(trait_id)
    }

    fn fn_def_datum(&self, fn_def_id: chalk_ir::FnDefId<I>) -> Arc<FnDefDatum<I>> {
        self.record(fn_def_id);
        self.db.borrow().fn_def_datum(fn_def_id)
    }

    fn fn_def_name(&self, fn_def_id: FnDefId<I>) -> String {
        self.db.borrow().fn_def_name(fn_def_id)
    }

    fn closure_kind(&self, closure_id: ClosureId<I>, substs: &Substitution<I>) -> ClosureKind {
        // TODO: record closure IDs
        self.db.borrow().closure_kind(closure_id, substs)
    }

    fn closure_inputs_and_output(
        &self,
        closure_id: ClosureId<I>,
        substs: &Substitution<I>,
    ) -> Binders<FnDefInputsAndOutputDatum<I>> {
        // TODO: record closure IDs
        self.db
            .borrow()
            .closure_inputs_and_output(closure_id, substs)
    }

    fn closure_upvars(&self, closure_id: ClosureId<I>, substs: &Substitution<I>) -> Binders<Ty<I>> {
        // TODO: record closure IDs
        self.db.borrow().closure_upvars(closure_id, substs)
    }

    fn closure_fn_substitution(
        &self,
        closure_id: ClosureId<I>,
        substs: &Substitution<I>,
    ) -> Substitution<I> {
        // TODO: record closure IDs
        self.db.borrow().closure_fn_substitution(closure_id, substs)
    }
}

/// Wraps a [`RustIrDatabase`], and, when dropped, writes out all used
/// definition to the given file.
///
/// Uses [`LoggingRustIrDatabase`] internally.
///
/// Uses a separate type, `P`, for the database stored inside to account for
/// `Arc` or wrapping other storage mediums.
pub struct WriteOnDropRustIrDatabase<I, W, DB, P = DB>
where
    I: Interner,
    W: Write,
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
{
    db: LoggingRustIrDatabase<I, DB, P>,
    write: W,
}

impl<I, W, DB, P> fmt::Debug for WriteOnDropRustIrDatabase<I, W, DB, P>
where
    I: Interner,
    W: Write,
    DB: RustIrDatabase<I>,
    P: Borrow<DB> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteOnDropRustIrDatabase")
            .field("db", &self.db)
            .field("write", &"<opaque>")
            .finish()
    }
}

impl<I, W, DB, P> WriteOnDropRustIrDatabase<I, W, DB, P>
where
    I: Interner,
    W: Write,
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
{
    pub fn new(db: P, write: W) -> Self {
        WriteOnDropRustIrDatabase {
            db: LoggingRustIrDatabase::new(db),
            write,
        }
    }

    pub fn from_logging_db(db: LoggingRustIrDatabase<I, DB, P>, write: W) -> Self {
        WriteOnDropRustIrDatabase { db, write }
    }
}

impl<I, W, DB, P> Drop for WriteOnDropRustIrDatabase<I, W, DB, P>
where
    I: Interner,
    W: Write,
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
{
    fn drop(&mut self) {
        write!(self.write, "{}", self.db)
            .and_then(|_| self.write.flush())
            .expect("expected to be able to write rust ir database");
    }
}

impl<I, W, DB, P> RustIrDatabase<I> for WriteOnDropRustIrDatabase<I, W, DB, P>
where
    I: Interner,
    W: Write,
    DB: RustIrDatabase<I>,
    P: Borrow<DB> + Debug,
{
    fn custom_clauses(&self) -> Vec<chalk_ir::ProgramClause<I>> {
        self.db.borrow().custom_clauses()
    }

    fn associated_ty_data(
        &self,
        ty: chalk_ir::AssocTypeId<I>,
    ) -> Arc<crate::rust_ir::AssociatedTyDatum<I>> {
        self.db.borrow().associated_ty_data(ty)
    }

    fn trait_datum(&self, trait_id: TraitId<I>) -> Arc<TraitDatum<I>> {
        self.db.borrow().trait_datum(trait_id)
    }

    fn adt_datum(&self, adt_id: AdtId<I>) -> Arc<AdtDatum<I>> {
        self.db.borrow().adt_datum(adt_id)
    }

    fn impl_datum(&self, impl_id: ImplId<I>) -> Arc<ImplDatum<I>> {
        self.db.borrow().impl_datum(impl_id)
    }

    fn associated_ty_value(
        &self,
        id: crate::rust_ir::AssociatedTyValueId<I>,
    ) -> Arc<crate::rust_ir::AssociatedTyValue<I>> {
        self.db.borrow().associated_ty_value(id)
    }

    fn opaque_ty_data(&self, id: OpaqueTyId<I>) -> Arc<OpaqueTyDatum<I>> {
        self.db.borrow().opaque_ty_data(id)
    }

    fn hidden_opaque_type(&self, id: OpaqueTyId<I>) -> Ty<I> {
        self.db.borrow().hidden_opaque_type(id)
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<I>,
        parameters: &[chalk_ir::GenericArg<I>],
    ) -> Vec<ImplId<I>> {
        self.db.borrow().impls_for_trait(trait_id, parameters)
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<I>) -> Vec<ImplId<I>> {
        self.db.borrow().local_impls_to_coherence_check(trait_id)
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId<I>, adt_id: AdtId<I>) -> bool {
        self.db.borrow().impl_provided_for(auto_trait_id, adt_id)
    }

    fn well_known_trait_id(
        &self,
        well_known_trait: crate::rust_ir::WellKnownTrait,
    ) -> Option<TraitId<I>> {
        self.db.borrow().well_known_trait_id(well_known_trait)
    }

    fn program_clauses_for_env(
        &self,
        environment: &chalk_ir::Environment<I>,
    ) -> chalk_ir::ProgramClauses<I> {
        self.db.borrow().program_clauses_for_env(environment)
    }

    fn interner(&self) -> &I {
        self.db.borrow().interner()
    }

    fn is_object_safe(&self, trait_id: TraitId<I>) -> bool {
        self.db.borrow().is_object_safe(trait_id)
    }

    fn trait_name(&self, trait_id: TraitId<I>) -> String {
        self.db.borrow().trait_name(trait_id)
    }

    fn adt_name(&self, adt_id: AdtId<I>) -> String {
        self.db.borrow().adt_name(adt_id)
    }

    fn assoc_type_name(&self, assoc_ty_id: AssocTypeId<I>) -> String {
        self.db.borrow().assoc_type_name(assoc_ty_id)
    }

    fn opaque_type_name(&self, opaque_ty_id: OpaqueTyId<I>) -> String {
        self.db.borrow().opaque_type_name(opaque_ty_id)
    }

    fn fn_def_datum(&self, fn_def_id: chalk_ir::FnDefId<I>) -> Arc<FnDefDatum<I>> {
        self.db.borrow().fn_def_datum(fn_def_id)
    }

    fn fn_def_name(&self, fn_def_id: FnDefId<I>) -> String {
        self.db.borrow().fn_def_name(fn_def_id)
    }

    fn closure_kind(&self, closure_id: ClosureId<I>, substs: &Substitution<I>) -> ClosureKind {
        // TODO: record closure IDs
        self.db.borrow().closure_kind(closure_id, substs)
    }

    fn closure_inputs_and_output(
        &self,
        closure_id: ClosureId<I>,
        substs: &Substitution<I>,
    ) -> Binders<FnDefInputsAndOutputDatum<I>> {
        self.db
            .borrow()
            .closure_inputs_and_output(closure_id, substs)
    }

    fn closure_upvars(&self, closure_id: ClosureId<I>, substs: &Substitution<I>) -> Binders<Ty<I>> {
        self.db.borrow().closure_upvars(closure_id, substs)
    }

    fn closure_fn_substitution(
        &self,
        closure_id: ClosureId<I>,
        substs: &Substitution<I>,
    ) -> Substitution<I> {
        self.db.borrow().closure_fn_substitution(closure_id, substs)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RecordedItemId<I: Interner> {
    Adt(AdtId<I>),
    Trait(TraitId<I>),
    Impl(ImplId<I>),
    OpaqueTy(OpaqueTyId<I>),
    FnDef(FnDefId<I>),
}

impl<I: Interner> From<AdtId<I>> for RecordedItemId<I> {
    fn from(v: AdtId<I>) -> Self {
        RecordedItemId::Adt(v)
    }
}

impl<I: Interner> From<TraitId<I>> for RecordedItemId<I> {
    fn from(v: TraitId<I>) -> Self {
        RecordedItemId::Trait(v)
    }
}

impl<I: Interner> From<ImplId<I>> for RecordedItemId<I> {
    fn from(v: ImplId<I>) -> Self {
        RecordedItemId::Impl(v)
    }
}

impl<I: Interner> From<OpaqueTyId<I>> for RecordedItemId<I> {
    fn from(v: OpaqueTyId<I>) -> Self {
        RecordedItemId::OpaqueTy(v)
    }
}

impl<I: Interner> From<FnDefId<I>> for RecordedItemId<I> {
    fn from(v: FnDefId<I>) -> Self {
        RecordedItemId::FnDef(v)
    }
}

/// Utility for implementing Ord for RecordedItemId.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum OrderedItemId<'a, DefId, AdtId> {
    DefId(&'a DefId),
    AdtId(&'a AdtId),
}

impl<I: Interner> RecordedItemId<I> {
    /// Extract internal identifier. Allows for absolute ordering matching the
    /// order in which chalk saw things (and thus reproducing that order in
    /// printed programs)
    fn ordered_item_id(&self) -> OrderedItemId<'_, I::DefId, I::InternedAdtId> {
        match self {
            RecordedItemId::Trait(TraitId(x))
            | RecordedItemId::Impl(ImplId(x))
            | RecordedItemId::OpaqueTy(OpaqueTyId(x))
            | RecordedItemId::FnDef(FnDefId(x)) => OrderedItemId::DefId(x),
            RecordedItemId::Adt(AdtId(x)) => OrderedItemId::AdtId(x),
        }
    }
}

impl<I: Interner> PartialOrd for RecordedItemId<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Interner> Ord for RecordedItemId<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ordered_item_id().cmp(&other.ordered_item_id())
    }
}
