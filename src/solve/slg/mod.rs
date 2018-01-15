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

use cast::{Cast, Caster};
use ir::*;
use ir::could_match::CouldMatch;
use solve::infer::{InferenceTable, unify::UnificationResult};
use solve::truncate::{truncate, Truncated};
use stacker;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::cmp::min;
use std::iter;
use std::mem;
use std::ops::{Index, IndexMut, Range};
use std::sync::Arc;
use std::usize;
use zip::Zip;

mod aggregate;
mod resolvent;
mod test;

/// Finds all possible solutions to the given root goal in the context
/// of the given program, approximating if the size of solutions or
/// subqueries exceeds `max_size`.
///
/// If this returns `Ok`, a complete set of answers is returned, some
/// of which may be approximated. This can be converted into a
/// solution using the method `into_solution` on `Answers`.
///
/// If this returns `Err`, then the success or failure of the program
/// could not be interpreted due to some execution error (typically
/// involving negation of a term with unresolved existential variables
/// -- that is, a non-ground term -- which is called "floundering").
pub fn solve_root_goal(
    max_size: usize,
    program: &Arc<ProgramEnvironment>,
    root_goal: &UCanonicalGoal,
) -> Result<Answers, ExplorationError> {
    Forest::solve_root_goal(max_size, program, &root_goal)
}

/// The **FOREST** of evaluation tracks all the in-progress work.
/// Conceptually, it corresponds to the forest F described in NFTD,
/// however, we structure it more like the "table" described in EWFS.
/// In particular, we never materialize the forest and subgraphs
/// *directly*, instead keeping two bits of information:
///
/// - There is **table** for each tree with root node `A :- A` in the forest.
///   This table is indexed by the (canonical) root node A. It contains
///   the answers found so far, as well as links to nodes from other trees in the
///   forest that are still waiting for answeres.
/// - There is a **stack** of nodes `A :- G` from the forest. Roughly
///   speaking, this stack stores nodes in the forest which have not
///   yet been completely evaluated.
///   - Calling this is stack can be a bit misleading: although the
///     activity of the system is focused on the top of the stack, we
///     also will wind up doing things like producing a new answer
///     that feeds into a goal higher-up the stack. For example, we might
///     have a stack like the following (where the stack grows down):
///
///         // foo(X) :- bar(X), baz(X).
///         // bar(X) :- ...
///
///     Here, we see that `foo(X)` is waiting on a result from `bar(X)`. Let's
///     say we just found an answer, `bar(1)`. In that case, we would feed that answer
///     to `foo`, causing us to push a new stack entry:
///
///         // foo(X) :- bar(X), baz(X).
///         // bar(X) :- ...
///         // foo(X) :- baz(1).
///
///     `bar(X)` and the node on top of it in the stack are not really
///     related. (Indeed, coping with this is actually the source of
///     some complexity in the machine itself.)
struct Forest {
    infer: InferenceTable,
    program: Arc<ProgramEnvironment>,
    dfn: DepthFirstNumber,
    tables: Tables,
    stack: Stack,
    max_size: usize,
}

/// A unit type used to indicate that we have fully explored a
/// particular pathway.
struct FullyExplored;

/// The various kinds of errors we can encounter during exploration.
/// Note that these do not indicate **failed results** -- i.e, traits
/// not implemented. They also do not indicate the "third value" in
/// the WFS semantics. Rather they indicate that we could not figure
/// out the result for a given predicate in the WFS semantics (i.e.,
/// we could not prove, disprove, nor even find a definitive undefined
/// result).
#[derive(Debug)]
pub enum ExplorationError {
    /// Indicates that execution "flounded", meaning that it
    /// encountered a negative goal with unresolved variables.
    Floundered,

    /// We do not tolerate overly large goals along negative paths
    /// right now.
    NegativeOverflow,
}

/// The result of exploration: either we fully explored some subtree,
/// populating the result tables with answers, or else we encountered
/// some kind of exploration error along the way.
type ExplorationResult = ::std::result::Result<FullyExplored, ExplorationError>;

/// See `Forest`.
#[derive(Default)]
struct Tables {
    /// Maps from a canonical goal to the index of its table.
    table_indices: HashMap<CanonicalGoal, TableIndex>,

    /// Table: as described above, stores the key information for each
    /// tree in the forest.
    tables: Vec<Table>,
}

/// See `Forest`.
#[derive(Default)]
struct Stack {
    /// Stack: as described above, stores the in-progress goals.
    stack: Vec<StackEntry>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TableIndex {
    value: usize,
}

copy_fold!(TableIndex);

/// The StackIndex identifies the position of a table's goal in the
/// stack of goals that are actively being processed. Note that once a
/// table is completely evaluated, it may be popped from the stack,
/// and hence no longer have a stack index.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct StackIndex {
    value: usize,
}

copy_fold!(StackIndex);

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

struct StackEntry {
    /// The goal G from the stack entry `A :- G` represented here.
    table: TableIndex,

    /// The DFN of this computation.
    dfn: DepthFirstNumber,

    /// Tracks the dependencies of this stack entry on things beneath
    /// it in the stack. This field is updated "periodically",
    /// e.g. when a direct subgoal completes. Otherwise, the minimums
    /// for the active computation are tracked in a local variable
    /// that is threaded around.
    ///
    /// Note that this field is an over-approximation. As described in
    /// section 3.4.1 of EWFS, it actually stores the minimal
    /// dependencies of this stack entry **and anything on top of it
    /// in the stack**. In some cases, it can happen that this entry
    /// on the stack does not depend on the things on top of it, in
    /// which case the `link` is overapproximated -- this
    /// overapproximation reflects the fact that, because of the
    /// nature of a stack, we cannot in fact pop this entry until
    /// those other entries are popped, even though there is no
    /// *logical* dependency between us. This is the price we pay for
    /// using such a simple data structure.
    link: Minimums,
}

struct Table {
    /// The goal this table is trying to solve (also the key to look
    /// it up).
    table_goal: CanonicalGoal,

    /// Stores the answers that we have found thus far. For each head
    /// goal, we store a set of "delayed literals" instances. So, if
    /// the SLG algorithm would have computed (e.g.) `A :- ~B |` and
    /// `A :- ~C |` as answers, this would be stored as a `A -> {{B},
    /// {C}}` map entry.
    answers: HashMap<CanonicalConstrainedSubst, DelayedLiteralSets>,

    /// Stack entries waiting to hear about POSITIVE results from this
    /// table. This occurs when you have something like `foo(X) :-
    /// bar(X)`.
    positives: Vec<CanonicalPendingExClause>,

    /// Stack entries waiting to hear about NEGATIVE results from this
    /// table. This occurs when you have something like `foo(X) :- not
    /// bar(X)`.
    negatives: Vec<CanonicalPendingExClause>,

    /// Stores the index of this table on the stack. This is only
    /// `Some` until the table has been COMPLETELY EVALUATED -- i.e.,
    /// all possible answers have been found -- at which point it is
    /// set to `None`.
    depth: Option<StackIndex>,
}

/// A truth value in the WFS.
#[derive(Copy, Clone, Debug)]
enum TruthValue {
    /// Contains a finite proof.
    True,

    /// Contains no proof or an infinite proof.
    False,

    /// Participates in a negative cycle.
    ///
    /// Consider this: are `a` and `b` true if `a :- not b. b :- not a.`?
    Unknown,
}

/// A link between two tables, indicating that when an answer is
/// produced by one table, it should be fed into another table.
/// For example, if we have a clause like
///
/// ```notrust
/// foo(?X) :- bar(?X), baz(?X)
/// ```
///
/// then `foo` might insert a `PendingExClause` into the table for
/// `bar`, indicating that each value of `?X` could lead to an answer
/// for `foo` (if `baz(?X)` is true).
#[derive(Clone, Debug)]
struct PendingExClause {
    /// The `goal_depth` in the stack of `foo`, the blocked goal.
    /// Note that `foo` must always be in the stack, since it is
    /// actively awaiting an answer.
    goal_depth: StackIndex,

    /// Answer substitution that the pending ex-clause carries along
    /// with it. Maps from the free variables in the table goal to the
    /// final values they wind up with.
    subst: Substitution,

    /// The goal `bar(?X)` that `foo(?X)` was trying to solve;
    /// typically equal to the table-goal in which this pending
    /// ex-clause is contained, modulo the ordering of variables. This
    /// is not *always* true, however, because the table goal may have
    /// been truncated.
    selected_goal: InEnvironment<Goal>,

    /// Any delayed literals in the ex-clause we were solving
    /// when we blocked on `bar`.
    delayed_literals: Vec<DelayedLiteral>,

    /// Constraints accumulated thus far in the ex-clause we were solving.
    constraints: Vec<InEnvironment<Constraint>>,

    /// Further subgoals, like `baz(?X)`, that must be solved after `foo`.
    subgoals: Vec<Literal>,
}

struct_fold!(PendingExClause {
    goal_depth,
    subst,
    selected_goal,
    delayed_literals,
    constraints,
    subgoals,
});

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
pub struct Answers {
    pub answers: Vec<Answer>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Answer {
    /// A fully instantiated version of the goal for which the query
    /// is true (including region constraints).
    pub subst: CanonicalConstrainedSubst,

    /// If this flag is set, then the answer could be neither proven
    /// nor disproven. In general, the existence of a non-empty set of
    /// delayed literals simply means the answer's status is UNKNOWN,
    /// either because the size of the answer exceeded `max_size` or
    /// because of a negative loop (e.g., `P :- not { P }`).
    pub ambiguous: bool,
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Sign {
    Positive,
    Negative,
}

#[derive(Copy, Clone, Debug)]
enum Satisfiable<T> {
    Yes(T),
    No,
}

type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
type CanonicalGoal = Canonical<InEnvironment<Goal>>;
type CanonicalPendingExClause = Canonical<PendingExClause>;
type UCanonicalGoal = UCanonical<InEnvironment<Goal>>;

impl Forest {
    fn solve_root_goal(
        max_size: usize,
        program: &Arc<ProgramEnvironment>,
        root_goal: &UCanonicalGoal,
    ) -> Result<Answers, ExplorationError> {
        let program = program.clone();

        let mut forest = Forest {
            infer: InferenceTable::new(),
            dfn: DepthFirstNumber::MIN,
            program: program.clone(),
            tables: Tables::default(),
            stack: Stack::default(),
            max_size: max_size,
        };

        let root_goal = forest.infer.instantiate_universes(root_goal);

        let (root_table, root_table_depth) = forest.push_new_table(&root_goal, None, None);
        let mut minimums = forest.stack[root_table_depth].link;
        let subst = forest.infer.fresh_subst(&root_goal.binders);
        let instantiated_goal = root_goal.substitute(&subst);
        forest.subgoal(root_table_depth, instantiated_goal, subst, &mut minimums)?;
        Simplification::simplify(&mut forest.tables);
        Ok(forest.tables[root_table].export_answers())
    }

    /// Pushes a new goal onto the stack, creating a table entry in the process.
    fn push_new_table(
        &mut self,
        goal: &CanonicalGoal,
        positive_pending: Option<CanonicalPendingExClause>,
        negative_pending: Option<CanonicalPendingExClause>,
    ) -> (TableIndex, StackIndex) {
        let depth = self.stack.next_index();
        let dfn = self.dfn.next();
        let table = self.tables.insert(goal, depth);
        debug!(
            "push_new_table: depth {:?} is table {:?} with goal {:?}",
            depth,
            table,
            goal
        );
        self.tables[table].positives.extend(positive_pending);
        self.tables[table].negatives.extend(negative_pending);
        self.stack.push(table, dfn);
        (table, depth)
    }

    /// Creates an inference snapshot and executes `op`, rolling back
    /// the snapshot afterwards. This is generally safe to use in any
    /// context where we are doing exploring (hence the return type),
    /// since -- due to the nature of the EWFS algorithm -- any result
    /// that may escape a stack frame (e.g., by being stored in a
    /// table) is canonicalized first.
    fn snapshotted<F>(&mut self, op: F) -> ExplorationResult
    where
        F: FnOnce(&mut Self) -> ExplorationResult,
    {
        let snapshot = self.infer.snapshot();
        let result = op(self);
        self.infer.rollback_to(snapshot);
        result
    }

    /// This is SLG_SUBGOAL from EWFS. It is invoked when a new goal
    /// has been freshly pushed. We do a slight tweak to account for
    /// HH vs domain goals.
    fn subgoal(
        &mut self,
        goal_depth: StackIndex,
        goal: InEnvironment<Goal>,
        subst: Substitution,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        debug_heading!(
            "subgoal(goal_depth={:?}, goal={:?}, minimums={:?})",
            goal_depth,
            goal,
            minimums
        );

        // We want to consider two cases:
        //
        // - The goal is a domain goal. In that case, we will make N alternatives,
        //   one for each clause we can find.
        // - The goal is some other kind of HH goal. In that case, we will break it
        //   down into a product of literals, and create 1 alternative.

        let InEnvironment { environment, goal } = goal;
        match goal {
            Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => {
                let domain_goal = InEnvironment::new(&environment, domain_goal);
                let clauses = self.clauses(&domain_goal);

                for clause in clauses {
                    self.snapshotted(|this| {
                        debug!("program clause = {:#?}", clause);
                        match resolvent::resolvent_clause(
                            &mut this.infer,
                            &domain_goal,
                            &subst,
                            &clause.implication,
                        ) {
                            Satisfiable::No => Ok(FullyExplored),
                            Satisfiable::Yes(resolvent) => {
                                this.new_clause(goal_depth, resolvent, minimums)
                            }
                        }
                    })?;
                }
            }

            _ => {
                // `canonical_goal` is an HH goal. We can simplify it
                // into a series of *literals*, all of which must be
                // true. Thus, in EWFS terms, we are effectively
                // creating a single child of the `A :- A` goal that
                // is like `A :- B, C, D` where B, C, and D are the
                // simplified subgoals. You can think of this as
                // applying built-in "meta program clauses" that
                // reduce HH goals into Domain goals.
                let hh_goal = InEnvironment::new(&environment, goal);
                let ex_clause = match Self::simplify_hh_goal(&mut self.infer, subst, hh_goal) {
                    Satisfiable::Yes(ex_clause) => ex_clause,
                    Satisfiable::No => return Ok(FullyExplored), // now way to solve
                };

                self.new_clause(goal_depth, ex_clause, minimums)?;
            }
        }

        debug!(
            "subgoal: goal_depth={:?} minimums={:?}",
            goal_depth,
            minimums
        );
        self.complete(goal_depth, minimums)
    }


    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    fn simplify_hh_goal(
        infer: &mut InferenceTable,
        subst: Substitution,
        initial_goal: InEnvironment<Goal>,
    ) -> Satisfiable<ExClause> {
        let mut ex_clause = ExClause {
            subst,
            delayed_literals: vec![],
            constraints: vec![],
            subgoals: vec![],
        };

        // A stack of higher-level goals to process.
        let mut pending_goals = vec![initial_goal];

        while let Some(InEnvironment { environment, goal }) = pending_goals.pop() {
            match goal {
                Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                    let subgoal = infer.instantiate_binders_universally(&subgoal);
                    pending_goals.push(InEnvironment::new(&environment, *subgoal));
                }
                Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                    let subgoal = infer.instantiate_binders_existentially(&subgoal);
                    pending_goals.push(InEnvironment::new(&environment, *subgoal))
                }
                Goal::Implies(wc, subgoal) => {
                    let new_environment = &environment.add_clauses(wc);
                    pending_goals.push(InEnvironment::new(&new_environment, *subgoal));
                }
                Goal::And(subgoal1, subgoal2) => {
                    pending_goals.push(InEnvironment::new(&environment, *subgoal1));
                    pending_goals.push(InEnvironment::new(&environment, *subgoal2));
                }
                Goal::Not(subgoal) => {
                    let subgoal = (*subgoal).clone();
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(InEnvironment::new(&environment, subgoal)));
                }
                Goal::Leaf(LeafGoal::EqGoal(ref eq_goal)) => {
                    let UnificationResult { goals, constraints } = {
                        match infer.unify(&environment, &eq_goal.a, &eq_goal.b) {
                            Ok(v) => v,
                            Err(_) => return Satisfiable::No,
                        }
                    };

                    ex_clause.constraints.extend(constraints);
                    ex_clause
                        .subgoals
                        .extend(goals.into_iter().casted().map(Literal::Positive));
                }
                Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => {
                    let domain_goal = domain_goal.cast();
                    ex_clause.subgoals.push(Literal::Positive(
                        InEnvironment::new(&environment, domain_goal),
                    ));
                }
                Goal::CannotProve(()) => {
                    // You can think of `CannotProve` as a special
                    // goal that is only provable if `not {
                    // CannotProve }`. Trying to prove this, of
                    // course, will always create a negative cycle and
                    // hence a delayed literal that cannot be
                    // resolved.
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(InEnvironment::new(&environment, goal)));
                }
            }
        }

        Satisfiable::Yes(ex_clause)
    }

    /// Returns all clauses that are relevant to `goal`, either from
    /// the environment or the program.
    fn clauses(&mut self, goal: &InEnvironment<DomainGoal>) -> Vec<ProgramClause> {
        let &InEnvironment {
            ref environment,
            ref goal,
        } = goal;

        let environment_clauses = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .map(|env_clause| env_clause.clone().into_program_clause());

        let program_clauses = self.program
            .program_clauses
            .iter()
            .filter(|clause| clause.could_match(goal))
            .cloned();

        environment_clauses.chain(program_clauses).collect()
    }

    /// Pop off the next subgoal from `ex_clause` and try to solve
    /// it. Invoked when we have either just started a fresh goal (and
    /// selected a program clause) or when we have a new answer to a
    /// blocked goal that has just been incorporated.
    fn new_clause(
        &mut self,
        goal_depth: StackIndex,
        mut ex_clause: ExClause, // Contains both A and G together.
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        info_heading!(
            "new_clause(goal_depth={:?}, ex_clause={:?}, minimums={:?}",
            goal_depth,
            self.infer.normalize_deep(&ex_clause),
            minimums
        );

        maybe_grow_stack(|| {
            self.snapshotted(|this| {
                match ex_clause.subgoals.pop() {
                    // No goals left to prove: this is an answer.
                    None => this.answer(goal_depth, ex_clause, minimums),

                    // Positive goal.
                    Some(Literal::Positive(selected_goal)) => {
                        this.positive(goal_depth, ex_clause, selected_goal, minimums)
                    }

                    // Negative goal. EWFS checks for whether `selected_goal`
                    // is ground here. We push this check into `negative`.
                    Some(Literal::Negative(selected_goal)) => {
                        this.negative(goal_depth, ex_clause, selected_goal, minimums)
                    }
                }
            })
        })
    }

    /// Try to solve a positive selected literal.
    fn positive(
        &mut self,
        goal_depth: StackIndex,
        ex_clause: ExClause,
        selected_goal: InEnvironment<Goal>,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        debug_heading!(
            "positive(goal_depth={:?}, ex_clause={:?}, selected_goal={:?}, minimums={:?})",
            goal_depth,
            self.infer.normalize_deep(&ex_clause),
            selected_goal,
            minimums
        );

        // Subgoal abstraction: Rather than looking up the table for
        // `selected_goal` directly, first apply the truncation
        // function. This may introduce fresh variables, making the
        // goal that we are looking up more general, and forcing us to
        // reuse an existing table. For example, if we had a selected
        // goal of
        //
        //     // Vec<Vec<Vec<Vec<i32>>>>: Sized
        //
        // we might now produce a truncated goal of
        //
        //     // Vec<Vec<?T>>: Sized
        //
        // Obviously, the answer we are looking for -- if it exists -- will be
        // found amongst the answers of this new, truncated goal.
        //
        // Subtle point: Note that the **selected goal** remains
        // unchanged and will be carried over into the "pending
        // clause" for the positive link on the new subgoal. This
        // means that if our new, truncated subgoal produces
        // irrelevant answers (e.g., `Vec<Vec<u32>>: Sized`), they
        // will fail to unify with our selected goal, producing no
        // resolvent.
        let Truncated {
            overflow: _,
            value: truncated_subgoal,
        } = truncate(&mut self.infer, self.max_size, &selected_goal);

        // Check if we need to create a new table and (if so) stop.
        let subgoal_table = match self.select_goal(
            goal_depth,
            Sign::Positive,
            &ex_clause,
            &selected_goal,
            truncated_subgoal,
            minimums,
        )? {
            Some(t) => t,
            None => return Ok(FullyExplored),
        };

        let pending_ex_clause = self.pending_ex_clause(goal_depth, &ex_clause, &selected_goal);

        // A table for this entry already exists. We want to take
        // whatever answers we can find in the table -- bearing in
        // mind that the table may still be in the process of being
        // evaluated!
        if let Some(subgoal_depth) = self.tables[subgoal_table].depth {
            // If the table is not completely evaluated, then there is
            // a cycle.  We'll still use whatever answers have been
            // found so far, but we'll also register ourselves to
            // receive any new answers that will come later.
            self.tables[subgoal_table]
                .positives
                .push(pending_ex_clause.clone());
            self.update_lookup(goal_depth, subgoal_depth, Sign::Positive, minimums);
        }

        // Process the answers that have already been found one by
        // one.
        let new_ex_clauses: Vec<_> = {
            let infer = &mut self.infer;
            let subgoal_table_goal = &self.tables[subgoal_table].table_goal;
            self.tables[subgoal_table]
                .answers
                .iter()
                .filter_map(|(answer_subst, answer_delayed_literals)| {
                    // This is a bit subtle: when we incorporate the
                    // cached answer, we always use the
                    // `pending_ex_clause` as the starting point
                    // rather than `ex_clause`. This is because the
                    // pending ex-clause is canonicalized, and hence
                    // we wind up generating fresh copies of all the
                    // inference variables involved. In the case where
                    // there is more than one pending answer, this is
                    // important, because otherwise we can wind up
                    // "cross-contaminating" between answers here.
                    //
                    // Imagine that our ex-clause is `?T: Sour`, and
                    // we have two tabled answers `Lemon: Sour` and
                    // `Vingear: Sour` (this scenario btw is tested by
                    // `cached_answers1` and friends). When we
                    // incorporate the first cached answer, we wind up
                    // unifying `?T` with `Lemon` -- but we just store
                    // that in a vector, we don't immediately do anything with it.
                    //
                    // Then we try to incorporate the next cached answer,
                    // `Vinegar` -- but now we already have `?T = Lemon` as a constraint,
                    // so that (incorrectly) fails.
                    //
                    // There are other ways to fix this:
                    //
                    // - We might clone the answer set and then use
                    //   snapshots as we process each cached
                    //   answer. This is effectively what happens when
                    //   handling program clauses, except that in that
                    //   case no clone is needed, because the program
                    //   clauses are not stored in a structure that is
                    //   being updated during computation (unlike the
                    //   answer set).
                    //
                    // - We might alternatively clone our inference
                    //   table for each cached answer, so that `?T`
                    //   has distinct bindings on the different
                    //   paths. We would need to move `infer` from a
                    //   field to a parameter then.
                    //
                    // In both cases, we could incorporate persistent
                    // data structures to make those clones cheaper.
                    Self::incorporate_cached_answer(
                        infer,
                        &pending_ex_clause,
                        subgoal_table,
                        subgoal_table_goal,
                        answer_subst,
                        answer_delayed_literals,
                    ).yes()
                })
                .collect()
        };

        for (new_goal_depth, new_ex_clause) in new_ex_clauses {
            assert_eq!(new_goal_depth, goal_depth);
            let new_ex_clause = self.truncate_returned(new_ex_clause);
            self.new_clause(goal_depth, new_ex_clause, minimums)?;
        }

        Ok(FullyExplored)
    }

    /// Checks to see if a table exists for `instantiated_subgoal`. If
    /// so, returns `Ok(Some(index))` with its index. If not, creates the new
    /// table and starts solving it.
    ///
    /// If a new table is created, it will have a link to the
    /// ex-clause (`ex_clause`) being solved. This link will carry the
    /// current selected goal (`selected_goal`).  Typically,
    /// `selected_goal` and `instantiated_subgoal` are variants of one
    /// another, but due to subgoal abstraction (truncation) this may
    /// not *necessarily* be the case. See the callers of this
    /// function for detailed comments.
    ///
    /// # Parameters
    ///
    /// - `goal_depth`: depth of current goal that we are solving in the stack
    /// - `sign`: is the selected literal positive or negative
    /// - `ex_clause`: current X-clause we are solving
    /// - `selected_goal`: goal of current selected literal (unaltered by abstraction)
    /// - `instantiated_subgoal`: abstracted version of selected goal used for table lookup
    /// - `minimums`: minimums for current stack frame
    fn select_goal(
        &mut self,
        goal_depth: StackIndex,
        sign: Sign,
        ex_clause: &ExClause,
        selected_goal: &InEnvironment<Goal>,
        instantiated_subgoal: InEnvironment<Goal>,
        minimums: &mut Minimums,
    ) -> Result<Option<TableIndex>, ExplorationError> {
        debug_heading!(
            "select_goal(goal_depth={:?}, \
             sign={:?}, \
             selected_goal={:?}, \
             instantiated_subgoal={:?}, \
             minimums={:?})",
            goal_depth,
            sign,
            selected_goal,
            instantiated_subgoal,
            minimums
        );

        let (canonical_subgoal, subst) = self.infer
            .canonicalize(&instantiated_subgoal)
            .into_quantified_and_subst();
        debug!("selected_goal: canonical_subgoal={:?}", canonical_subgoal);

        // Check if we have an existing table. If yet, return it.
        if let Some(subgoal_table) = self.tables.index_of(&canonical_subgoal) {
            return Ok(Some(subgoal_table));
        }

        // Otherwise, create the new table, listing the current goal
        // as being pending. Then try to solve this new table.
        let pending_ex_clause = self.pending_ex_clause(goal_depth, ex_clause, selected_goal);
        let (positive_link, negative_link) = match sign {
            Sign::Positive => (Some(pending_ex_clause), None),
            Sign::Negative => (None, Some(pending_ex_clause)),
        };
        let (subgoal_table, subgoal_depth) =
            self.push_new_table(&canonical_subgoal, positive_link, negative_link);
        let mut subgoal_minimums = self.stack.top().link;
        self.subgoal(
            subgoal_depth,
            instantiated_subgoal,
            subst,
            &mut subgoal_minimums,
        )?;
        self.update_solution(
            goal_depth,
            subgoal_table,
            sign,
            minimums,
            &mut subgoal_minimums,
        );
        Ok(None)
    }

    /// Creates a `PendingExClause` representing the current node in the forest.
    ///
    /// # Parameters
    ///
    /// - `goal_depth` -- the depth of the suspended goal in the stack
    /// - `ex_clause` -- the thing we are trying to prove (`A |- G` in EWFS),
    ///   but with selected literal popped
    /// - `selected_goal` -- the selected literal. This could be either positive
    ///   or negative depending on context.
    fn pending_ex_clause(
        &mut self,
        goal_depth: StackIndex,
        ex_clause: &ExClause,
        selected_goal: &InEnvironment<Goal>,
    ) -> CanonicalPendingExClause {
        let parts = (
            &ex_clause.subst,
            &selected_goal,
            &ex_clause.delayed_literals,
            &ex_clause.constraints,
            &ex_clause.subgoals,
        );
        let canonical_parts = self.infer.canonicalize(&parts).quantified;
        canonical_parts.map(
            |(subst, selected_goal, delayed_literals, constraints, subgoals)| {
                PendingExClause {
                    goal_depth,
                    subst,
                    selected_goal,
                    delayed_literals,
                    constraints,
                    subgoals,
                }
            },
        )
    }

    fn negative(
        &mut self,
        goal_depth: StackIndex,
        ex_clause: ExClause,
        selected_goal: InEnvironment<Goal>,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        debug_heading!(
            "negative(goal_depth={:?}, ex_clause={:?}, selected_goal={:?}, minimums={:?})",
            goal_depth,
            self.infer.normalize_deep(&ex_clause),
            selected_goal,
            minimums
        );

        // First, we have to check that the selected negative literal
        // is ground, and invert any universally quantified variables.
        //
        // DIVERGENCE -- In the RR paper, to ensure completeness, they
        // permit non-ground negative literals, but only consider
        // them to succeed when the target table has no answers at
        // all. This is equivalent inverting those free existentials
        // into universals, as discussed in the comments of
        // `invert`. This is clearly *sound*, but the completeness is
        // a subtle point. In particular, it can cause **us** to reach
        // false conclusions, because e.g. given a program like
        // (selected left-to-right):
        //
        //     not { ?T: Copy }, ?T = Vec<u32>
        //
        // we would select `not { ?T: Copy }` first. For this goal to
        // succeed we would require that -- effectively -- `forall<T>
        // { not { T: Copy } }`, which clearly doesn't hold. (In the
        // terms of RR, we would require that the table for `?T: Copy`
        // has failed before we can continue.)
        //
        // In the RR paper, this is acceptable because they assume all
        // of their input programs are both **normal** (negative
        // literals are selected after positive ones) and **safe**
        // (all free variables in negative literals occur in positive
        // literals). It is plausible for us to guarantee "normal"
        // form, we can reorder clauses as we need. I suspect we can
        // guarantee safety too, but I have to think about it.
        //
        // For now, we opt for the safer route of terming such
        // executions as floundering, because I think our use of
        // negative goals is sufficiently limited we can get away with
        // it. The practical effect is that we will judge more
        // executions as floundering than we ought to (i.e., where we
        // could instead generate an (imprecise) result). As you can
        // see a bit later, we also diverge in some other aspects that
        // affect completeness when it comes to subgoal abstraction.
        let inverted_subgoal = match self.infer.invert(&selected_goal) {
            Some(g) => g,
            None => {
                return Err(ExplorationError::Floundered);
            }
        };

        // DIVERGENCE
        //
        // If the negative subgoal has grown so large that we would have
        // to truncate it, we currently just abort the computation
        // entirely. This is not necessary -- the SA paper makes no
        // such distinction, for example, and applies truncation equally
        // for positive/negative literals. However, there are some complications
        // that arise that I do not wish to deal with right now.
        //
        // Let's work through an example to show you what I
        // mean. Imagine we have this (negative) selected literal;
        // hence `selected_subgoal` will just be the inner part:
        //
        //     // not { Vec<Vec<Vec<Vec<i32>>>>: Sized }
        //     //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //     //       `selected_goal`
        //
        // (In this case, the `inverted_subgoal` would be the same,
        // since there are no free universal variables.)
        //
        // If truncation **doesn't apply**, we would go and lookup the
        // table for the selected goal (`Vec<Vec<..>>: Sized`) and see
        // whether it has any answers. If it does, and they are
        // definite, then this negative literal is false. We don't
        // really care even how many answers there are and so forth
        // (if the goal is ground, as in this case, there can be at
        // most one definite answer, but if there are universals, then
        // the inverted goal would have variables; even so, a single
        // definite answer suffices to show that the `not { .. }` goal
        // is false).
        //
        // Truncation muddies the water, because the table may
        // generate answers that are not relevant to our original,
        // untracted literal.  Suppose that we truncate the selected
        // goal to:
        //
        //     // Vec<Vec<T>: Sized
        //
        // Clearly this table will have some solutions that don't
        // apply to us.  e.g., `Vec<Vec<u32>>: Sized` is a solution to
        // this table, but that doesn't imply that `not {
        // Vec<Vec<Vec<..>>>: Sized }` is false.
        //
        // This can be made to work -- we carry along the original
        // selected goal when we establish links between tables, and
        // we could use that to screen the resulting answers. (There
        // are some further complications around the fact that
        // selected goal may contain universally quantified free
        // variables that have been inverted, as discussed in the
        // prior paragraph above.) I just didn't feel like dealing
        // with it yet.
        if truncate(&mut self.infer, self.max_size, &inverted_subgoal).overflow {
            return Err(ExplorationError::NegativeOverflow);
        }

        // Check if we need to create a new table and (if so) stop.
        let subgoal_table = match self.select_goal(
            goal_depth,
            Sign::Negative,
            &ex_clause,
            &selected_goal,
            inverted_subgoal,
            minimums,
        )? {
            Some(t) => t,
            None => return Ok(FullyExplored),
        };

        // If we already know that the subgoal is satisfiable, we can
        // stop now.
        if self.tables[subgoal_table].is_satisfiable() {
            return Ok(FullyExplored);
        }

        if let Some(subgoal_depth) = self.tables[subgoal_table].depth {
            // Not yet completely evaluated. Register ourselves as
            // having interest in negative solutions and stop for now.
            let pending_ex_clause = self.pending_ex_clause(goal_depth, &ex_clause, &selected_goal);
            self.tables[subgoal_table].negatives.push(pending_ex_clause);
            self.update_lookup(goal_depth, subgoal_depth, Sign::Negative, minimums);
            return Ok(FullyExplored);
        }

        // Proceed to the remaining subgoals.
        self.new_clause(goal_depth, ex_clause, minimums)
    }

    fn incorporate_cached_answer(
        infer: &mut InferenceTable,
        pending_ex_clause: &CanonicalPendingExClause,
        answer_table: TableIndex,
        answer_table_goal: &CanonicalGoal,
        answer_subst: &CanonicalConstrainedSubst,
        answer_delayed_literals: &DelayedLiteralSets,
    ) -> Satisfiable<(StackIndex, ExClause)> {
        debug!(
            "incorporate_cached_answer(answer_subst={:?},\
             \n    pending_ex_clause={:?}\
             \n    answer_table={:?},\
             \n    answer_table_goal={:?},\
             \n    answer_delayed_literals={:?})",
            answer_subst,
            pending_ex_clause,
            answer_table,
            answer_table_goal,
            answer_delayed_literals
        );

        match *answer_delayed_literals {
            DelayedLiteralSets::None => resolvent::resolvent_pending(
                infer,
                pending_ex_clause,
                answer_table_goal,
                answer_subst,
            ),
            DelayedLiteralSets::Some(_) => resolvent::factor_pending(
                infer,
                pending_ex_clause,
                answer_table,
                answer_table_goal,
                answer_subst,
            ),
        }
    }

    fn answer(
        &mut self,
        goal_depth: StackIndex,
        ex_clause: ExClause, // Contains both A and G together.
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        info_heading!(
            "answer(goal_depth={:?}, ex_clause={:?}, minimums={:?})",
            goal_depth,
            self.infer.normalize_deep(&ex_clause),
            minimums
        );

        let goal_table = self.stack[goal_depth].table;

        // Decompose the answer.
        let ExClause {
            subst,
            constraints,
            delayed_literals,
            subgoals,
        } = ex_clause;
        assert!(subgoals.is_empty());

        // Produce the canonical form of the answer.
        let answer_subst = self.infer
            .canonicalize(&ConstrainedSubst { subst, constraints })
            .quantified;
        debug!(
            "answer: goal_table={:?}, answer_subst={:?}",
            goal_table,
            answer_subst
        );

        // Convert the `DelayedLiterals` instance representing the set
        // of delayed literals from this ex-clause.
        let delayed_literals = {
            let mut delayed_literals: Vec<_> = delayed_literals.into_iter().collect();
            delayed_literals.sort();
            DelayedLiteralSet { delayed_literals }
        };
        debug!("answer: delayed_literals={:?}", delayed_literals);

        // (*) NB: delayed literals cannot have free inference variables

        // Determine if answer is new. If so, insert and notify.
        let list: Vec<_> = if delayed_literals.delayed_literals.is_empty() {
            debug!(
                "answer: no delayed_literals, previous answer = {:?}",
                self.tables[goal_table].answers.get(&answer_subst)
            );

            // If we already saw an answer with no delayed literals,
            // stop. Otherwise, continue.
            match self.tables[goal_table].answers.entry(answer_subst.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(DelayedLiteralSets::None);
                }

                Entry::Occupied(mut entry) => {
                    if let &DelayedLiteralSets::None = entry.get() {
                        return Ok(FullyExplored);
                    }

                    entry.insert(DelayedLiteralSets::None);
                }
            }

            // Clear out all the people waiting for negative results; we
            // have an answer now, so they have failed.
            self.tables[goal_table].negatives = vec![];

            // Produce a list of people waiting for *positive* results.
            let infer = &mut self.infer;
            let table_goal = &self.tables[goal_table].table_goal;
            self.tables[goal_table]
                .positives
                .iter()
                .filter_map(|p| {
                    resolvent::resolvent_pending(infer, p, table_goal, &answer_subst).yes()
                })
                .collect()
        } else {
            debug!(
                "answer: has delayed_literals, previous answer = {:?}",
                self.tables[goal_table].answers.get(&answer_subst)
            );

            if let Some(entry) = self.tables[goal_table].answers.get_mut(&answer_subst) {
                // Already have an entry with this head atom. No
                // need to notify pending people, since they don't
                // care about the details of what the delayed
                // literals are.

                match *entry {
                    DelayedLiteralSets::None => {
                        // We don't care about answers with
                        // delayed literals if we already had an
                        // unconditional answer.
                    }

                    DelayedLiteralSets::Some(ref mut v) => {
                        // We don't care about answers with
                        // delayed literals if we already had an
                        // unconditional answer.
                        v.insert(delayed_literals);
                    }
                }

                return Ok(FullyExplored);
            }

            // No entry yet with this head atom. We need to
            // notify pending people, so don't return.
            self.tables[goal_table].answers.insert(
                answer_subst.clone(),
                DelayedLiteralSets::Some(iter::once(delayed_literals).collect()),
            );

            let infer = &mut self.infer;
            let answer_table_goal = &self.tables[goal_table].table_goal;
            self.tables[goal_table]
                .positives
                .iter()
                .filter_map(|p| {
                    resolvent::factor_pending(
                        infer,
                        p,
                        goal_table,
                        answer_table_goal,
                        &answer_subst,
                    ).yes()
                })
                .collect()
        };

        // Process each of them in turn.
        for (pending_table, pending_ex_clause) in list {
            let pending_ex_clause = self.truncate_returned(pending_ex_clause);
            self.new_clause(pending_table, pending_ex_clause, minimums)?;
        }

        Ok(FullyExplored)
    }

    /// Updates `minimums` to account for the dependencies of a
    /// subgoal. Invoked when:
    ///
    /// - in the midst of solving `table`,
    /// - `subgoal_table` was the selected literal,
    /// - we invoked `subgoal()` and it returned,
    /// - with `subgoal_minimums` as its "result".
    fn update_solution(
        &mut self,
        goal_depth: StackIndex,
        subgoal_table: TableIndex,
        sign: Sign,
        minimums: &mut Minimums,
        subgoal_minimums: &Minimums,
    ) {
        debug!(
            "update_solution(goal_depth={:?}, subgoal_table={:?}, sign={:?}, \
             minimums={:?}, subgoal_minimums={:?})",
            goal_depth,
            subgoal_table,
            sign,
            minimums,
            subgoal_minimums
        );

        if let Some(subgoal_depth) = self.tables[subgoal_table].depth {
            self.update_lookup(goal_depth, subgoal_depth, sign, minimums);
        } else {
            self.stack[goal_depth].link.take_minimums(subgoal_minimums);
            minimums.take_minimums(subgoal_minimums);
        }
    }

    /// Like `update_solution`, but invoked when `subgoal_table`
    /// is known to be incomplete.
    fn update_lookup(
        &mut self,
        goal_depth: StackIndex,
        subgoal_depth: StackIndex,
        sign: Sign,
        minimums: &mut Minimums,
    ) {
        match sign {
            Sign::Positive => {
                let subgoal_link = self.stack[subgoal_depth].link;
                self.stack[goal_depth].link.take_minimums(&subgoal_link);
                minimums.take_minimums(&subgoal_link);
            }

            Sign::Negative => {
                // If `goal` depends on `not(subgoal)`, then for goal
                // to succeed, `subgoal` must be completely
                // evaluated. Therefore, `goal` depends (negatively)
                // on the minimum link of `subgoal` as a whole -- it
                // doesn't matter whether it's pos or neg.
                let subgoal_min = self.stack[subgoal_depth].link.minimum_of_pos_and_neg();
                self.stack[goal_depth]
                    .link
                    .take_negative_minimum(subgoal_min);
                minimums.take_negative_minimum(subgoal_min);
            }
        }
    }

    /// This method is invoked each time we exhaust all of the
    /// possibilities for exploration at the point in the stack
    /// (`goal_depth`). This doesn't mean that we are finished
    /// with the goal: for example, there may be a cycle, like
    ///
    /// ```notrust
    /// s :- p
    /// s :- ...
    /// p :- s // <-- when `complete` is invoked on `q`, we will not be done
    ///               exploring `s`
    /// ```
    fn complete(
        &mut self,
        completed_goal_depth: StackIndex,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        info_heading!(
            "complete(completed_goal_depth={:?}, minimums={:?})",
            completed_goal_depth,
            minimums
        );

        self.stack[completed_goal_depth]
            .link
            .take_minimums(minimums);

        // Here:
        // - `table` is the index of the table we are processing
        // - `dfn` is the depth-first number of the associated goal
        // - `link` summarizes the depth-first numbers of the goals that we transitively depend on
        let StackEntry {
            table: _,
            link,
            dfn,
        } = self.stack[completed_goal_depth];

        if link.positive == dfn && link.negative == DepthFirstNumber::MAX {
            self.complete_pop(completed_goal_depth, minimums)
        } else if link.positive == dfn && link.negative >= dfn {
            self.complete_delay(completed_goal_depth, minimums)
        } else {
            Ok(FullyExplored)
        }
    }

    /// Code to cover the sub-case of `complete` in which all of the
    /// goals that we depend upon lie atop us in the stack. Since all
    /// of *them* are completely evaluated, we are done. For example,
    /// imagine that we have this prolog program:
    ///
    /// ```notrust
    /// s :- p.
    /// p :- s.
    /// ```
    ///
    /// We would first push `s` onto the stack of goals with DFN 0,
    /// then `p` with DFN 1. When we finish exploring `p`, we would
    /// invoke `complete`, but it would have a *positive link* on
    /// `s`. This means that the `link.positive` for `q` would be
    /// 0, which is higher than `q`'s DFN of 1. Therefore, we would do nothing.
    /// But then we would invoke `complete` on `s` -- and the link for `s` is 0
    /// as is its DFN. In that case, this `if` is true.
    fn complete_pop(
        &mut self,
        completed_goal_depth: StackIndex,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        info!(
            "complete_pop(completed_goal_depth={:?}, minimums={:?}",
            completed_goal_depth,
            minimums
        );

        let completed_dfn = self.stack[completed_goal_depth].dfn;
        let popped_goals = self.stack.pop_until(completed_goal_depth);
        let mut new_clauses = vec![];
        for popped_goal in popped_goals.into_iter().rev() {
            let StackEntry {
                table: popped_table,
                link: popped_link,
                ..
            } = popped_goal;

            // None of the goals we pop should depend on anything
            // below the completed goal on the stack.
            assert!(
                popped_link.positive >= completed_dfn,
                "popped table {:?} with position link {:?} where completed_dfn = {:?}",
                popped_table,
                popped_link.positive,
                completed_dfn
            );

            // None of the popped goals should have negative dependencies.
            assert!(
                popped_link.negative == DepthFirstNumber::MAX,
                "popped table {:?} with negative link {:?} where completed_dfn = {:?}",
                popped_table,
                popped_link.negative,
                completed_dfn
            );

            // Take the list of negative goals. We will be updating those.
            let negatives = self.tables[popped_table].mark_complete();

            if self.tables[popped_table].is_not_satisfiable() {
                // If the set of answers is EMPTY, then this goal has
                // definitely FAILED. We can therefore go through the
                // list of clauses blocked negatively on this popped goal
                // and let them proceed.
                let unblocked = negatives.into_iter().map(|pending| {
                    let PendingExClause {
                        goal_depth,
                        subst,
                        selected_goal,
                        delayed_literals,
                        constraints,
                        subgoals,
                    } = self.infer.instantiate_canonical(&pending);
                    mem::drop(selected_goal); // don't need it
                    (
                        goal_depth,
                        ExClause {
                            subst,
                            delayed_literals,
                            constraints,
                            subgoals,
                        },
                    )
                });
                new_clauses.extend(unblocked);
            } else if self.tables[popped_table].is_satisfiable() {
                // We have a definitive answer. We can just
                // abandon the list of negative clauses then,
                // since they are all failed paths.
                //
                // DIVERGENCE: If we introduced subgoal abstraction
                // along negative paths,
            } else {
                // Otherwise, if we do not have a *definitive*
                // answer, then it is not yet known whether this
                // goal has succeeded or failed.  We must therefore
                // go through the list of clauses blocked
                // negatively on this popped goal and convert them
                // into DELAYED clauses.
                let delayed = Self::delay_negatives(&mut self.infer, popped_table, negatives);
                new_clauses.extend(delayed);
            }
        }

        *minimums = Minimums {
            positive: DepthFirstNumber::MAX,
            negative: DepthFirstNumber::MAX,
        };
        for (goal_depth, ex_clause) in new_clauses {
            self.new_clause(goal_depth, ex_clause, minimums)?;
        }
        Ok(FullyExplored)
    }

    fn complete_delay(
        &mut self,
        completed_goal_depth: StackIndex,
        minimums: &mut Minimums,
    ) -> ExplorationResult {
        info!(
            "complete_delay(completed_goal_depth={:?}, minimums={:?}",
            completed_goal_depth,
            minimums
        );

        let mut new_clauses;

        let top = self.stack.next_index();

        {
            let subgoals = self.stack.peek_until(completed_goal_depth);
            let tables = &mut self.tables;
            let len = subgoals
                .iter()
                .map(|g| tables[g.table].negatives.len())
                .sum();
            new_clauses = Vec::with_capacity(len);
            for subgoal in subgoals {
                // Take everything that depends on `subgoal` and convert those
                // depencies into delayed literals. In other words, if `subgoal` is `p`,
                // and we have some negative links arising from something like
                //
                // ```notrust
                // q :- ~p, r
                // ```
                //
                // we would remove the negative link and convert into
                // `q :- ~p | r`.

                let subtable = subgoal.table;
                let negatives = tables[subtable].take_negatives();
                subgoal.link.negative = DepthFirstNumber::MAX;
                new_clauses.extend(Self::delay_negatives(&mut self.infer, subtable, negatives));
            }
        }

        minimums.positive = self.stack[completed_goal_depth].dfn;
        minimums.negative = DepthFirstNumber::MAX;
        for (goal_depth, ex_clause) in new_clauses {
            self.new_clause(goal_depth, ex_clause, minimums)?;
        }

        // We've again completed all work on the things that were on
        // top of the stack. So `complete` them recursively.
        for table in {
            (completed_goal_depth.value..top.value)
                .map(|value| StackIndex { value })
                .rev()
        } {
            self.complete(table, minimums)?;
        }

        Ok(FullyExplored)
    }

    fn delay_negatives<'i>(
        infer: &'i mut InferenceTable,
        table: TableIndex,
        negatives: Vec<CanonicalPendingExClause>,
    ) -> impl Iterator<Item = (StackIndex, ExClause)> + 'i {
        negatives.into_iter().map(move |pending| {
            let PendingExClause {
                goal_depth,
                subst,
                selected_goal,
                mut delayed_literals,
                constraints,
                subgoals,
            } = infer.instantiate_canonical(&pending);

            // we don't need this for anything
            mem::drop(selected_goal);

            // delay the selected goal
            delayed_literals.push(DelayedLiteral::Negative(table));

            (
                goal_depth,
                ExClause {
                    subst,
                    delayed_literals,
                    constraints,
                    subgoals,
                },
            )
        })
    }

    ///////////////////////////////////////////////////////////////////////////

    /// Used whenever we process an answer (whether new or cached) on
    /// a positive edge (the SLG POSITIVE RETURN operation). Truncates
    /// the resolvent (or factor) if it has grown too large.
    fn truncate_returned(&mut self, ex_clause: ExClause) -> ExClause {
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

        match truncate(&mut self.infer, self.max_size, &ex_clause.subst) {
            // No need to truncate? Just propagate the resolvent back.
            Truncated {
                overflow: false, ..
            } => ex_clause,

            // Resolvent got too large. Have to introduce approximation.
            Truncated {
                overflow: true,
                value: truncated_subst,
            } => {
                // DIVERGENCE
                //
                // In RR, `ex_clause.delayed_literals` would be
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

///////////////////////////////////////////////////////////////////////////
// SIMPLIFICATION
//
// Simplification is not described in the EWFS paper in any
// detail, but it's a crucial final phase to the SLG
// algorithm. Basically, once the *main* processing terminates, we
// will have propagated all of the potential answers around
// between the tables, but some of them will have **delayed
// literals**. A delayed literal is some kind of subgoal whose
// truth could not be determined in the usual order (ordinarily,
// we select subgoals for processing in a fixed order). Delaying
// typically occurs because of negative loops, but, in the
// interest of efficiency, the procedure for selecting what to
// delay is a bit approximated, and so sometimes we will sweep up
// other goals and delay them.
//
// In any case, once we reach the simplification phase, we have
// the full set of answers accumulated, but some of them have
// delayed literals as part of them. The simplification phase
// resolves this by iterating and removing delayed literals until
// a fixed point is reached.
//
// The key constraint is this. Given an answer:
//
//     G :- D1..Dn |
//
// where D1..Dn are delayed literals, we can simplify as follows.
// Consider some delayed literal Di.
//
// If Di = ~G is a negative delayed literal:
//
// - If the table for G has NO answers, then the delayed literal
//   can be removed.
// - On the other hand, if the table for goal G has an
//   unconditional answer, then the answer can be removed.
// - (Note that in our representation, we just keep the TableIndex
//   for the goal G, since that's all we need.)
//
// If Di = ~G is a positive delayed literal:
//
// - If there is an unconditional answer for G somewhere in the forest, then Di can be removed.
//   - In our representation, we just remember the table and (canonical) answer
//     that we are interested in, for a faster lookup.
//   - Interestingly, that is not necessarily a unique place for
//     the answer to be found -- e.g. if the goal is `Foo<i32>`
//     and the table was for `Foo<T>`, there may be another table
//     for `Foo<i32>` specifically -- but the answer should be
//     there if it is anywhere.
// - If, on the other hand, the table for G has no answer at all for G, then the answer
//   can be removed.

struct Simplification<'tables> {
    tables: &'tables mut Tables,
}

impl<'tables> Simplification<'tables> {
    fn simplify(tables: &'tables mut Tables) {
        Simplification { tables }.simplification()
    }

    fn simplification(&mut self) {
        let mut any_changed = ChangedFlag::new(true);

        while any_changed.check() {
            for table in self.tables.indices() {
                // Make a local copy of the answers table for editing.
                let mut answers = self.tables[table].answers.clone();

                let mut table_changed = ChangedFlag::new(false);

                answers.retain(|_answer_subst, answer_sets| {
                    self.retain_delay_sets(&mut table_changed, answer_sets)
                });

                if table_changed.check() {
                    self.tables[table].answers = answers;
                    any_changed.mark_changed();
                }
            }
        }
    }

    /// Simplifies the delayed literal sets found in some answer.
    ///
    /// Returns false if the answer should be removed. This occurs when some
    /// delayed literal is found to be definitively not satisfied.
    ///
    /// Otherwise, returns true and modifies `*dl_sets` in place to
    /// reflect the new state.
    ///
    /// Whatever value is returned, updates `changed` to true if any
    /// changes were made.
    fn retain_delay_sets(
        &self,
        changed: &mut ChangedFlag,
        dl_sets: &mut DelayedLiteralSets,
    ) -> bool {
        // Temporarily swap `*sets` with `None`; thus, if we return
        // early, we either remove the answer (false) or declare it
        // unconditional (true).
        let tmp = mem::replace(dl_sets, DelayedLiteralSets::None);

        match tmp {
            // No delayed literals? Just return, no changes.
            DelayedLiteralSets::None => true,

            // There are various sets of delayed literals. Let's see
            // if we can simplify any of them.
            DelayedLiteralSets::Some(sets) => {
                let mut new_sets = HashSet::new();
                for mut set in sets {
                    let len_before = set.delayed_literals.len();

                    // If we find that some set is not satisfiable, skip it.
                    if !self.retain_delay_set(&mut set) {
                        changed.mark_changed();
                        continue;
                    }

                    // If the number of delayed literals changed, something was filtered out.
                    changed.mark_changed_if(set.delayed_literals.len() != len_before);

                    // If all delayed literals were filtered out, then
                    // the answer is unambiguously true. We can just
                    // return true -- `*dl_sets` has already been
                    // updated to `None` above.
                    if set.delayed_literals.is_empty() {
                        return true;
                    }

                    // Otherwise, remember this for later as a
                    // possible set of delayed literals.
                    new_sets.insert(set);
                }

                // If we wound up removing *all* the sets, return false.
                // Otherwise, update `*dl_sets` with the remaining sets.
                if new_sets.is_empty() {
                    false
                } else {
                    *dl_sets = DelayedLiteralSets::Some(new_sets);
                    true
                }
            }
        }
    }

    /// Iterates through `set` and removes any delayed literals that
    /// are now known to be satisfied. If a literal is found that is
    /// known to be **unsatisfied**, returns false.
    fn retain_delay_set(&self, set: &mut DelayedLiteralSet) -> bool {
        let mut found_false = false;
        set.delayed_literals.retain(|literal| {
            match self.eval_delayed_literal(literal) {
                // Drop literals that are known to be true.
                TruthValue::True => false,

                // Drop literals that are known to be false, and
                // remember that we saw one. (Ideally we'd
                // shortcircuit here but we can't easily do so inside
                // of `retain()`.)
                TruthValue::False => {
                    found_false = true;
                    false
                }

                // Keep delayed literals whose truth value is not yet known.
                TruthValue::Unknown => true,
            }
        });

        // Keep this answer unless we found a truth value that was
        // known to be false.
        !found_false
    }

    fn eval_delayed_literal(&self, literal: &DelayedLiteral) -> TruthValue {
        match literal {
            // CannotProve is never true nor false.
            DelayedLiteral::CannotProve(()) => TruthValue::Unknown,

            // A literal ~G is true if G is known to be unsatisfiable
            // and false if G is known to be satisfied.
            DelayedLiteral::Negative(table) => if self.tables[*table].is_not_satisfiable() {
                TruthValue::True
            } else if self.tables[*table].is_satisfiable() {
                TruthValue::False
            } else {
                TruthValue::Unknown
            },

            // A literal G (from table T) is true is true if the table
            // T has an answer G that has no delayed literals; it is
            // false if the table T has no answer G at all.
            DelayedLiteral::Positive(table, constrained_goal) => {
                match self.tables[*table].answers.get(&constrained_goal) {
                    Some(DelayedLiteralSets::None) => TruthValue::True,
                    Some(DelayedLiteralSets::Some(_)) => TruthValue::Unknown,
                    None => TruthValue::False,
                }
            }
        }
    }
}

struct ChangedFlag {
    changed: bool,
}

impl ChangedFlag {
    fn new(changed: bool) -> ChangedFlag {
        ChangedFlag { changed }
    }

    /// Reads current value of the changed flag, setting it to false
    /// afterwards.
    fn check(&mut self) -> bool {
        mem::replace(&mut self.changed, false)
    }

    /// Sets flag to true.
    fn mark_changed(&mut self) {
        self.changed = true;
    }

    /// Sets flag to true if `changed` is true.
    fn mark_changed_if(&mut self, changed: bool) {
        self.changed |= bool::from(changed);
    }
}

impl Stack {
    fn next_index(&self) -> StackIndex {
        StackIndex {
            value: self.stack.len(),
        }
    }

    // Pop all stack frames up to and and including the one at `depth`.
    fn pop_until(&mut self, depth: StackIndex) -> Vec<StackEntry> {
        assert!(depth.value < self.stack.len()); // `depth` must not yet have been popped
        let mut result = Vec::with_capacity(self.stack.len() - depth.value);
        while self.next_index() != depth {
            result.push(self.stack.pop().unwrap());
        }
        result
    }

    fn peek_until(&mut self, depth: StackIndex) -> &mut [StackEntry] {
        &mut self.stack[depth.value..]
    }

    fn push(&mut self, table: TableIndex, dfn: DepthFirstNumber) {
        self.stack.push(StackEntry {
            table,
            dfn,
            link: Minimums {
                positive: dfn,
                negative: DepthFirstNumber::MAX,
            },
        });
    }

    fn top(&self) -> &StackEntry {
        self.stack.last().unwrap()
    }
}

impl Index<StackIndex> for Stack {
    type Output = StackEntry;

    fn index(&self, index: StackIndex) -> &StackEntry {
        &self.stack[index.value]
    }
}

impl IndexMut<StackIndex> for Stack {
    fn index_mut(&mut self, index: StackIndex) -> &mut StackEntry {
        &mut self.stack[index.value]
    }
}

impl Tables {
    fn indices(&self) -> Range<TableIndex> {
        TableIndex { value: 0 }..self.next_index()
    }

    fn next_index(&self) -> TableIndex {
        TableIndex {
            value: self.tables.len(),
        }
    }

    fn insert(&mut self, goal: &CanonicalGoal, depth: StackIndex) -> TableIndex {
        let index = self.next_index();
        self.tables.push(Table {
            table_goal: goal.clone(),
            answers: HashMap::new(),
            positives: vec![],
            negatives: vec![],
            depth: Some(depth),
        });
        self.table_indices.insert(goal.clone(), index);
        index
    }

    fn index_of(&self, literal: &CanonicalGoal) -> Option<TableIndex> {
        self.table_indices.get(literal).cloned()
    }
}

impl Index<TableIndex> for Tables {
    type Output = Table;

    fn index(&self, index: TableIndex) -> &Table {
        &self.tables[index.value]
    }
}

impl IndexMut<TableIndex> for Tables {
    fn index_mut(&mut self, index: TableIndex) -> &mut Table {
        &mut self.tables[index.value]
    }
}

impl<'a> IntoIterator for &'a mut Tables {
    type IntoIter = <&'a mut Vec<Table> as IntoIterator>::IntoIter;
    type Item = <&'a mut Vec<Table> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&mut self.tables)
    }
}

impl Table {
    fn export_answers(&self) -> Answers {
        let mut result = Answers {
            answers: self.answers
                .iter()
                .map(|(subst, delay_sets)| {
                    Answer {
                        subst: subst.clone(),
                        ambiguous: !delay_sets.is_empty(),
                    }
                })
                .collect(),
        };

        result.answers.sort();

        result
    }

    /// Marks this table as completely evaluated. In the process,
    /// returns the list of pending negative clauses, since those can
    /// possibly now be updated (either to mark them as SUCCESSFUL or
    /// FAILED).
    fn mark_complete(&mut self) -> Vec<CanonicalPendingExClause> {
        let negatives = self.take_negatives();
        self.positives = vec![];
        self.depth = None;
        negatives
    }

    /// Takes the list of negatives and replaces it with an empty
    /// list.  Used when either marking a table as completed or else
    /// delaying a bunch of literals.
    fn take_negatives(&mut self) -> Vec<CanonicalPendingExClause> {
        mem::replace(&mut self.negatives, vec![])
    }

    /// True if this table has at least one solution without a delayed
    /// literal.
    fn is_satisfiable(&self) -> bool {
        self.answers
            .values()
            .any(|delayed_literals| delayed_literals.is_empty())
    }

    /// True if this table has at least one solution without a delayed
    /// literal.
    fn is_not_satisfiable(&self) -> bool {
        self.answers.is_empty()
    }
}

impl DelayedLiteralSets {
    fn is_empty(&self) -> bool {
        match *self {
            DelayedLiteralSets::None => true,
            DelayedLiteralSets::Some(_) => false,
        }
    }
}

impl Minimums {
    /// Update our fields to be the minimum of our current value
    /// and the values from other.
    fn take_minimums(&mut self, other: &Minimums) {
        self.positive = min(self.positive, other.positive);
        self.negative = min(self.negative, other.negative);
    }

    fn take_negative_minimum(&mut self, other: DepthFirstNumber) {
        self.negative = min(self.negative, other);
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
    fn with_constraints<I>(mut self, constraints: I) -> Self
    where
        I: IntoIterator<Item = InEnvironment<Constraint>>,
    {
        self.constraints.extend(constraints);
        self.constraints.sort();
        self.constraints.dedup();
        self
    }
}

impl<T> Satisfiable<T> {
    fn yes(self) -> Option<T> {
        match self {
            Satisfiable::Yes(v) => Some(v),
            Satisfiable::No => None,
        }
    }

    fn map<F, U>(self, op: F) -> Satisfiable<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Satisfiable::Yes(v) => Satisfiable::Yes(op(v)),
            Satisfiable::No => Satisfiable::No,
        }
    }
}

impl iter::Step for TableIndex {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        usize::steps_between(&start.value, &end.value)
    }

    fn replace_one(&mut self) -> Self {
        TableIndex {
            value: usize::replace_one(&mut self.value),
        }
    }

    fn replace_zero(&mut self) -> Self {
        TableIndex {
            value: usize::replace_zero(&mut self.value),
        }
    }

    fn add_one(&self) -> Self {
        TableIndex {
            value: usize::add_one(&self.value),
        }
    }

    fn sub_one(&self) -> Self {
        TableIndex {
            value: usize::sub_one(&self.value),
        }
    }

    fn add_usize(&self, n: usize) -> Option<Self> {
        usize::add_usize(&self.value, n).map(|value| TableIndex { value })
    }
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
