//! Persistent state passed down between writers.
//!
//! This is essentially `InternalWriterState` and other things supporting that.
use core::hash::Hash;
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Result},
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::RustIrDatabase;
use chalk_ir::{interner::Interner, *};
use indexmap::IndexMap;
use itertools::Itertools;

/// Like a BoundVar, but with the debrujin index inverted so as to create a
/// canonical name we can use anywhere for each bound variable.
///
/// In BoundVar, the innermost bound variables have debrujin index `0`, and
/// each further out BoundVar has a debrujin index `1` higher.
///
/// In InvertedBoundVar, the outermost variables have inverted_debrujin_idx `0`,
/// and the innermost have their depth, not the other way around.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct InvertedBoundVar {
    /// The inverted debrujin index. Corresponds roughly to an inverted `DebrujinIndex::depth`.
    inverted_debrujin_idx: i64,
    /// The index within the debrujin index. Corresponds to `BoundVar::index`.
    within_idx: IndexWithinBinding,
}

impl Display for InvertedBoundVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "_{}_{}", self.inverted_debrujin_idx, self.within_idx)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum UnifiedId<I: Interner> {
    AdtId(I::InternedAdtId),
    DefId(I::DefId),
}

#[derive(Debug)]
pub struct IdAliasStore<T> {
    /// Map from the DefIds we've encountered to a u32 alias id unique to all ids
    /// the same name.
    aliases: IndexMap<T, u32>,
    /// Map from each name to the next unused u32 alias id.
    next_unused_for_name: BTreeMap<String, u32>,
}

impl<T> Default for IdAliasStore<T> {
    fn default() -> Self {
        IdAliasStore {
            aliases: IndexMap::default(),
            next_unused_for_name: BTreeMap::default(),
        }
    }
}

impl<T: Copy + Eq + Hash> IdAliasStore<T> {
    fn alias_for_id_name(&mut self, id: T, name: String) -> String {
        let next_unused_for_name = &mut self.next_unused_for_name;
        let alias = *self.aliases.entry(id).or_insert_with(|| {
            let next_unused: &mut u32 = next_unused_for_name.entry(name.clone()).or_default();
            let id = *next_unused;
            *next_unused += 1;
            id
        });
        // If there are no conflicts, keep the name the same so that we don't
        // need name-agnostic equality in display tests.
        if alias == 0 {
            name
        } else {
            format!("{}_{}", name, alias)
        }
    }
}

#[derive(Debug)]
struct IdAliases<I: Interner> {
    id_aliases: IdAliasStore<UnifiedId<I>>,
}

impl<I: Interner> Default for IdAliases<I> {
    fn default() -> Self {
        IdAliases {
            id_aliases: IdAliasStore::default(),
        }
    }
}

/// Writer state which persists across multiple writes.
///
/// Currently, this means keeping track of what IDs have been given what names,
/// including deduplication information.
///
/// This data is stored using interior mutability - clones will point to the same underlying
/// data.
///
/// Uses a separate type, `P`, for the database stored inside to account for
/// `Arc` or wrapping other storage mediums.
#[derive(Debug)]
pub struct WriterState<I, DB: ?Sized, P = DB>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    pub(super) db: P,
    id_aliases: Arc<Mutex<IdAliases<I>>>,
    _phantom: PhantomData<DB>,
}

impl<I, DB: ?Sized, P> Clone for WriterState<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB> + Clone,
    I: Interner,
{
    fn clone(&self) -> Self {
        WriterState {
            db: self.db.clone(),
            id_aliases: self.id_aliases.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I, DB: ?Sized, P> WriterState<I, DB, P>
where
    DB: RustIrDatabase<I>,
    P: Borrow<DB>,
    I: Interner,
{
    pub fn new(db: P) -> Self {
        WriterState {
            db,
            id_aliases: Arc::new(Mutex::new(IdAliases::default())),
            _phantom: PhantomData,
        }
    }

    /// Returns a new version of self containing a wrapped database which
    /// references the outer data.
    ///
    /// `f` will be run on the internal database, and the returned result will
    /// wrap the result from `f`. For consistency, `f` should always contain the
    /// given database, and must keep the same ID<->item relationships.
    pub(super) fn wrap_db_ref<'a, DB2: ?Sized, P2, F>(&'a self, f: F) -> WriterState<I, DB2, P2>
    where
        DB2: RustIrDatabase<I>,
        P2: Borrow<DB2>,
        // We need to pass in `&'a P` specifically to guarantee that the `&P`
        // can outlive the function body, and thus that it's safe to store `&P`
        // in `P2`.
        F: FnOnce(&'a P) -> P2,
    {
        WriterState {
            db: f(&self.db),
            id_aliases: self.id_aliases.clone(),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn db(&self) -> &DB {
        self.db.borrow()
    }
}

/// Writer state for a single write call, persistent only as long as necessary
/// to write a single item.
///
/// Stores things necessary for .
#[derive(Clone, Debug)]
pub(super) struct InternalWriterState<'a, I: Interner> {
    persistent_state: WriterState<I, dyn RustIrDatabase<I> + 'a, &'a dyn RustIrDatabase<I>>,
    indent_level: usize,
    debrujin_indices_deep: u32,
    // lowered_(inverted_debrujin_idx, index) -> src_correct_(inverted_debrujin_idx, index)
    remapping: Rc<BTreeMap<InvertedBoundVar, InvertedBoundVar>>,
    // the inverted_bound_var which maps to "Self"
    self_mapping: Option<InvertedBoundVar>,
}

type IndexWithinBinding = usize;

impl<'a, I: Interner> InternalWriterState<'a, I> {
    pub fn new<DB, P>(persistent_state: &'a WriterState<I, DB, P>) -> Self
    where
        DB: RustIrDatabase<I>,
        P: Borrow<DB>,
    {
        InternalWriterState {
            persistent_state: persistent_state
                .wrap_db_ref(|db| db.borrow() as &dyn RustIrDatabase<I>),
            indent_level: 0,
            debrujin_indices_deep: 0,
            remapping: Rc::new(BTreeMap::new()),
            self_mapping: None,
        }
    }

    pub(super) fn db(&self) -> &dyn RustIrDatabase<I> {
        self.persistent_state.db
    }

    pub(super) fn add_indent(&self) -> Self {
        InternalWriterState {
            indent_level: self.indent_level + 1,
            ..self.clone()
        }
    }

    pub(super) fn indent(&self) -> impl Display {
        std::iter::repeat("  ").take(self.indent_level).format("")
    }

    pub(super) fn alias_for_adt_id_name(&self, id: I::InternedAdtId, name: String) -> impl Display {
        self.persistent_state
            .id_aliases
            .lock()
            .unwrap()
            .id_aliases
            .alias_for_id_name(UnifiedId::AdtId(id), name)
    }

    pub(super) fn alias_for_id_name(&self, id: I::DefId, name: String) -> impl Display {
        self.persistent_state
            .id_aliases
            .lock()
            .unwrap()
            .id_aliases
            .alias_for_id_name(UnifiedId::DefId(id), name)
    }

    /// Adds a level of debrujin index, and possibly a "Self" parameter.
    ///
    /// This should be called whenever recursing into the value within a
    /// [`Binders`].
    ///
    /// If `self_binding` is `Some`, then it will introduce a new variable named
    /// `Self` with the within-debrujin index given within and the innermost
    /// debrujian index after increasing debrujin index.
    #[must_use = "this returns a new `InternalWriterState`, and does not modify the existing one"]
    pub(super) fn add_debrujin_index(&self, self_binding: Option<IndexWithinBinding>) -> Self {
        let mut new_state = self.clone();
        new_state.debrujin_indices_deep += 1;
        new_state.self_mapping = self_binding
            .map(|idx| new_state.indices_for_introduced_bound_var(idx))
            .or(self.self_mapping);
        new_state
    }

    /// Adds parameter remapping.
    ///
    /// Each of the parameters in `lowered_vars` will be mapped to its
    /// corresponding variable in `original_vars` when printed through the
    /// `InternalWriterState` returned from this method.
    ///
    /// `lowered_vars` and `original_vars` must have the same length.
    pub(super) fn add_parameter_mapping(
        &self,
        lowered_vars: impl Iterator<Item = InvertedBoundVar>,
        original_vars: impl Iterator<Item = InvertedBoundVar>,
    ) -> Self {
        let remapping = self
            .remapping
            .iter()
            .map(|(a, b)| (*a, *b))
            .chain(lowered_vars.zip(original_vars))
            .collect::<BTreeMap<_, _>>();

        InternalWriterState {
            remapping: Rc::new(remapping),
            ..self.clone()
        }
    }

    /// Inverts the debrujin index so as to create a canonical name we can
    /// anywhere for each bound variable.
    ///
    /// See [`InvertedBoundVar`][InvertedBoundVar].
    pub(super) fn invert_debrujin_idx(
        &self,
        debrujin_idx: u32,
        index: IndexWithinBinding,
    ) -> InvertedBoundVar {
        InvertedBoundVar {
            inverted_debrujin_idx: (self.debrujin_indices_deep as i64) - (debrujin_idx as i64),
            within_idx: index,
        }
    }

    pub(super) fn apply_mappings(&self, b: InvertedBoundVar) -> impl Display {
        let remapped = self.remapping.get(&b).copied().unwrap_or(b);
        if self.self_mapping == Some(remapped) {
            "Self".to_owned()
        } else {
            remapped.to_string()
        }
    }

    pub(super) fn indices_for_bound_var(&self, b: &BoundVar) -> InvertedBoundVar {
        self.invert_debrujin_idx(b.debruijn.depth(), b.index)
    }

    pub(super) fn indices_for_introduced_bound_var(
        &self,
        idx: IndexWithinBinding,
    ) -> InvertedBoundVar {
        // freshly introduced bound vars will always have debrujin index of 0,
        // they're always "innermost".
        self.invert_debrujin_idx(0, idx)
    }

    pub(super) fn display_bound_var(&self, b: &BoundVar) -> impl Display {
        self.apply_mappings(self.indices_for_bound_var(b))
    }

    pub(super) fn name_for_introduced_bound_var(&self, idx: IndexWithinBinding) -> impl Display {
        self.apply_mappings(self.indices_for_introduced_bound_var(idx))
    }

    pub(super) fn binder_var_indices<'b>(
        &'b self,
        binders: &'b VariableKinds<I>,
    ) -> impl Iterator<Item = InvertedBoundVar> + 'b {
        binders
            .iter(self.db().interner())
            .enumerate()
            .map(move |(idx, _param)| self.indices_for_introduced_bound_var(idx))
    }

    pub(super) fn binder_var_display<'b>(
        &'b self,
        binders: &'b VariableKinds<I>,
    ) -> impl Iterator<Item = String> + 'b {
        binders
            .iter(self.db().interner())
            .zip(self.binder_var_indices(binders))
            .map(move |(parameter, var)| match parameter {
                VariableKind::Ty(_) => format!("{}", self.apply_mappings(var)),
                VariableKind::Lifetime => format!("'{}", self.apply_mappings(var)),
                VariableKind::Const(_ty) => format!("const {}", self.apply_mappings(var)),
            })
    }
}
