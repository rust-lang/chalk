use crate::context::Context;

use chalk_ir::interner::Interner;
use chalk_ir::{DomainGoal, GenericArg, Goal};

#[derive(Clone, PartialEq, Eq, Hash)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum HhGoal<I: Interner, C: Context<I>> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    ForAll(C::BindersGoal),
    Exists(C::BindersGoal),
    Implies(C::ProgramClauses, Goal<I>),
    All(Vec<Goal<I>>),
    Not(Goal<I>),
    Unify(C::Variance, GenericArg<I>, GenericArg<I>),
    DomainGoal(DomainGoal<I>),

    /// Indicates something that cannot be proven to be true or false
    /// definitively. This can occur with overflow but also with
    /// unifications of placeholder variables like `forall<X,Y> { X = Y
    /// }`. Of course, that statement is false, as there exist types
    /// X, Y where `X = Y` is not true. But we treat it as "cannot
    /// prove" so that `forall<X,Y> { not { X = Y } }` also winds up
    /// as cannot prove.
    CannotProve,
}
