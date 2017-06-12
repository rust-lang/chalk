use std::sync::Arc;

use itertools::Itertools;

use errors::*;
use ir::*;
use solve::{Solution, Guidance};
use solve::solver::{Solver, CycleStrategy};

impl Program {
    pub(super) fn find_specializations<F>(&self, mut record_specialization: F) -> Result<()>
        where F: FnMut(ItemId, ItemId) -> Result<()>
    {
        let mut solver = Solver::new(&Arc::new(self.environment()), CycleStrategy::Tabling);

        // Create a vector of references to impl datums, sorted by trait ref
        let impl_data = self.impl_data.iter().sorted_by(|&(_, lhs), &(_, rhs)| {
            lhs.binders.value.trait_ref.trait_id.cmp(&rhs.binders.value.trait_ref.trait_id)
        });

        // Group impls by trait.
        let impl_groupings = impl_data.into_iter().group_by(|&(_, impl_datum)| {
            impl_datum.binders.value.trait_ref.trait_id
        });

        for (trait_id, impls) in &impl_groupings {
            let impls: Vec<(&ItemId, &ImplDatum)> = impls.collect();

            // Iterate over every pair of impls for the same trait
            for ((&l_id, lhs), (&r_id, rhs)) in impls.into_iter().tuple_combinations() {

                // First, determine if they overlap using the "intersection_of" goal.
                // A successful result means that these two impls are overlapping
                match intersection_of(&mut solver, lhs, rhs) {
                    Ok(sol) => {

                        // If they're overlapping, check if each specializes the other.
                        // One success means this is a specialization, two errors (no
                        // specialization) or two successes (identical impls) is an
                        // error.
                        //
                        // Successful specializations are recorded using the function
                        // passed as an argument to this method.
                        match specializes(&mut solver, lhs, rhs, sol) {
                            Specialization::LeftToRight     => record_specialization(l_id, r_id),
                            Specialization::RightToLeft     => record_specialization(r_id, l_id),
                            Specialization::None            => {
                                let trait_id = self.type_kinds.get(&trait_id).unwrap().name;
                                Err(Error::from_kind(ErrorKind::OverlappingImpls(trait_id)))
                            }
                        }
                    }
                    Err(_)  => Ok(())
                }?;
            }
        }

        Ok(())
    }
}

// Test for overlap.
//
// If this goal succeeds, these two impls overlap.
//
// We combine the binders of the two impls & treat them as existential
// quantifiers. Then we attempt to unify the input types to the trait provided
// by each impl, as well as prove that the where clauses from both impls all
// hold.
//
// Examples:
//
//  Impls:
//      impl<T> Foo for T { }
//      impl Foo for i32 { }
//  Generates:
//      exists<T> { T = i32 }
//
//  Impls:
//      impl<T1, U> Foo<T1> for Vec<U> { }
//      impl<T2> Foo<T2> for Vec<i32> { }
//  Generates:
//      exists<T1, U, T2> { Vec<U> = Vec<i32>, T1 = T2 }
//
//
//  Impls:
//      impl<T> Foo for Vec<T> where T: Bar { }
//      impl<U> Foo for Vec<U> where U: Baz { }
//  Generates:
//      exists<T, U> { Vec<T> = Vec<U>, T: Bar, U: Baz }
//
fn intersection_of(solver: &mut Solver, lhs: &ImplDatum, rhs: &ImplDatum) -> Result<Solution> {
    fn params(impl_datum: &ImplDatum) -> &[Parameter] {
        &impl_datum.binders.value.trait_ref.parameters
    }

    debug_assert!(params(lhs).len() == params(rhs).len());

    let lhs_len = lhs.binders.len();

    // Join the two impls' binders together
    let mut binders = lhs.binders.binders.clone();
    binders.extend(rhs.binders.binders.clone());

    // Upshift the rhs variables in params to account for the joined binders
    let lhs_params = params(lhs).iter().cloned();
    let rhs_params = params(rhs).iter().map(|param| param.up_shift(lhs_len));

    // Create an equality goal for every input type the trait, attempting
    // to unify the inputs to both impls with one another
    let params_goals = lhs_params.zip(rhs_params)
                        .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

    // Upshift the rhs variables in where clauses
    let lhs_where_clauses = lhs.binders.value.where_clauses.iter().cloned();
    let rhs_where_clauses = rhs.binders.value.where_clauses.iter().map(|wc| wc.up_shift(lhs_len));

    // Create a goal for each clause in both where clauses
    let wc_goals = lhs_where_clauses.chain(rhs_where_clauses)
                .map(|wc| Goal::Leaf(LeafGoal::DomainGoal(wc)));

    // Join all the goals we've created together with And, then quantify them
    // over the joined binders. This is our query.
    let goal = params_goals.chain(wc_goals)
                .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                .expect("Every trait takes at least one input type")
                .quantify(QuantifierKind::Exists, binders);

    solver.solve_closed_goal(InEnvironment::empty(goal))
}

enum Specialization {
    LeftToRight,
    RightToLeft,
    None,
}

// Test for specialization.
fn specializes(
    solver: &mut Solver,
    lhs: &ImplDatum,
    rhs: &ImplDatum,
    overlap: Solution
) -> Specialization {
    if let Guidance::Definite(subst) = overlap.into_guidance() {
        panic!()
    } else { Specialization::None }
}
