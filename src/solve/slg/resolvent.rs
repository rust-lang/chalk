use cast::Caster;
use super::*;

///////////////////////////////////////////////////////////////////////////
// SLG RESOLVENTS
//
// The "SLG Resolvent" is used to combine a *goal* G with some
// clause or answer *C*.  It unifies the goal's selected literal
// with the clause and then inserts the clause's conditions into
// the goal's list of things to prove, basically. Although this is
// one operation in EWFS, we have specialized variants for merging
// a program clause and an answer (though they share some code in
// common).
//
// Terminology note: The NFTD and RR papers use the term
// "resolvent" to mean both the factor and the resolvent, but EWFS
// distinguishes the two. We follow EWFS here since -- in the code
// -- we tend to know whether there are delayed literals or not,
// and hence to know which code path we actually want.
//
// From EWFS:
//
// Let G be an X-clause A :- D | L1,...Ln, where N > 0, and Li be selected atom.
//
// Let C be an X-clause with no delayed literals. Let
//
//     C' = A' :- L'1...L'm
//
// be a variant of C such that G and C' have no variables in
// common.
//
// Let Li and A' be unified with MGU S.
//
// Then:
//
//     S(A :- D | L1...Li-1, L1'...L'm, Li+1...Ln)
//
// is the SLG resolvent of G with C.

/// Applies the SLG resolvent algorithm to incorporate a new
/// answer and apply it to a previously blocked ex-clause.
pub(super) fn resolvent_pending(
    infer: &mut InferenceTable,
    pending_ex_clause: &CanonicalPendingExClause,
    answer_table_goal: &CanonicalGoal,
    answer_subst: &CanonicalConstrainedSubst,
) -> Satisfiable<(StackIndex, ExClause)> {
    let PendingExClause {
        goal_depth,
        subst,
        selected_goal,
        delayed_literals,
        constraints,
        subgoals,
    } = infer.instantiate_canonical(pending_ex_clause);

    let ex_clause = ExClause {
        subst,
        delayed_literals,
        constraints,
        subgoals,
    };

    resolvent::resolvent_answer(
        infer,
        &ex_clause,
        &selected_goal,
        answer_table_goal,
        answer_subst,
    ).map(|r| (goal_depth, r))
}

/// Applies the SLG resolvent algorithm to incorporate an answer
/// produced by the selected literal into the main X-clause,
/// producing a new X-clause that must be solved.
///
/// # Parameters
///
/// - `ex_clause` is the X-clause we are trying to prove,
///   with the selected literal removed from its list of subgoals
/// - `selected_goal` is the selected literal that was removed
/// - `answer` is some answer to `selected_goal` that has been found
fn resolvent_answer(
    infer: &mut InferenceTable,
    ex_clause: &ExClause,
    selected_goal: &InEnvironment<Goal>,
    answer_table_goal: &CanonicalGoal,
    answer_subst: &CanonicalConstrainedSubst,
) -> Satisfiable<ExClause> {
    // Relating the above describes to our parameters:
    //
    // - the goal G is `ex_clause` is, with `selected_goal` being
    //   the selected literal Li, already removed from the list.
    // - the clause C is `answer.` (i.e., the answer has no conditions).

    // C' is now `answer`. No variables in commmon with G.
    let ConstrainedSubst {
        subst: answer_subst,
        constraints: answer_constraints,
    } = infer.instantiate_canonical(&answer_subst);

    // Apply the substitution from the answer to the original
    // table goal to yield our new `answer_goal`. This is needed
    // for unifying.
    let answer_goal = answer_table_goal.substitute(&answer_subst);

    // Perform the SLG resolvent unification.
    let resolvent = resolvent::resolvent_unify(
        infer,
        ex_clause.clone(),
        selected_goal,
        &answer_goal,
        vec![],
    );

    // We have one additional complication: we have to insert the
    // region constraints.
    resolvent.map(|ex_clause| ex_clause.with_constraints(answer_constraints))
}

/// Applies the SLG resolvent algorithm to incorporate a program
/// clause into the main X-clause, producing a new X-clause that
/// must be solved.
///
/// # Parameters
///
/// - `goal` is the goal G that we are trying to solve
/// - `clause` is the program clause that may be useful to that end
pub(super) fn resolvent_clause(
    infer: &mut InferenceTable,
    goal: &InEnvironment<DomainGoal>,
    subst: &Substitution,
    clause: &Binders<ProgramClauseImplication>,
) -> Satisfiable<ExClause> {
    // Relating the above description to our situation:
    //
    // - `goal` G, except with binders for any existential variables.
    //   - Also, we always select the first literal in `ex_clause.literals`, so `i` is 0.
    // - `clause` is C, except with binders for any existential variables.

    // Goal here is now G.
    let ex_clause = ExClause {
        subst: subst.clone(),
        delayed_literals: vec![],
        constraints: vec![],
        subgoals: vec![],
    };

    // The selected literal for us will always be the main goal
    // `G`. See if we can unify that with C'.
    let environment = &goal.environment;

    // C' in the description above is `consequence :- conditions`.
    //
    // Note that G and C' have no variables in common.
    let ProgramClauseImplication {
        consequence,
        conditions,
    } = infer.instantiate_binders_existentially(clause);
    let consequence: InEnvironment<DomainGoal> = InEnvironment::new(&environment, consequence);

    resolvent::resolvent_unify(infer, ex_clause, &goal, &consequence, conditions)
}

/// Given the goal G (`goal`) with selected literal Li
/// (`selected_goal`), the goal environment `environment`, and
/// the clause C' (`consequence :- conditions`), applies the SLG
/// resolvent algorithm to yield a new `ExClause`.
fn resolvent_unify<G>(
    infer: &mut InferenceTable,
    mut goal: ExClause,
    selected_goal: &InEnvironment<G>,
    consequence: &InEnvironment<G>,
    conditions: Vec<Goal>,
) -> Satisfiable<ExClause>
where
    G: Zip,
{
    let environment = &selected_goal.environment;

    // Unify the selected literal Li with C'.
    let UnificationResult { goals, constraints } = {
        match infer.unify(&selected_goal.environment, selected_goal, consequence) {
            Err(_) => return Satisfiable::No,
            Ok(v) => v,
        }
    };

    goal.constraints.extend(constraints);

    // One (minor) complication: unification for us sometimes yields further domain goals.
    goal.subgoals
        .extend(goals.into_iter().casted().map(Literal::Positive));

    // Add the `conditions` into the result. One complication is
    // that these are HH-clauses, so we have to simplify into
    // literals first. This can product a sum-of-products. This is
    // why we return a vector.
    goal.subgoals
        .extend(conditions.into_iter().map(|c| match c {
            Goal::Not(c) => Literal::Negative(InEnvironment::new(&environment, *c)),
            c => Literal::Positive(InEnvironment::new(&environment, c)),
        }));

    Satisfiable::Yes(goal)
}

///////////////////////////////////////////////////////////////////////////
// SLG FACTOR
//
// The "SLG Factor" is used to combine a *goal* G with some answer
// *C*, where C contains delayed literals. It unifies the goal's
// selected literal with the answer and then inserts the delayed
// literals into the goal's list of delayed literals.
//
// Terminology note: The NFTD and RR papers use the term
// "resolvent" to mean both the factor and the resolvent, but EWFS
// distinguishes the two. We follow EWFS here since -- in the code
// -- we tend to know whether there are delayed literals or not,
// and hence to know which code path we actually want.
//
// From EWFS:
//
// Let G be an X-clause A :- D | L1,...Ln, where N > 0, and Li be selected atom.
//
// Let C be an X-clause with delayed literals. Let
//
//     C' = A' :- D' |
//
// be a variant of C such that G and C' have no variables in
// common.
//
// Let Li and A' be unified with MGU S.
//
// Then:
//
//     S(A :- D,Li | L1...Li-1, Li+1...Ln)
//                             ^ see below
//
// is the SLG factor of G with C. We alter the process mildly to insert
// some clauses into `^` -- in particular, the side-effects of unification.

pub(super) fn factor_pending(
    infer: &mut InferenceTable,
    pending_ex_clause: &CanonicalPendingExClause,
    answer_table: TableIndex,
    answer_table_goal: &CanonicalGoal,
    answer_subst: &CanonicalConstrainedSubst,
) -> Satisfiable<(StackIndex, ExClause)> {
    let PendingExClause {
        goal_depth,
        subst,
        selected_goal,
        delayed_literals,
        constraints,
        subgoals,
    } = infer.instantiate_canonical(pending_ex_clause);

    let ex_clause = ExClause {
        subst,
        delayed_literals,
        constraints,
        subgoals,
    };

    resolvent::factor(
        infer,
        &ex_clause,
        &selected_goal,
        answer_table,
        answer_table_goal,
        answer_subst,
    ).map(|c| (goal_depth, c))
}

fn factor(
    infer: &mut InferenceTable,
    ex_clause: &ExClause,
    selected_goal: &InEnvironment<Goal>,
    answer_table: TableIndex,
    answer_table_goal: &CanonicalGoal,
    canonical_answer_subst: &CanonicalConstrainedSubst,
) -> Satisfiable<ExClause> {
    let mut ex_clause = ex_clause.clone();

    // C' is now `answer`. No variables in commmon with G.
    let ConstrainedSubst {
        subst: answer_subst,

        // Assuming unification succeeds, we incorporate the
        // region constraints from the answer into the result;
        // we'll need them if this answer (which is not yet known
        // to be true) winds up being true, and otherwise (if the
        // answer is false or unknown) it doesn't matter.
        constraints: answer_constraints,
    } = infer.instantiate_canonical(&canonical_answer_subst);

    let answer_goal = answer_table_goal.substitute(&answer_subst);

    // Unify the selected literal Li with C'.
    let UnificationResult { goals, constraints } = {
        match infer.unify(&selected_goal.environment, selected_goal, &answer_goal) {
            Err(_) => return Satisfiable::No,
            Ok(v) => v,
        }
    };

    // Push Li into the list of delayed literals.
    ex_clause.delayed_literals.push(DelayedLiteral::Positive(
        answer_table,
        canonical_answer_subst.clone(),
    ));

    // We must also take into account the add'l conditions that
    // arise from our unification procedure.
    ex_clause.constraints.extend(constraints);
    ex_clause.constraints.extend(answer_constraints);
    ex_clause
        .subgoals
        .extend(goals.into_iter().casted().map(Literal::Positive));

    Satisfiable::Yes(ex_clause)
}
