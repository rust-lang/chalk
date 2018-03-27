use crate::context::{Context, InferenceContext};

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum HhGoal<C: Context, I: InferenceContext<C>> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    ForAll(I::BindersGoal),
    Exists(I::BindersGoal),
    Implies(Vec<I::ProgramClause>, I::Goal),
    And(I::Goal, I::Goal),
    Not(I::Goal),
    Unify(I::Parameter, I::Parameter),
    DomainGoal(I::DomainGoal),

    /// Indicates something that cannot be proven to be true or false
    /// definitively. This can occur with overflow but also with
    /// unifications of skolemized variables like `forall<X,Y> { X = Y
    /// }`. Of course, that statement is false, as there exist types
    /// X, Y where `X = Y` is not true. But we treat it as "cannot
    /// prove" so that `forall<X,Y> { not { X = Y } }` also winds up
    /// as cannot prove.
    CannotProve,
}

