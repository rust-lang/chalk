use crate::Solution;
use tracing::debug;

use chalk_ir::interner::Interner;
use chalk_ir::{ClausePriority, DomainGoal, Fallible, GenericArg, Goal, GoalData};

pub(crate) fn with_priorities_for_goal<I: Interner>(
    interner: &I,
    goal: &Goal<I>,
    a: Fallible<Solution<I>>,
    prio_a: ClausePriority,
    b: Fallible<Solution<I>>,
    prio_b: ClausePriority,
) -> (Fallible<Solution<I>>, ClausePriority) {
    let domain_goal = match goal.data(interner) {
        GoalData::DomainGoal(domain_goal) => domain_goal,
        _ => {
            // non-domain goals currently have no priorities, so we always take the new solution here
            return (b, prio_b);
        }
    };
    match (a, b) {
        (Ok(a), Ok(b)) => {
            let (solution, prio) = with_priorities(interner, domain_goal, a, prio_a, b, prio_b);
            (Ok(solution), prio)
        }
        (Ok(solution), Err(_)) => (Ok(solution), prio_a),
        (Err(_), Ok(solution)) => (Ok(solution), prio_b),
        (Err(_), Err(e)) => (Err(e), prio_b),
    }
}

pub(super) fn with_priorities<I: Interner>(
    interner: &I,
    domain_goal: &DomainGoal<I>,
    a: Solution<I>,
    prio_a: ClausePriority,
    b: Solution<I>,
    prio_b: ClausePriority,
) -> (Solution<I>, ClausePriority) {
    match (prio_a, prio_b, a, b) {
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
    }
}

fn calculate_inputs<I: Interner>(
    interner: &I,
    domain_goal: &DomainGoal<I>,
    solution: &Solution<I>,
) -> Vec<GenericArg<I>> {
    if let Some(subst) = solution.constrained_subst(interner) {
        let subst_goal = subst.value.subst.apply(&domain_goal, interner);
        subst_goal.inputs(interner)
    } else {
        domain_goal.inputs(interner)
    }
}
