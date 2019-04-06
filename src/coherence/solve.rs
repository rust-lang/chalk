use super::CoherenceError;
use crate::program::Program;
use crate::program_environment::ProgramEnvironment;
use chalk_ir::cast::*;
use chalk_ir::fold::shift::Shift;
use chalk_ir::*;
use chalk_rust_ir::*;
use chalk_solve::ext::*;
use chalk_solve::solve::{Solution, SolverChoice};
use failure::Fallible;
use itertools::Itertools;
use std::sync::Arc;

struct DisjointSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

impl Program {
    pub(super) fn visit_specializations<F>(
        &self,
        env: Arc<ProgramEnvironment>,
        solver_choice: SolverChoice,
        mut record_specialization: F,
    ) -> Fallible<()>
    where
        F: FnMut(ImplId, ImplId),
    {
        let mut solver = DisjointSolver { env, solver_choice };

        // Create a vector of references to impl datums, sorted by trait ref.
        let impl_data = self
            .impl_data
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
        let impl_groupings = impl_data
            .into_iter()
            .group_by(|&(_, impl_datum)| impl_datum.binders.value.trait_ref.trait_ref().trait_id);

        // Iterate over every pair of impls for the same trait.
        for (trait_id, impls) in &impl_groupings {
            let impls: Vec<(&ImplId, &ImplDatum)> = impls.collect();

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
                            let trait_id = self.type_kinds.get(&trait_id.into()).unwrap().name;
                            Err(CoherenceError::OverlappingImpls(trait_id))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl DisjointSolver {
    // Test if the set of types that these two impls apply to overlap. If the test succeeds, these
    // two impls are disjoint.
    //
    // We combine the binders of the two impls & treat them as existential quantifiers. Then we
    // attempt to unify the input types to the trait provided by each impl, as well as prove that
    // the where clauses from both impls all hold. At the end, we apply the `compatible` modality
    // and negate the query. Negating the query means that we are asking chalk to prove that no
    // such overlapping impl exists. By applying `compatible { G }`, chalk attempts to prove that
    // "there exists a compatible world where G is provable." When we negate compatible, it turns
    // into the statement "for all compatible worlds, G is not provable." This is exactly what we
    // want since we want to ensure that there is no overlap in *all* compatible worlds, not just
    // that there is no overlap in *some* compatible world.
    //
    // Examples:
    //
    //  Impls:
    //      impl<T> Foo for T { }
    //      impl Foo for i32 { }
    //  Generates:
    //      not { compatible { exists<T> { T = i32 } } }
    //
    //  Impls:
    //      impl<T1, U> Foo<T1> for Vec<U> { }
    //      impl<T2> Foo<T2> for Vec<i32> { }
    //  Generates:
    //      not { compatible { exists<T1, U, T2> { Vec<U> = Vec<i32>, T1 = T2 } } }
    //
    //  Impls:
    //      impl<T> Foo for Vec<T> where T: Bar { }
    //      impl<U> Foo for Vec<U> where U: Baz { }
    //  Generates:
    //      not { compatible { exists<T, U> { Vec<T> = Vec<U>, T: Bar, U: Baz } } }
    //
    fn disjoint(&self, lhs: &ImplDatum, rhs: &ImplDatum) -> bool {
        debug_heading!("overlaps(lhs={:#?}, rhs={:#?})", lhs, rhs);

        let lhs_len = lhs.binders.len();

        // Join the two impls' binders together
        let mut binders = lhs.binders.binders.clone();
        binders.extend(rhs.binders.binders.clone());

        // Upshift the rhs variables in params to account for the joined binders
        let lhs_params = params(lhs).iter().cloned();
        let rhs_params = params(rhs).iter().map(|param| param.shifted_in(lhs_len));

        // Create an equality goal for every input type the trait, attempting
        // to unify the inputs to both impls with one another
        let params_goals = lhs_params
            .zip(rhs_params)
            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Upshift the rhs variables in where clauses
        let lhs_where_clauses = lhs.binders.value.where_clauses.iter().cloned();
        let rhs_where_clauses = rhs
            .binders
            .value
            .where_clauses
            .iter()
            .map(|wc| wc.shifted_in(lhs_len));

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
            .compatible()
            .negate();

        let canonical_goal = &goal.into_closed_goal();
        let solution = self
            .solver_choice
            .solve_root_goal(&*self.env, canonical_goal)
            .unwrap(); // internal errors in the solver are fatal
        let result = match solution {
            // Goal was proven with a unique solution, so no impl was found that causes these two
            // to overlap
            Some(Solution::Unique(_)) => true,
            // Goal was ambiguous, so there *may* be overlap
            Some(Solution::Ambig(_)) |
            // Goal cannot be proven, so there is some impl that causes overlap
            None => false,
        };
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
        let less_special_params = params(less_special).iter().map(|p| p.shifted_in(more_len));
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
            .map(|wc| wc.shifted_in(more_len).cast());

        // Join all of the goals together.
        let goal = params_goals
            .chain(less_special_wc)
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("Every trait takes at least one input type")
            .quantify(QuantifierKind::Exists, less_special.binders.binders.clone())
            .implied_by(more_special_wc)
            .quantify(QuantifierKind::ForAll, more_special.binders.binders.clone());

        let canonical_goal = &goal.into_closed_goal();
        let result = match self
            .solver_choice
            .solve_root_goal(&*self.env, canonical_goal)
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
