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

use ir::*;
use ir::could_match::CouldMatch;
use solve::infer::InferenceTable;
use solve::truncate::{truncate, Truncated};
use stacker;
use std::collections::HashSet;
use std::cmp::min;
use std::sync::Arc;
use std::usize;

mod aggregate;
crate mod on_demand;
mod resolvent;
mod simplify;

index_struct! {
    struct TableIndex {
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

copy_fold!(DepthFirstNumber);

/// The paper describes these as `A :- D | G`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ExClause {
    /// The substitution which, applied to the goal of our table,
    /// would yield A.
    subst: Substitution,

    /// Delayed literals: things that we depend on negatively,
    /// but which have not yet been fully evaluated.
    delayed_literals: Vec<DelayedLiteral>,

    /// Region constraints we have accumulated.
    constraints: Vec<InEnvironment<Constraint>>,

    /// Subgoals: literals that must be proven
    subgoals: Vec<Literal>,
}

struct_fold!(ExClause {
    subst,
    delayed_literals,
    constraints,
    subgoals,
});

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SimplifiedAnswers {
    answers: Vec<SimplifiedAnswer>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SimplifiedAnswer {
    /// A fully instantiated version of the goal for which the query
    /// is true (including region constraints).
    subst: CanonicalConstrainedSubst,

    /// If this flag is set, then the answer could be neither proven
    /// nor disproven. In general, the existence of a non-empty set of
    /// delayed literals simply means the answer's status is UNKNOWN,
    /// either because the size of the answer exceeded `max_size` or
    /// because of a negative loop (e.g., `P :- not { P }`).
    ambiguous: bool,
}

#[derive(Clone, Debug)]
enum DelayedLiteralSets {
    /// Corresponds to a single, empty set.
    None,

    /// Some (non-zero) number of non-empty sets.
    Some(HashSet<DelayedLiteralSet>),
}

/// A set of delayed literals. The vector in this struct must
/// be sorted, ensuring that we don't have to worry about permutations.
///
/// (One might expect delayed literals to always be ground, since
/// non-ground negative literals result in flounded
/// executions. However, due to the approximations introduced via RR
/// to ensure termination, it *is* in fact possible for delayed goals
/// to contain free variables. For example, what could happen is that
/// we get back an approximated answer with `Goal::CannotProve` as a
/// delayed literal, which in turn forces its subgoal to be delayed,
/// and so forth. Therefore, we store canonicalized goals.)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct DelayedLiteralSet {
    delayed_literals: Vec<DelayedLiteral>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DelayedLiteral {
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
    Positive(TableIndex, CanonicalConstrainedSubst),
}

enum_fold!(DelayedLiteral[] { CannotProve(a), Negative(a), Positive(a, b) });

/// Either `A` or `~A`, where `A` is a `Env |- Goal`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Literal {
    Positive(InEnvironment<Goal>),
    Negative(InEnvironment<Goal>),
}

enum_fold!(Literal[] { Positive(a), Negative(a) });

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

#[derive(Copy, Clone, Debug)]
enum Satisfiable<T> {
    Yes(T),
    No,
}

type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
type CanonicalGoal = Canonical<InEnvironment<Goal>>;
type UCanonicalGoal = UCanonical<InEnvironment<Goal>>;

impl DelayedLiteralSets {
    fn is_empty(&self) -> bool {
        match *self {
            DelayedLiteralSets::None => true,
            DelayedLiteralSets::Some(_) => false,
        }
    }
}

impl DelayedLiteralSet {
    fn is_empty(&self) -> bool {
        self.delayed_literals.is_empty()
    }

    fn is_subset(&self, other: &DelayedLiteralSet) -> bool {
        self.delayed_literals
            .iter()
            .all(|elem| other.delayed_literals.binary_search(elem).is_ok())
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

impl ExClause {
    /// Used whenever we process an answer (whether new or cached) on
    /// a positive edge (the SLG POSITIVE RETURN operation). Truncates
    /// the resolvent (or factor) if it has grown too large.
    fn truncate_returned(self, infer: &mut InferenceTable, max_size: usize) -> ExClause {
        // DIVERGENCE
        //
        // In the original RR paper, truncation is only applied
        // when the result of resolution is a new answer (i.e.,
        // `ex_clause.subgoals.is_empty()`).  I've chosen to be
        // more aggressive here, precisely because or our extended
        // semantics for unification. In particular, unification
        // can insert new goals, so I fear that positive feedback
        // loops could still run indefinitely in the original
        // formulation. I would like to revise our unification
        // mechanism to avoid that problem, in which case this could
        // be tightened up to be more like the original RR paper.
        //
        // Still, I *believe* this more aggressive approx. should
        // not interfere with any of the properties of the
        // original paper. In particular, applying truncation only
        // when the resolvent has no subgoals seems like it is
        // aimed at giving us more times to eliminate this
        // ambiguous answer.

        match truncate(infer, max_size, &self.subst) {
            // No need to truncate? Just propagate the resolvent back.
            Truncated {
                overflow: false, ..
            } => self,

            // Resolvent got too large. Have to introduce approximation.
            Truncated {
                overflow: true,
                value: truncated_subst,
            } => {
                // DIVERGENCE
                //
                // In RR, `self.delayed_literals` would be
                // preserved. I have chosen to drop them. Keeping
                // them does allow for the possibility of
                // eliminating this answer if any of them turn out
                // to be satisfiable. However, it also introduces
                // an annoying edge case I didn't want to think
                // about -- one which, interestingly, the paper
                // did not discuss, which may indicate it is
                // impossible for some subtle reason. In
                // particular, a truncated delayed literal has a
                // sort of inverse semantics. i.e. if we convert
                // `Foo :- ~Bar(Rc<Rc<u32>>) |` to `Foo :-
                // ~Bar(Rc<X>), Unknown |`, then this could be
                // invalidated by an instance of `Bar(Rc<i32>)`,
                // which is irrelevant to the original
                // clause. (There is an additional annoyance,
                // which is that we may not have tried to solve
                // `Bar(Rc<X>)` at all.)

                ExClause {
                    subst: truncated_subst,
                    delayed_literals: vec![DelayedLiteral::CannotProve(())],
                    constraints: vec![],
                    subgoals: vec![],
                }
            }
        }
    }
}

/// Returns all clauses that are relevant to `goal`, either from
/// the environment or the program.
fn clauses(
    program: &Arc<ProgramEnvironment>,
    goal: &InEnvironment<DomainGoal>,
) -> Vec<ProgramClause> {
    let &InEnvironment {
        ref environment,
        ref goal,
    } = goal;

    let environment_clauses = environment
        .clauses
        .iter()
        .filter(|&env_clause| env_clause.could_match(goal))
        .map(|env_clause| env_clause.clone().into_program_clause());

    let program_clauses = program
        .program_clauses
        .iter()
        .filter(|clause| clause.could_match(goal))
        .cloned();

    environment_clauses.chain(program_clauses).collect()
}

/// Because we recurse so deeply, we rely on stacker to
/// avoid overflowing the stack.
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
