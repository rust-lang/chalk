//! Defines traits used to embed the chalk-engine in another crate.
//!
//! chalk and rustc both define types which implement the traits in this
//! module. This allows each user of chalk-engine to define their own
//! `DomainGoal` type, add arena lifetime parameters, and more. See
//! [`Context`] trait for a list of types.

use crate::{CompleteAnswer, ExClause};
use chalk_ir::interner::Interner;
use chalk_ir::{
    AnswerSubst, Binders, Canonical, ConstrainedSubst, Constraint, DomainGoal, Environment,
    Fallible, Floundered, GenericArg, Goal, InEnvironment, ProgramClause, ProgramClauses,
    Substitution, UCanonical, UniverseMap,
};
use std::fmt::Debug;

/// The "context" in which the SLG solver operates. It defines all the
/// types that the SLG solver may need to refer to, as well as a few
/// very simple interconversion methods.
///
/// At any given time, the SLG solver may have more than one context
/// active. First, there is always the *global* context, but when we
/// are in the midst of pursuing some particular strand, we will
/// instantiate a second context just for that work, via the
/// `instantiate_ucanonical_goal` and `instantiate_ex_clause` methods.
///
/// In the chalk implementation, these two contexts are mapped to the
/// same type. But in the rustc implementation, this second context
/// corresponds to a fresh arena, and data allocated in that second
/// context will be freed once the work is done. (The "canonicalizing"
/// steps offer a way to convert data from the inference context back
/// into the global context.)
///
/// FIXME: Clone and Debug bounds are just for easy derive, they are
/// not actually necessary. But dang are they convenient.
pub trait Context<I: Interner>: Clone + Debug {
    /// Represents an inference table.
    type InferenceTable: InferenceTable<I, Self> + Clone;

    /// Selects the next appropriate subgoal index for evaluation.
    /// Used by: logic
    fn next_subgoal_index(ex_clause: &ExClause<I>) -> usize;
}

pub trait ContextOps<I: Interner, C: Context<I>>: Sized + Clone + Debug {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal<I>>>) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    ///
    /// If this callback returns `None`, that indicates that the set
    /// of program clauses cannot be enumerated because there are
    /// unresolved type variables that would have to be resolved
    /// first; the goal will be considered floundered.
    fn program_clauses(
        &self,
        environment: &Environment<I>,
        goal: &DomainGoal<I>,
        infer: &mut C::InferenceTable,
    ) -> Result<Vec<ProgramClause<I>>, Floundered>;

    // Used by: simplify
    fn add_clauses(&self, env: &Environment<I>, clauses: ProgramClauses<I>) -> Environment<I>;

    /// Create an inference table for processing a new goal and instantiate that goal
    /// in that context, returning "all the pieces".
    ///
    /// More specifically: given a u-canonical goal `arg`, creates a
    /// new inference table `T` and populates it with the universes
    /// found in `arg`. Then, creates a substitution `S` that maps
    /// each bound variable in `arg` to a fresh inference variable
    /// from T. Returns:
    ///
    /// - the table `T`
    /// - the substitution `S`
    /// - the environment and goal found by substitution `S` into `arg`
    fn instantiate_ucanonical_goal(
        &self,
        arg: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> (C::InferenceTable, Substitution<I>, Environment<I>, Goal<I>);

    fn instantiate_ex_clause(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<I>>,
    ) -> (C::InferenceTable, ExClause<I>);

    // Used by: logic
    fn instantiate_answer_subst(
        &self,
        num_universes: usize,
        answer: &Canonical<AnswerSubst<I>>,
    ) -> (
        C::InferenceTable,
        Substitution<I>,
        Vec<InEnvironment<Constraint<I>>>,
        Vec<InEnvironment<Goal<I>>>,
    );

    /// Returns a identity substitution.
    fn identity_constrained_subst(
        &self,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Canonical<ConstrainedSubst<I>>;

    /// Convert a goal G *from* the canonical universes *into* our
    /// local universes. This will yield a goal G' that is the same
    /// but for the universes of universally quantified names.
    fn map_goal_from_canonical(
        &self,
        _: &UniverseMap,
        value: &Canonical<InEnvironment<Goal<I>>>,
    ) -> Canonical<InEnvironment<Goal<I>>>;

    /// Convert a substitution *from* the canonical universes *into*
    /// our local universes. This will yield a substitution S' that is
    /// the same but for the universes of universally quantified
    /// names.
    fn map_subst_from_canonical(
        &self,
        _: &UniverseMap,
        value: &Canonical<AnswerSubst<I>>,
    ) -> Canonical<AnswerSubst<I>>;

    fn interner(&self) -> &I;

    /// Upcast this domain goal into a more general goal.
    fn into_goal(&self, domain_goal: DomainGoal<I>) -> Goal<I>;

    fn is_trivial_constrained_substitution(
        &self,
        constrained_subst: &Canonical<ConstrainedSubst<I>>,
    ) -> bool;

    fn is_trivial_substitution(
        &self,
        u_canon: &UCanonical<InEnvironment<Goal<I>>>,
        canonical_subst: &Canonical<AnswerSubst<I>>,
    ) -> bool;
}

/// An "inference table" contains the state to support unification and
/// other operations on terms.
pub trait InferenceTable<I: Interner, C: Context<I>>:
    ResolventOps<I, C> + TruncateOps<I, C> + UnificationOps<I, C>
{
}

/// Methods for unifying and manipulating terms and binders.
pub trait UnificationOps<I: Interner, C: Context<I>> {
    // Used by: simplify
    fn instantiate_binders_universally(&mut self, interner: &I, arg: &Binders<Goal<I>>) -> Goal<I>;

    // Used by: simplify
    fn instantiate_binders_existentially(
        &mut self,
        interner: &I,
        arg: &Binders<Goal<I>>,
    ) -> Goal<I>;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause<'v>(&mut self, interner: &I, value: &'v ExClause<I>) -> Box<dyn Debug + 'v>;

    // Used by: logic
    fn fully_canonicalize_goal(
        &mut self,
        interner: &I,
        value: &InEnvironment<Goal<I>>,
    ) -> (UCanonical<InEnvironment<Goal<I>>>, UniverseMap);

    // Used by: logic
    fn canonicalize_ex_clause(
        &mut self,
        interner: &I,
        value: &ExClause<I>,
    ) -> Canonical<ExClause<I>>;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        interner: &I,
        subst: Substitution<I>,
        constraints: Vec<InEnvironment<Constraint<I>>>,
    ) -> Canonical<ConstrainedSubst<I>>;

    // Used by: logic
    fn canonicalize_answer_subst(
        &mut self,
        interner: &I,
        subst: Substitution<I>,
        constraints: Vec<InEnvironment<Constraint<I>>>,
        delayed_subgoals: Vec<InEnvironment<Goal<I>>>,
    ) -> Canonical<AnswerSubst<I>>;

    // Used by: logic
    fn invert_goal(
        &mut self,
        interner: &I,
        value: &InEnvironment<Goal<I>>,
    ) -> Option<InEnvironment<Goal<I>>>;

    /// First unify the parameters, then add the residual subgoals
    /// as new subgoals of the ex-clause.
    /// Also add region constraints.
    ///
    /// If the parameters fail to unify, then `Error` is returned
    // Used by: simplify
    fn unify_generic_args_into_ex_clause(
        &mut self,
        interner: &I,
        environment: &Environment<I>,
        a: &GenericArg<I>,
        b: &GenericArg<I>,
        ex_clause: &mut ExClause<I>,
    ) -> Fallible<()>;
}

/// "Truncation" (called "abstraction" in the papers referenced below)
/// refers to the act of modifying a goal or answer that has become
/// too large in order to guarantee termination.
///
/// Currently we don't perform truncation (but it might me readded later).
///
/// Citations:
///
/// - Terminating Evaluation of Logic Programs with Finite Three-Valued Models
///   - Riguzzi and Swift; ACM Transactions on Computational Logic 2013
/// - Radial Restraint
///   - Grosof and Swift; 2013
pub trait TruncateOps<I: Interner, C: Context<I>> {
    /// Check if `subgoal` is too large
    fn goal_needs_truncation(&mut self, interner: &I, subgoal: &InEnvironment<Goal<I>>) -> bool;

    /// Check if `subst` is too large
    fn answer_needs_truncation(&mut self, interner: &I, subst: &Substitution<I>) -> bool;
}

pub trait ResolventOps<I: Interner, C: Context<I>> {
    /// Combines the `goal` (instantiated within `infer`) with the
    /// given program clause to yield the start of a new strand (a
    /// canonical ex-clause).
    ///
    /// The bindings in `infer` are unaffected by this operation.
    fn resolvent_clause(
        &mut self,
        interner: &I,
        environment: &Environment<I>,
        goal: &DomainGoal<I>,
        subst: &Substitution<I>,
        clause: &ProgramClause<I>,
    ) -> Fallible<ExClause<I>>;

    fn apply_answer_subst(
        &mut self,
        interner: &I,
        ex_clause: &mut ExClause<I>,
        selected_goal: &InEnvironment<Goal<I>>,
        answer_table_goal: &Canonical<InEnvironment<Goal<I>>>,
        canonical_answer_subst: &Canonical<AnswerSubst<I>>,
    ) -> Fallible<()>;
}

pub enum AnswerResult<I: Interner> {
    /// The next available answer.
    Answer(CompleteAnswer<I>),

    /// No answer could be returned because there are no more solutions.
    NoMoreSolutions,

    /// No answer could be returned because the goal has floundered.
    Floundered,

    // No answer could be returned *yet*, because we exceeded our
    // quantum (`should_continue` returned false).
    QuantumExceeded,
}

impl<I: Interner> AnswerResult<I> {
    pub fn is_answer(&self) -> bool {
        match self {
            Self::Answer(_) => true,
            _ => false,
        }
    }

    pub fn answer(self) -> CompleteAnswer<I> {
        match self {
            Self::Answer(answer) => answer,
            _ => panic!("Not an answer."),
        }
    }

    pub fn is_no_more_solutions(&self) -> bool {
        match self {
            Self::NoMoreSolutions => true,
            _ => false,
        }
    }

    pub fn is_quantum_exceeded(&self) -> bool {
        match self {
            Self::QuantumExceeded => true,
            _ => false,
        }
    }
}

impl<I: Interner> Debug for AnswerResult<I> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnswerResult::Answer(answer) => write!(fmt, "{:?}", answer),
            AnswerResult::Floundered => write!(fmt, "Floundered"),
            AnswerResult::NoMoreSolutions => write!(fmt, "None"),
            AnswerResult::QuantumExceeded => write!(fmt, "QuantumExceeded"),
        }
    }
}

pub trait AnswerStream<I: Interner> {
    /// Gets the next answer for a given goal, but doesn't increment the answer index.
    /// Calling this or `next_answer` again will give the same answer.
    fn peek_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I>;

    /// Gets the next answer for a given goal, incrementing the answer index.
    /// Calling this or `peek_answer` again will give the next answer.
    fn next_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I>;

    /// Invokes `test` with each possible future answer, returning true immediately
    /// if we find any answer for which `test` returns true.
    fn any_future_answer(&self, test: impl Fn(&Substitution<I>) -> bool) -> bool;
}
