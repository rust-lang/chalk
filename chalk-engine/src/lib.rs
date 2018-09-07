//! An alternative solver based around the SLG algorithm, which
//! implements the well-formed semantics. This algorithm is very
//! closed based on the description found in the following paper,
//! which I will refer to in the comments as EWFS:
//!
//! > Efficient Top-Down Computation of Queries Under the Well-formed Semantics
//! > (Chen, Swift, and Warren; Journal of Logic Programming '95)
//!
//! However, to understand that paper, I would recommend first
//! starting with the following paper, which I will refer to in the
//! comments as NFTD:
//!
//! > A New Formulation of Tabled resolution With Delay
//! > (Swift; EPIA '99)
//!
//! In addition, I incorporated extensions from the following papers,
//! which I will refer to as SA and RR respectively, that
//! describes how to do introduce approximation when processing
//! subgoals and so forth:
//!
//! > Terminating Evaluation of Logic Programs with Finite Three-Valued Models
//! > Riguzzi and Swift; ACM Transactions on Computational Logic 2013
//! > (Introduces "subgoal abstraction", hence the name SA)
//! >
//! > Radial Restraint
//! > Grosof and Swift; 2013
//!
//! Another useful paper that gives a kind of high-level overview of
//! concepts at play is the following, which I will refer to as XSB:
//!
//! > XSB: Extending Prolog with Tabled Logic Programming
//! > (Swift and Warren; Theory and Practice of Logic Programming '10)
//!
//! While this code is adapted from the algorithms described in those
//! papers, it is not the same. For one thing, the approaches there
//! had to be extended to our context, and in particular to coping
//! with hereditary harrop predicates and our version of unification
//! (which produces subgoals). I believe those to be largely faithful
//! extensions. However, there are some other places where I
//! intentionally dieverged from the semantics as described in the
//! papers -- e.g. by more aggressively approximating -- which I
//! marked them with a comment DIVERGENCE. Those places may want to be
//! evaluated in the future.
//!
//! Glossary of other terms:
//!
//! - WAM: Warren abstract machine, an efficient way to evaluate Prolog programs.
//!   See <http://wambook.sourceforge.net/>.
//! - HH: Hereditary harrop predicates. What Chalk deals in.
//!   Popularized by Lambda Prolog.

#![feature(in_band_lifetimes)]
#![feature(step_trait)]
#![feature(non_modrs_mods)]

#[macro_use]
extern crate chalk_macros;

#[cfg(feature = "stack_protection")]
extern crate stacker;

extern crate rustc_hash;

use context::Context;
use rustc_hash::FxHashSet;
use std::cmp::min;
use std::usize;

pub mod context;
mod derived;
pub mod fallible;
pub mod forest;
pub mod hh;
mod logic;
mod simplify;
mod stack;
mod strand;
mod table;
mod tables;

index_struct! {
    pub struct TableIndex { // FIXME: pub b/c Fold
        value: usize,
    }
}

/// The StackIndex identifies the position of a table's goal in the
/// stack of goals that are actively being processed. Note that once a
/// table is completely evaluated, it may be popped from the stack,
/// and hence no longer have a stack index.
index_struct! {
    struct StackIndex {
        value: usize,
    }
}

/// The `DepthFirstNumber` (DFN) is a sequential number assigned to
/// each goal when it is first encountered. The naming (taken from
/// EWFS) refers to the idea that this number tracks the index of when
/// we encounter the goal during a depth-first traversal of the proof
/// tree.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DepthFirstNumber {
    value: u64,
}

/// The paper describes these as `A :- D | G`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExClause<C: Context> {
    /// The substitution which, applied to the goal of our table,
    /// would yield A.
    pub subst: C::Substitution,

    /// Delayed literals: things that we depend on negatively,
    /// but which have not yet been fully evaluated.
    pub delayed_literals: Vec<DelayedLiteral<C>>,

    /// Region constraints we have accumulated.
    pub constraints: Vec<C::RegionConstraint>,

    /// Subgoals: literals that must be proven
    pub subgoals: Vec<Literal<C>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SimplifiedAnswers<C: Context> {
    answers: Vec<SimplifiedAnswer<C>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimplifiedAnswer<C: Context> {
    /// A fully instantiated version of the goal for which the query
    /// is true (including region constraints).
    pub subst: C::CanonicalConstrainedSubst,

    /// If this flag is set, then the answer could be neither proven
    /// nor disproven. In general, the existence of a non-empty set of
    /// delayed literals simply means the answer's status is UNKNOWN,
    /// either because the size of the answer exceeded `max_size` or
    /// because of a negative loop (e.g., `P :- not { P }`).
    pub ambiguous: bool,
}

#[derive(Debug)]
struct DelayedLiteralSets<C: Context>(InnerDelayedLiteralSets<C>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum InnerDelayedLiteralSets<C: Context> {
    /// Corresponds to a single, empty set.
    None,

    /// Some (non-zero) number of non-empty sets.
    /// Must be a set of sets, but HashSets are not Hash so we manually ensure uniqueness.
    Some(Vec<DelayedLiteralSet<C>>),
}

/// A set of delayed literals.
///
/// (One might expect delayed literals to always be ground, since
/// non-ground negative literals result in flounded
/// executions. However, due to the approximations introduced via RR
/// to ensure termination, it *is* in fact possible for delayed goals
/// to contain free variables. For example, what could happen is that
/// we get back an approximated answer with `Goal::CannotProve` as a
/// delayed literal, which in turn forces its subgoal to be delayed,
/// and so forth. Therefore, we store canonicalized goals.)
#[derive(Clone, Debug, Default)]
struct DelayedLiteralSet<C: Context> {
    delayed_literals: FxHashSet<DelayedLiteral<C>>,
}

#[derive(Clone, Debug)]
pub enum DelayedLiteral<C: Context> {
    /// Something which can never be proven nor disproven. Inserted
    /// when truncation triggers; doesn't arise normally.
    CannotProve(()),

    /// We are blocked on a negative literal `~G`, where `G` is the
    /// goal of the given table. Because negative goals must always be
    /// ground, we don't need any other information.
    Negative(TableIndex),

    /// We are blocked on a positive literal `Li`; we found a
    /// **conditional** answer (the `CanonicalConstrainedSubst`) within the
    /// given table, but we have to come back later and see whether
    /// that answer turns out to be true.
    Positive(TableIndex, C::CanonicalConstrainedSubst),
}

/// Either `A` or `~A`, where `A` is a `Env |- Goal`.
#[derive(Clone, Debug)]
pub enum Literal<C: Context> { // FIXME: pub b/c fold
    Positive(C::GoalInEnvironment),
    Negative(C::GoalInEnvironment),
}

/// The `Minimums` structure is used to track the dependencies between
/// some item E on the evaluation stack. In particular, it tracks
/// cases where the success of E depends (or may depend) on items
/// deeper in the stack than E (i.e., with lower DFNs).
///
/// `positive` tracks the lowest index on the stack to which we had a
/// POSITIVE dependency (e.g. `foo(X) :- bar(X)`) -- meaning that in
/// order for E to succeed, the dependency must succeed. It is
/// initialized with the index of the predicate on the stack. So
/// imagine we have a stack like this:
///
///     // 0 foo(X)   <-- bottom of stack
///     // 1 bar(X)
///     // 2 baz(X)   <-- top of stack
///
/// In this case, `positive` would be initially 0, 1, and 2 for `foo`,
/// `bar`, and `baz` respectively. This reflects the fact that the
/// answers for `foo(X)` depend on the answers for `foo(X)`. =)
///
/// Now imagine that we had a clause `baz(X) :- foo(X)`, inducing a
/// cycle. In this case, we would update `positive` for `baz(X)` to be
/// 0, reflecting the fact that its answers depend on the answers for
/// `foo(X)`. Similarly, the minimum for `bar` would (eventually) be
/// updated, since it too transitively depends on `foo`. `foo` is
/// unaffected.
///
/// `negative` tracks the lowest index on the stack to which we had a
/// NEGATIVE dependency (e.g., `foo(X) :- not { bar(X) }`) -- meaning
/// that for E to succeed, the dependency must fail. This is initially
/// `usize::MAX`, reflecting the fact that the answers for `foo(X)` do
/// not depend on `not(foo(X))`. When negative cycles are encountered,
/// however, this value must be updated.
#[derive(Copy, Clone, Debug)]
struct Minimums {
    positive: DepthFirstNumber,
    negative: DepthFirstNumber,
}

impl<C: Context> DelayedLiteralSets<C> {
    fn singleton(set: DelayedLiteralSet<C>) -> Self {
        if set.is_empty() {
            DelayedLiteralSets(InnerDelayedLiteralSets::None)
        } else {
            DelayedLiteralSets(InnerDelayedLiteralSets::Some(vec![set]))
        }
    }

    /// Inserts the set if it is minimal in the family.
    /// Returns true iff the set was inserted.
    fn insert_if_minimal(&mut self, set: &DelayedLiteralSet<C>) -> bool {
        match self.0 {
            // The empty set is always minimal.
            InnerDelayedLiteralSets::None => false,
            // Are we inserting an empty set?
            InnerDelayedLiteralSets::Some(_) if set.is_empty() => {
                self.0 = InnerDelayedLiteralSets::None;
                true
            }
            InnerDelayedLiteralSets::Some(ref mut sets) => {
                // Look for a subset.
                if sets.iter().any(|set| set.is_subset(&set)) {
                    false
                } else {
                    // No subset therefore `set` is minimal, discard supersets and insert.
                    sets.retain(|set| !set.is_subset(set));
                    sets.push(set.clone());
                    true
                }
            }
        }
    }
}

impl<C: Context> DelayedLiteralSet<C> {
    fn is_empty(&self) -> bool {
        self.delayed_literals.is_empty()
    }

    fn is_subset(&self, other: &DelayedLiteralSet<C>) -> bool {
        self.delayed_literals
            .iter()
            .all(|elem| other.delayed_literals.contains(elem))
    }
}

impl Minimums {
    const MAX: Minimums = Minimums {
        positive: DepthFirstNumber::MAX,
        negative: DepthFirstNumber::MAX,
    };

    /// Update our fields to be the minimum of our current value
    /// and the values from other.
    fn take_minimums(&mut self, other: &Minimums) {
        self.positive = min(self.positive, other.positive);
        self.negative = min(self.negative, other.negative);
    }

    fn minimum_of_pos_and_neg(&self) -> DepthFirstNumber {
        min(self.positive, self.negative)
    }
}

impl DepthFirstNumber {
    const MIN: DepthFirstNumber = DepthFirstNumber { value: 0 };
    const MAX: DepthFirstNumber = DepthFirstNumber {
        value: ::std::u64::MAX,
    };

    fn next(&mut self) -> DepthFirstNumber {
        let value = self.value;
        assert!(value < ::std::u64::MAX);
        self.value += 1;
        DepthFirstNumber { value }
    }
}

/// Because we recurse so deeply, we rely on stacker to
/// avoid overflowing the stack.
#[cfg(feature = "stack_protection")]
fn maybe_grow_stack<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    // These numbers are somewhat randomly chosen to make tests work
    // well enough on my system. In particular, because we only test
    // for growing the stack in `new_clause`, a red zone of 32K was
    // insufficient to prevent stack overflow. - nikomatsakis
    stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, op)
}

#[cfg(not(feature = "stack_protection"))]
fn maybe_grow_stack<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    op()
}
