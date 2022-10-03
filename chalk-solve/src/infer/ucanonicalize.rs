use crate::debug_span;
use chalk_derive::FallibleTypeFolder;
use chalk_ir::fold::{TypeFoldable, TypeFolder};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::visit::{TypeVisitable, TypeVisitor};
use chalk_ir::*;
use std::ops::ControlFlow;

use super::InferenceTable;

impl<I: Interner> InferenceTable<I> {
    pub fn u_canonicalize<T>(interner: I, value0: &Canonical<T>) -> UCanonicalized<T>
    where
        T: Clone + HasInterner<Interner = I> + TypeFoldable<I> + TypeVisitable<I>,
        T: HasInterner<Interner = I>,
    {
        debug_span!("u_canonicalize", "{:#?}", value0);

        // First, find all the universes that appear in `value`.
        let mut universes = UniverseMap::new();

        for universe in value0.binders.iter(interner) {
            universes.add(*universe.skip_kind());
        }

        value0.value.visit_with(
            &mut UCollector {
                universes: &mut universes,
                interner,
            },
            DebruijnIndex::INNERMOST,
        );

        // Now re-map the universes found in value. We have to do this
        // in a second pass because it is only then that we know the
        // full set of universes found in the original value.
        let value1 = value0
            .value
            .clone()
            .try_fold_with(
                &mut UMapToCanonical {
                    universes: &universes,
                    interner,
                },
                DebruijnIndex::INNERMOST,
            )
            .unwrap();
        let binders = CanonicalVarKinds::from_iter(
            interner,
            value0
                .binders
                .iter(interner)
                .map(|pk| pk.map_ref(|&ui| universes.map_universe_to_canonical(ui).unwrap())),
        );

        UCanonicalized {
            quantified: UCanonical {
                universes: universes.num_canonical_universes(),
                canonical: Canonical {
                    value: value1,
                    binders,
                },
            },
            universes,
        }
    }
}

#[derive(Debug)]
pub struct UCanonicalized<T: HasInterner> {
    /// The canonicalized result.
    pub quantified: UCanonical<T>,

    /// A map between the universes in `quantified` and the original universes
    pub universes: UniverseMap,
}

pub trait UniverseMapExt {
    fn add(&mut self, universe: UniverseIndex);
    fn map_universe_to_canonical(&self, universe: UniverseIndex) -> Option<UniverseIndex>;
    fn map_universe_from_canonical(&self, universe: UniverseIndex) -> UniverseIndex;
    fn map_from_canonical<T, I>(&self, interner: I, canonical_value: &Canonical<T>) -> Canonical<T>
    where
        T: Clone + TypeFoldable<I> + HasInterner<Interner = I>,
        T: HasInterner<Interner = I>,
        I: Interner;
}
impl UniverseMapExt for UniverseMap {
    fn add(&mut self, universe: UniverseIndex) {
        if let Err(i) = self.universes.binary_search(&universe) {
            self.universes.insert(i, universe);
        }
    }

    /// Given a universe U that appeared in our original value, return
    /// the universe to use in the u-canonical value. This is done by
    /// looking for the index I of U in `self.universes`. We will
    /// return the universe with "counter" I. This effectively
    /// "compresses" the range of universes to things from
    /// `0..self.universes.len()`. If the universe is not present in the map,
    /// we return `None`.
    fn map_universe_to_canonical(&self, universe: UniverseIndex) -> Option<UniverseIndex> {
        self.universes
            .binary_search(&universe)
            .ok()
            .map(|index| UniverseIndex { counter: index })
    }

    /// Given a "canonical universe" -- one found in the
    /// `u_canonicalize` result -- returns the original universe that
    /// it corresponded to.
    fn map_universe_from_canonical(&self, universe: UniverseIndex) -> UniverseIndex {
        if universe.counter < self.universes.len() {
            self.universes[universe.counter]
        } else {
            // If this universe is out of bounds, we assume an
            // implicit `forall` binder, effectively, and map to a
            // "big enough" universe in the original space. See
            // comments on `map_from_canonical` for a detailed
            // explanation.
            let difference = universe.counter - self.universes.len();
            let max_counter = self.universes.last().unwrap().counter;
            let new_counter = max_counter + difference + 1;
            UniverseIndex {
                counter: new_counter,
            }
        }
    }

    /// Returns a mapped version of `value` where the universes have
    /// been translated from canonical universes into the original
    /// universes.
    ///
    /// In some cases, `value` may contain fresh universes that are
    /// not described in the original map. This occurs when we return
    /// region constraints -- for example, if we were to process a
    /// constraint like `for<'a> 'a == 'b`, where `'b` is an inference
    /// variable, that would generate a region constraint that `!2 ==
    /// ?0`. (This constraint is typically not, as it happens,
    /// satisfiable, but it may be, depending on the bounds on `!2`.)
    /// In effect, there is a "for all" binder around the constraint,
    /// but it is not represented explicitly -- only implicitly, by
    /// the presence of a U2 variable.
    ///
    /// If we encounter universes like this, which are "out of bounds"
    /// from our original set of universes, we map them to a distinct
    /// universe in the original space that is greater than all the
    /// other universes in the map. That is, if we encounter a
    /// canonical universe `Ux` where our canonical vector is (say)
    /// `[U0, U3]`, we would compute the difference `d = x - 2` and
    /// then return the universe `3 + d + 1`.
    ///
    /// The important thing is that we preserve (a) the relative order
    /// of universes, since that determines visibility, and (b) that
    /// the universe we produce does not correspond to any of the
    /// other original universes.
    fn map_from_canonical<T, I>(&self, interner: I, canonical_value: &Canonical<T>) -> Canonical<T>
    where
        T: Clone + TypeFoldable<I> + HasInterner<Interner = I>,
        T: HasInterner<Interner = I>,
        I: Interner,
    {
        debug_span!("map_from_canonical", ?canonical_value, universes = ?self.universes);

        let binders = canonical_value
            .binders
            .iter(interner)
            .map(|cvk| cvk.map_ref(|&universe| self.map_universe_from_canonical(universe)));

        let value = canonical_value
            .value
            .clone()
            .try_fold_with(
                &mut UMapFromCanonical {
                    interner,
                    universes: self,
                },
                DebruijnIndex::INNERMOST,
            )
            .unwrap();

        Canonical {
            binders: CanonicalVarKinds::from_iter(interner, binders),
            value,
        }
    }
}

/// The `UCollector` is a "no-op" in terms of the value, but along the
/// way it collects all universes that were found into a vector.
struct UCollector<'q, I> {
    universes: &'q mut UniverseMap,
    interner: I,
}

impl<I: Interner> TypeVisitor<I> for UCollector<'_, I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn visit_free_placeholder(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> ControlFlow<()> {
        self.universes.add(universe.ui);
        ControlFlow::Continue(())
    }

    fn forbid_inference_vars(&self) -> bool {
        true
    }

    fn interner(&self) -> I {
        self.interner
    }
}

#[derive(FallibleTypeFolder)]
struct UMapToCanonical<'q, I: Interner> {
    interner: I,
    universes: &'q UniverseMap,
}

impl<'i, I: Interner> TypeFolder<I> for UMapToCanonical<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn TypeFolder<I> {
        self
    }

    fn forbid_inference_vars(&self) -> bool {
        true
    }

    fn fold_free_placeholder_ty(
        &mut self,
        universe0: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Ty<I> {
        let ui = self
            .universes
            .map_universe_to_canonical(universe0.ui)
            .expect("Expected UCollector to encounter this universe");
        PlaceholderIndex {
            ui,
            idx: universe0.idx,
        }
        .to_ty(TypeFolder::interner(self))
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe0: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        let universe = self
            .universes
            .map_universe_to_canonical(universe0.ui)
            .expect("Expected UCollector to encounter this universe");

        PlaceholderIndex {
            ui: universe,
            idx: universe0.idx,
        }
        .to_lifetime(TypeFolder::interner(self))
    }

    fn fold_free_placeholder_const(
        &mut self,
        ty: Ty<I>,
        universe0: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Const<I> {
        let universe = self
            .universes
            .map_universe_to_canonical(universe0.ui)
            .expect("Expected UCollector to encounter this universe");

        PlaceholderIndex {
            ui: universe,
            idx: universe0.idx,
        }
        .to_const(TypeFolder::interner(self), ty)
    }

    fn interner(&self) -> I {
        self.interner
    }
}

#[derive(FallibleTypeFolder)]
struct UMapFromCanonical<'q, I: Interner> {
    interner: I,
    universes: &'q UniverseMap,
}

impl<'i, I: Interner> TypeFolder<I> for UMapFromCanonical<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn TypeFolder<I> {
        self
    }

    fn fold_free_placeholder_ty(
        &mut self,
        universe0: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Ty<I> {
        let ui = self.universes.map_universe_from_canonical(universe0.ui);
        PlaceholderIndex {
            ui,
            idx: universe0.idx,
        }
        .to_ty(TypeFolder::interner(self))
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        universe0: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Lifetime<I> {
        let universe = self.universes.map_universe_from_canonical(universe0.ui);
        PlaceholderIndex {
            ui: universe,
            idx: universe0.idx,
        }
        .to_lifetime(TypeFolder::interner(self))
    }

    fn forbid_inference_vars(&self) -> bool {
        true
    }

    fn interner(&self) -> I {
        self.interner
    }
}
