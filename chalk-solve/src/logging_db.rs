//! Provides wrappers over `RustIrDatabase` which record used definitions and write
//! `.chalk` files containing those definitions.
use std::{
    cmp::Ord, cmp::Ordering, collections::BTreeSet, fmt, fmt::Display, io::Write, sync::Arc,
    sync::Mutex,
};

use chalk_ir::{interner::Interner, ImplId, OpaqueTyId, StructId, TraitId};
use chalk_rust_ir::{ImplDatum, OpaqueTyDatum, StructDatum, TraitDatum};

use crate::{display, RustIrDatabase};
/// Wraps another `RustIrDatabase` (`DB`) and records which definitions are used.
///
/// A full .chalk file containing all used definitions can be recovered through
/// `LoggingRustIrDatabase`'s `Display` implementation.
#[derive(Debug)]
pub struct LoggingRustIrDatabase<I, DB>
where
    DB: RustIrDatabase<I>,
    I: Interner,
{
    db: DB,
    def_ids: Mutex<BTreeSet<RecordedItemId<I>>>,
}

impl<I, DB> LoggingRustIrDatabase<I, DB>
where
    DB: RustIrDatabase<I>,
    I: Interner,
{
    pub fn new(db: DB) -> Self {
        LoggingRustIrDatabase {
            db,
            def_ids: Default::default(),
        }
    }
}

impl<I, DB> Display for LoggingRustIrDatabase<I, DB>
where
    DB: RustIrDatabase<I>,
    I: Interner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let def_ids = self.def_ids.lock().unwrap();
        display::write_program(f, &self.db, def_ids.iter().copied())
    }
}

impl<I, DB> LoggingRustIrDatabase<I, DB>
where
    DB: RustIrDatabase<I>,
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

impl<I, DB> RustIrDatabase<I> for LoggingRustIrDatabase<I, DB>
where
    DB: RustIrDatabase<I>,
    I: Interner,
{
    fn custom_clauses(&self) -> Vec<chalk_ir::ProgramClause<I>> {
        self.db.custom_clauses()
    }
    fn associated_ty_data(
        &self,
        ty: chalk_ir::AssocTypeId<I>,
    ) -> Arc<chalk_rust_ir::AssociatedTyDatum<I>> {
        let ty_datum = self.db.associated_ty_data(ty);
        self.record(ty_datum.trait_id);
        ty_datum
    }
    fn trait_datum(&self, trait_id: TraitId<I>) -> Arc<TraitDatum<I>> {
        self.record(trait_id);
        self.db.trait_datum(trait_id)
    }
    fn struct_datum(&self, struct_id: StructId<I>) -> Arc<StructDatum<I>> {
        self.record(struct_id);
        self.db.struct_datum(struct_id)
    }
    fn impl_datum(&self, impl_id: ImplId<I>) -> Arc<ImplDatum<I>> {
        self.record(impl_id);
        self.db.impl_datum(impl_id)
    }
    fn associated_ty_value(
        &self,
        id: chalk_rust_ir::AssociatedTyValueId<I>,
    ) -> Arc<chalk_rust_ir::AssociatedTyValue<I>> {
        let value = self.db.associated_ty_value(id);
        self.record(value.impl_id);
        value
    }
    fn opaque_ty_data(&self, id: OpaqueTyId<I>) -> Arc<OpaqueTyDatum<I>> {
        self.record(id);
        self.db.opaque_ty_data(id)
    }
    fn impls_for_trait(
        &self,
        trait_id: TraitId<I>,
        parameters: &[chalk_ir::Parameter<I>],
    ) -> Vec<ImplId<I>> {
        self.record(trait_id);
        let impl_ids = self.db.impls_for_trait(trait_id, parameters);
        self.record_all(impl_ids.iter().copied());
        impl_ids
    }
    fn local_impls_to_coherence_check(&self, trait_id: TraitId<I>) -> Vec<ImplId<I>> {
        self.record(trait_id);
        self.db.local_impls_to_coherence_check(trait_id)
    }
    fn impl_provided_for(&self, auto_trait_id: TraitId<I>, struct_id: StructId<I>) -> bool {
        self.record(auto_trait_id);
        self.record(struct_id);
        self.db.impl_provided_for(auto_trait_id, struct_id)
    }
    fn well_known_trait_id(
        &self,
        well_known_trait: chalk_rust_ir::WellKnownTrait,
    ) -> Option<TraitId<I>> {
        let trait_id = self.db.well_known_trait_id(well_known_trait);
        trait_id.map(|id| self.record(id));
        trait_id
    }
    fn program_clauses_for_env(
        &self,
        environment: &chalk_ir::Environment<I>,
    ) -> chalk_ir::ProgramClauses<I> {
        self.db.program_clauses_for_env(environment)
    }
    fn interner(&self) -> &I {
        self.db.interner()
    }
    fn trait_name(&self, trait_id: TraitId<I>) -> String {
        self.db.trait_name(trait_id)
    }
    fn struct_name(&self, struct_id: StructId<I>) -> String {
        self.db.struct_name(struct_id)
    }
    fn identifier_name(&self, ident: &I::Identifier) -> String {
        self.db.identifier_name(ident)
    }
    fn is_object_safe(&self, trait_id: TraitId<I>) -> bool {
        self.record(trait_id);
        self.db.is_object_safe(trait_id)
    }
}

/// Wraps a [`RustIrDatabase`], and, when dropped, writes out all used
/// definition to the given file.
///
/// Uses [`LoggingRustIrDatabase`] internally.
pub struct WriteOnDropRustIrDatabase<I, DB, W>
where
    DB: RustIrDatabase<I>,
    I: Interner,
    W: Write,
{
    db: LoggingRustIrDatabase<I, DB>,
    write: W,
}

impl<I, DB, W> fmt::Debug for WriteOnDropRustIrDatabase<I, DB, W>
where
    I: Interner,
    DB: RustIrDatabase<I> + fmt::Debug,
    W: Write,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteOnDropRustIrDatabase")
            .field("db", &self.db)
            .field("write", &"<opaque>")
            .finish()
    }
}

impl<I, DB, W> WriteOnDropRustIrDatabase<I, DB, W>
where
    DB: RustIrDatabase<I>,
    I: Interner,
    W: Write,
{
    pub fn new(db: DB, write: W) -> Self {
        WriteOnDropRustIrDatabase {
            db: LoggingRustIrDatabase::new(db),
            write,
        }
    }

    pub fn from_logging_db(db: LoggingRustIrDatabase<I, DB>, write: W) -> Self {
        WriteOnDropRustIrDatabase { db, write }
    }
}

impl<I, DB, W> Drop for WriteOnDropRustIrDatabase<I, DB, W>
where
    DB: RustIrDatabase<I>,
    I: Interner,
    W: Write,
{
    fn drop(&mut self) {
        write!(self.write, "{}", self.db)
            .and_then(|_| self.write.flush())
            .expect("expected to be able to write rust ir database");
    }
}

impl<I, DB, W> RustIrDatabase<I> for WriteOnDropRustIrDatabase<I, DB, W>
where
    DB: RustIrDatabase<I>,
    W: Write,
    I: Interner,
{
    fn custom_clauses(&self) -> Vec<chalk_ir::ProgramClause<I>> {
        self.db.custom_clauses()
    }
    fn associated_ty_data(
        &self,
        ty: chalk_ir::AssocTypeId<I>,
    ) -> Arc<chalk_rust_ir::AssociatedTyDatum<I>> {
        self.db.associated_ty_data(ty)
    }
    fn trait_datum(&self, trait_id: TraitId<I>) -> Arc<TraitDatum<I>> {
        self.db.trait_datum(trait_id)
    }
    fn struct_datum(&self, struct_id: StructId<I>) -> Arc<StructDatum<I>> {
        self.db.struct_datum(struct_id)
    }
    fn impl_datum(&self, impl_id: ImplId<I>) -> Arc<ImplDatum<I>> {
        self.db.impl_datum(impl_id)
    }
    fn associated_ty_value(
        &self,
        id: chalk_rust_ir::AssociatedTyValueId<I>,
    ) -> Arc<chalk_rust_ir::AssociatedTyValue<I>> {
        self.db.associated_ty_value(id)
    }
    fn opaque_ty_data(&self, id: OpaqueTyId<I>) -> Arc<OpaqueTyDatum<I>> {
        self.db.opaque_ty_data(id)
    }
    fn impls_for_trait(
        &self,
        trait_id: TraitId<I>,
        parameters: &[chalk_ir::Parameter<I>],
    ) -> Vec<ImplId<I>> {
        self.db.impls_for_trait(trait_id, parameters)
    }
    fn local_impls_to_coherence_check(&self, trait_id: TraitId<I>) -> Vec<ImplId<I>> {
        self.db.local_impls_to_coherence_check(trait_id)
    }
    fn impl_provided_for(&self, auto_trait_id: TraitId<I>, struct_id: StructId<I>) -> bool {
        self.db.impl_provided_for(auto_trait_id, struct_id)
    }
    fn well_known_trait_id(
        &self,
        well_known_trait: chalk_rust_ir::WellKnownTrait,
    ) -> Option<TraitId<I>> {
        self.db.well_known_trait_id(well_known_trait)
    }
    fn program_clauses_for_env(
        &self,
        environment: &chalk_ir::Environment<I>,
    ) -> chalk_ir::ProgramClauses<I> {
        self.db.program_clauses_for_env(environment)
    }
    fn interner(&self) -> &I {
        self.db.interner()
    }
    fn trait_name(&self, trait_id: TraitId<I>) -> String {
        self.db.trait_name(trait_id)
    }
    fn struct_name(&self, struct_id: StructId<I>) -> String {
        self.db.struct_name(struct_id)
    }
    fn identifier_name(&self, ident: &I::Identifier) -> String {
        self.db.identifier_name(ident)
    }

    fn is_object_safe(&self, trait_id: TraitId<I>) -> bool {
        self.db.is_object_safe(trait_id)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RecordedItemId<I: Interner> {
    Struct(StructId<I>),
    Trait(TraitId<I>),
    Impl(ImplId<I>),
    OpaqueTy(OpaqueTyId<I>),
}

impl<I: Interner> From<StructId<I>> for RecordedItemId<I> {
    fn from(v: StructId<I>) -> Self {
        RecordedItemId::Struct(v)
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

impl<I: Interner> RecordedItemId<I> {
    /// Extract internal identifier. Allows for absolute ordering matching the
    /// order in which chalk saw things (and thus reproducing that order in
    /// printed programs)
    fn def_id(&self) -> &I::DefId {
        match self {
            RecordedItemId::Trait(TraitId(x))
            | RecordedItemId::Struct(StructId(x))
            | RecordedItemId::Impl(ImplId(x))
            | RecordedItemId::OpaqueTy(OpaqueTyId(x)) => x,
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
        self.def_id().cmp(other.def_id())
    }
}
