use chalk_solve::Solution;
use tracing::debug;

use chalk_ir::interner::Interner;
use chalk_ir::{ClausePriority, DomainGoal, GenericArg};

#[tracing::instrument(level = "Debug", skip(interner))]
pub(super) fn with_priorities<I: Interner>(
    interner: I,
    domain_goal: &DomainGoal<I>,
    a: Solution<I>,
    prio_a: ClausePriority,
    b: Solution<I>,
    prio_b: ClausePriority,
) -> (Solution<I>, ClausePriority) {
    let result = match (prio_a, prio_b, a, b) {
        (ClausePriority::High, ClausePriority::Low, higher, lower)
        | (ClausePriority::Low, ClausePriority::High, lower, higher) => {
            // if we have a high-priority solution and a low-priority solution,
            // the high-priority solution overrides *if* they are both for the
            // same inputs -- we don't want a more specific high-priority
            // solution overriding a general low-priority one. Currently inputs
            // only matter for projections; in a goal like `AliasEq(<?0 as
            // Trait>::Type = ?1)`, ?0 is the input.
            let inputs_higher = calculate_inputs(interner, domain_goal, &higher);
            let inputs_lower = calculate_inputs(interner, domain_goal, &lower);
            if inputs_higher == inputs_lower {
                debug!(
                    "preferring solution: {:?} over {:?} because of higher prio",
                    higher, lower
                );
                (higher, ClausePriority::High)
            } else {
                (higher.combine(lower, interner), ClausePriority::High)
            }
        }
        (_, _, a, b) => (a.combine(b, interner), prio_a),
    };
    debug!(?result, "combined result");
    result
}

fn calculate_inputs<I: Interner>(
    interner: I,
    domain_goal: &DomainGoal<I>,
    solution: &Solution<I>,
) -> Vec<GenericArg<I>> {
    if let Some(subst) = solution.constrained_subst(interner) {
        let subst_goal = subst.value.subst.apply(domain_goal.clone(), interner);
        subst_goal.inputs(interner)
    } else {
        domain_goal.inputs(interner)
    }
}
