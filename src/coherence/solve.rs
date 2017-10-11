use std::sync::Arc;

use itertools::Itertools;
use errors::*;
use ir::*;
use solve::solver::{self, Solver, CycleStrategy};

impl Program {
    pub(super) fn visit_specializations<F>(&self, mut record_specialization: F) -> Result<()>
        where F: FnMut(ItemId, ItemId)
    {
        let mut solver = Solver::new(
            &Arc::new(self.environment()),
            CycleStrategy::Tabling,
            solver::get_overflow_depth()
        );

        // Create a vector of references to impl datums, sorted by trait ref
        let impl_data = self.impl_data.iter().sorted_by(|&(_, lhs), &(_, rhs)| {
            lhs.binders.value.trait_ref.trait_ref().trait_id.cmp(&rhs.binders.value.trait_ref.trait_ref().trait_id)
        });

        // Group impls by trait.
        let impl_groupings = impl_data.into_iter().group_by(|&(_, impl_datum)| {
            impl_datum.binders.value.trait_ref.trait_ref().trait_id
        });


        // Iterate over every pair of impls for the same trait.
        for (trait_id, impls) in &impl_groupings {
            let impls: Vec<(&ItemId, &ImplDatum)> = impls.collect();

            for ((&l_id, lhs), (&r_id, rhs)) in impls.into_iter().tuple_combinations() {
                // Two negative impls never overlap.
                if !lhs.binders.value.trait_ref.is_positive() && !rhs.binders.value.trait_ref.is_positive() {
                    continue;
                }

                // Check if the impls overlap, then if they do, check if one specializes
                // the other. Note that specialization can only run one way - if both
                // specialization checks return *either* true or false, that's an error.
                if solver.overlaps(lhs, rhs) {
                    match (solver.specializes(lhs, rhs), solver.specializes(rhs, lhs)) {
                        (true, false)   => record_specialization(l_id, r_id),
                        (false, true)   => record_specialization(r_id, l_id),
                        (_, _)          => {
                            let trait_id = self.type_kinds.get(&trait_id).unwrap().name;
                            return Err(Error::from_kind(ErrorKind::OverlappingImpls(trait_id)))
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Solver {
    // Test for overlap.
    //
    // If this test succeeds, these two impls overlap.
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
    fn overlaps(&mut self, lhs: &ImplDatum, rhs: &ImplDatum) -> bool {
        debug_heading!("overlaps(lhs={:?}, rhs={:?})", lhs, rhs);

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

        self.solve_closed_goal(InEnvironment::empty(goal))
            .ok()
            .map(|sol| {
                debug!("solution = {:?}", sol);
                sol.has_definite()
            })
            .unwrap_or(false)
    }

    // Test for specialization.
    //
    // If this test suceeds, the second impl specializes the first.
    //
    // Example lowering:
    //
    // more: impl<T: Clone> Foo for Vec<T>
    // less: impl<U: Clone> Foo for U
    //
    // forall<T> {
    //  if (T: Clone) {
    //    exists<U> {
    //      Vec<T> = U, U: Clone
    //    }
    //  }
    // }
    fn specializes(&mut self, less_special: &ImplDatum, more_special: &ImplDatum) -> bool {
        // Negative impls cannot specialize.
        if !less_special.binders.value.trait_ref.is_positive() || !more_special.binders.value.trait_ref.is_positive() {
            return false;
        }

        let more_len = more_special.binders.len();

        // Create parameter equality goals.
        let more_special_params = params(more_special).iter().cloned();
        let less_special_params = params(less_special).iter().map(|p| p.up_shift(more_len));
        let params_goals = more_special_params.zip(less_special_params)
                            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Create the where clause goals.
        let more_special_wc = more_special.binders.value.where_clauses.clone();
        let less_special_wc = less_special.binders.value.where_clauses.iter().map(|wc| {
            Goal::Leaf(LeafGoal::DomainGoal(wc.up_shift(more_len)))
        });

        // Join all of the goals together.
        let goal = params_goals.chain(less_special_wc)
                    .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                    .expect("Every trait takes at least one input type")
                    .quantify(QuantifierKind::Exists, less_special.binders.binders.clone())
                    .implied_by(more_special_wc)
                    .quantify(QuantifierKind::ForAll, more_special.binders.binders.clone());

        self.solve_closed_goal(InEnvironment::empty(goal)).ok().map_or(false, |sol| sol.is_unique())
    }
}

fn params(impl_datum: &ImplDatum) -> &[Parameter] {
    &impl_datum.binders.value.trait_ref.trait_ref().parameters
}
