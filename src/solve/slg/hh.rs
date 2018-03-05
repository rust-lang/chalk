use solve::slg::context::Context;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A general goal; this is the full range of questions you can pose to Chalk.
crate enum HhGoal<C: Context> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    ForAll(C::BindersGoal),
    Exists(C::BindersGoal),
    Implies(Vec<C::DomainGoal>, C::Goal),
    And(C::Goal, C::Goal),
    Not(C::Goal),
    Unify(C::Parameter, C::Parameter),
    DomainGoal(C::DomainGoal),

    /// Indicates something that cannot be proven to be true or false
    /// definitively. This can occur with overflow but also with
    /// unifications of skolemized variables like `forall<X,Y> { X = Y
    /// }`. Of course, that statement is false, as there exist types
    /// X, Y where `X = Y` is not true. But we treat it as "cannot
    /// prove" so that `forall<X,Y> { not { X = Y } }` also winds up
    /// as cannot prove.
    CannotProve,
}

