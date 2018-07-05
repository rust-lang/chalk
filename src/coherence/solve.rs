use std::sync::Arc;

use fold::shift::Shift;
use itertools::Itertools;
use errors::*;
use ir::*;
use cast::*;
use solve::{SolverChoice, Solution};

struct DisjointSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

impl Program {
    pub(super) fn visit_specializations<F>(
        &self,
        solver_choice: SolverChoice,
        mut record_specialization: F,
    ) -> Result<()>
    where
        F: FnMut(ItemId, ItemId),
    {
        let mut solver = DisjointSolver {
            env: Arc::new(self.environment()),
            solver_choice,
        };

        // Create a vector of references to impl datums, sorted by trait ref.
        let impl_data = self.impl_data
            .iter()
            .filter(|&(_, impl_datum)| {
                // Ignore impls for marker traits as they are allowed to overlap.
                let trait_id = impl_datum.binders.value.trait_ref.trait_ref().trait_id;
                let trait_datum = &self.trait_data[&trait_id];
                !trait_datum.binders.value.flags.marker
            })
            .sorted_by(|&(_, lhs), &(_, rhs)| {
                lhs.binders
                    .value
                    .trait_ref
                    .trait_ref()
                    .trait_id
                    .cmp(&rhs.binders.value.trait_ref.trait_ref().trait_id)
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
                if !lhs.binders.value.trait_ref.is_positive()
                    && !rhs.binders.value.trait_ref.is_positive()
                {
                    continue;
                }

                // Check if the impls overlap, then if they do, check if one specializes
                // the other. Note that specialization can only run one way - if both
                // specialization checks return *either* true or false, that's an error.
                if !solver.disjoint(lhs, rhs) {
                    match (solver.specializes(lhs, rhs), solver.specializes(rhs, lhs)) {
                        (true, false) => record_specialization(l_id, r_id),
                        (false, true) => record_specialization(r_id, l_id),
                        (_, _) => {
                            let trait_id = self.type_kinds.get(&trait_id).unwrap().name;
                            return Err(Error::from_kind(ErrorKind::OverlappingImpls(trait_id)));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl DisjointSolver {
    // Test if two impls are disjoint. If the test does not succeed, there is an overlap.
    //
    // We combine the binders of the two impls & treat them as existential
    // quantifiers. Then we attempt to unify the input types to the trait provided
    // by each impl, as well as prove that the where clauses from both impls all
    // hold. At the end, we negate the query because we only want to return `true` if
    // it is provable that there is no overlap.
    //
    // Examples:
    //
    //  Impls:
    //      impl<T> Foo for T { }
    //      impl Foo for i32 { }
    //  Generates:
    //      compatible { not { exists<T> { T = i32 } } }
    //
    //  Impls:
    //      impl<T1, U> Foo<T1> for Vec<U> { }
    //      impl<T2> Foo<T2> for Vec<i32> { }
    //  Generates:
    //      compatible { not { exists<T1, U, T2> { Vec<U> = Vec<i32>, T1 = T2 } } }
    //
    //  Impls:
    //      impl<T> Foo for Vec<T> where T: Bar { }
    //      impl<U> Foo for Vec<U> where U: Baz { }
    //  Generates:
    //      compatible { not { exists<T, U> { Vec<T> = Vec<U>, T: Bar, U: Baz } } }
    //
    fn disjoint(&self, lhs: &ImplDatum, rhs: &ImplDatum) -> bool {
        debug_heading!("overlaps(lhs={:#?}, rhs={:#?})", lhs, rhs);

        let lhs_len = lhs.binders.len();

        // Join the two impls' binders together
        let mut binders = lhs.binders.binders.clone();
        binders.extend(rhs.binders.binders.clone());

        // Upshift the rhs variables in params to account for the joined binders
        let lhs_params = params(lhs).iter().cloned();
        let rhs_params = params(rhs).iter().map(|param| param.up_shift(lhs_len));

        // Create an equality goal for every input type the trait, attempting
        // to unify the inputs to both impls with one another
        let params_goals = lhs_params
            .zip(rhs_params)
            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Upshift the rhs variables in where clauses
        let lhs_where_clauses = lhs.binders.value.where_clauses.iter().cloned();
        let rhs_where_clauses = rhs.binders
            .value
            .where_clauses
            .iter()
            .map(|wc| wc.up_shift(lhs_len));

        // Create a goal for each clause in both where clauses
        let wc_goals = lhs_where_clauses
            .chain(rhs_where_clauses)
            .map(|wc| wc.cast());

        // Join all the goals we've created together with And, then quantify them
        // over the joined binders. This is our query.
        let goal = params_goals
            .chain(wc_goals)
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("Every trait takes at least one input type")
            .quantify(QuantifierKind::Exists, binders)
            .negate()
            .compatible();

        // Unless we can prove NO solution, we consider things to overlap.
        let canonical_goal = &goal.into_closed_goal();
        let result = self.solver_choice
            .solve_root_goal(&self.env, canonical_goal)
            .unwrap()
            .is_some();
        debug!("overlaps: result = {:?}", result);
        result
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
        debug_heading!(
            "specializes(less_special={:#?}, more_special={:#?})",
            less_special,
            more_special
        );

        // Negative impls cannot specialize.
        if !less_special.binders.value.trait_ref.is_positive()
            || !more_special.binders.value.trait_ref.is_positive()
        {
            return false;
        }

        let more_len = more_special.binders.len();

        // Create parameter equality goals.
        let more_special_params = params(more_special).iter().cloned();
        let less_special_params = params(less_special).iter().map(|p| p.up_shift(more_len));
        let params_goals = more_special_params
            .zip(less_special_params)
            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Create the where clause goals.
        let more_special_wc = more_special
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .casted()
            .collect();
        let less_special_wc = less_special
            .binders
            .value
            .where_clauses
            .iter()
            .map(|wc| wc.up_shift(more_len).cast());

        // Join all of the goals together.
        let goal = params_goals
            .chain(less_special_wc)
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("Every trait takes at least one input type")
            .quantify(QuantifierKind::Exists, less_special.binders.binders.clone())
            .implied_by(more_special_wc)
            .quantify(QuantifierKind::ForAll, more_special.binders.binders.clone());

        let canonical_goal = &goal.into_closed_goal();
        let result = match self.solver_choice
            .solve_root_goal(&self.env, canonical_goal)
            .unwrap()
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        debug!("specializes: result = {:?}", result);

        result
    }
}

fn params(impl_datum: &ImplDatum) -> &[Parameter] {
    &impl_datum.binders.value.trait_ref.trait_ref().parameters
}
