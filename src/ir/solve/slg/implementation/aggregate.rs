use crate::ir::cast::Cast;
use crate::ir::*;
use crate::ir::solve::{Guidance, Solution};
use crate::ir::solve::infer::InferenceTable;

use chalk_engine::context;
use chalk_engine::SimplifiedAnswer;
use std::fmt::Debug;

use super::SlgContext;

/// Draws as many answers as it needs from `simplified_answers` (but
/// no more!) in order to come up with a solution.
impl context::AggregateOps<SlgContext> for SlgContext {
    fn make_solution(
        &self,
        root_goal: &Canonical<InEnvironment<Goal>>,
        mut simplified_answers: impl context::AnswerStream<SlgContext>,
    ) -> Option<Solution> {
        // No answers at all?
        if simplified_answers.peek_answer().is_none() {
            return None;
        }
        let SimplifiedAnswer { subst, ambiguous } = simplified_answers.next_answer().unwrap();

        // Exactly 1 unconditional answer?
        if simplified_answers.peek_answer().is_none() && !ambiguous {
            return Some(Solution::Unique(subst));
        }

        // Otherwise, we either have >1 answer, or else we have
        // ambiguity.  Either way, we are only going to be giving back
        // **guidance**, and with guidance, the caller doesn't get
        // back any region constraints. So drop them from our `subst`
        // variable.
        //
        // FIXME-- there is actually a 3rd possibility. We could have
        // >1 answer where all the answers have the same substitution,
        // but different region constraints. We should collapse those
        // cases into an `OR` region constraint at some point, but I
        // leave that for future work. This is basically
        // rust-lang/rust#21974.
        let mut subst = subst.map(|cs| cs.subst);

        // Extract answers and merge them into `subst`. Stop once we have
        // a trivial subst (or run out of answers).
        //
        // FIXME -- It would be nice if we could get some idea of the
        // "shape" of future answers to know if they *might* disrupt
        // existing substituion; the iterator interface is obviously too
        // limited for that, but the on-demand SLG solver probably could
        // give us that information.
        let guidance = loop {
            if subst.value.is_empty() || is_trivial(&subst) {
                break Guidance::Unknown;
            }

            if !simplified_answers.any_future_answer(|ref mut new_subst| {
                new_subst.may_invalidate(&subst)
            }) {
                break Guidance::Definite(subst);
            }

            match simplified_answers.next_answer() {
                Some(answer1) => {
                    subst = merge_into_guidance(root_goal, subst, &answer1.subst);
                }

                None => {
                    break Guidance::Definite(subst);
                }
            }
        };

        Some(Solution::Ambig(guidance))
    }
}

/// Given a current substitution used as guidance for `root_goal`, and
/// a new possible answer to `root_goal`, returns a new set of
/// guidance that encompasses both of them. This is often more general
/// than the old guidance. For example, if we had a guidance of `?0 =
/// u32` and the new answer is `?0 = i32`, then the guidance would
/// become `?0 = ?X` (where `?X` is some fresh variable).
fn merge_into_guidance(
    root_goal: &Canonical<InEnvironment<Goal>>,
    guidance: Canonical<Substitution>,
    answer: &Canonical<ConstrainedSubst>,
) -> Canonical<Substitution> {
    let mut infer = InferenceTable::new();
    let Canonical {
        value: ConstrainedSubst {
            subst: subst1,
            constraints: _,
        },
        binders: _,
    } = answer;

    // Collect the types that the two substitutions have in
    // common.
    let aggr_parameters: Vec<_> = guidance
        .value
        .parameters
        .iter()
        .zip(&subst1.parameters)
        .enumerate()
        .map(|(index, (value, value1))| {
            // We have two values for some variable X that
            // appears in the root goal. Find out the universe
            // of X.
            let universe = root_goal.binders[index].into_inner();

            let ty = match value {
                ParameterKind::Ty(ty) => ty,
                ParameterKind::Lifetime(_) => {
                    // Ignore the lifetimes from the substitution: we're just
                    // creating guidance here anyway.
                    return infer.new_variable(universe).to_lifetime().cast();
                }
            };

            let ty1 = value1.assert_ty_ref();

            // Combine the two types into a new type.
            let mut aggr = AntiUnifier {
                infer: &mut infer,
                universe,
            };
            aggr.aggregate_tys(&ty, ty1).cast()
        })
        .collect();

    let aggr_subst = Substitution {
        parameters: aggr_parameters,
    };

    infer.canonicalize(&aggr_subst).quantified
}

fn is_trivial(subst: &Canonical<Substitution>) -> bool {
    // A subst is trivial if..
    subst
        .value
        .parameters
        .iter()
        .enumerate()
        .all(|(index, parameter)| match parameter {
            // All types are mapped to distinct variables.  Since this
            // has been canonicalized, those will also be the first N
            // variables.
            ParameterKind::Ty(t) => match t.var() {
                None => false,
                Some(depth) => depth == index,
            },

            // And no lifetime mappings. (This is too strict, but we never
            // product substs with lifetimes.)
            ParameterKind::Lifetime(_) => false,
        })
}

/// [Anti-unification] is the act of taking two things that do not
/// unify and finding a minimal generarlization of them. So for
/// example `Vec<u32>` anti-unified with `Vec<i32>` might be
/// `Vec<?X>`. This is a **very simplistic** anti-unifier.
///
/// [Anti-unification]: https://en.wikipedia.org/wiki/Anti-unification_(computer_science)
struct AntiUnifier<'infer> {
    infer: &'infer mut InferenceTable,
    universe: UniverseIndex,
}

impl<'infer> AntiUnifier<'infer> {
    fn aggregate_tys(&mut self, ty0: &Ty, ty1: &Ty) -> Ty {
        match (ty0, ty1) {
            // If we see bound things on either side, just drop in a
            // fresh variable. This means we will sometimes
            // overgeneralize.  So for example if we have two
            // solutions that are both `(X, X)`, we just produce `(Y,
            // Z)` in all cases.
            (Ty::Var(_), Ty::Var(_)) => self.new_variable(),

            // Ugh. Aggregating two types like `for<'a> fn(&'a u32,
            // &'a u32)` and `for<'a, 'b> fn(&'a u32, &'b u32)` seems
            // kinda' hard. Don't try to be smart for now, just plop a
            // variable in there and be done with it.
            (Ty::ForAll(_), Ty::ForAll(_)) => self.new_variable(),

            (Ty::Apply(apply1), Ty::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (Ty::Projection(apply1), Ty::Projection(apply2)) => {
                self.aggregate_projection_tys(apply1, apply2)
            }

            (Ty::UnselectedProjection(apply1), Ty::UnselectedProjection(apply2)) => {
                self.aggregate_unselected_projection_tys(apply1, apply2)
            }

            // Mismatched base kinds.
            (Ty::Var(_), _)
            | (Ty::ForAll(_), _)
            | (Ty::Apply(_), _)
            | (Ty::Projection(_), _)
            | (Ty::UnselectedProjection(_), _) => self.new_variable(),
        }
    }

    fn aggregate_application_tys(&mut self, apply1: &ApplicationTy, apply2: &ApplicationTy) -> Ty {
        let ApplicationTy {
            name: name1,
            parameters: parameters1,
        } = apply1;
        let ApplicationTy {
            name: name2,
            parameters: parameters2,
        } = apply2;

        self.aggregate_name_and_substs(name1, parameters1, name2, parameters2)
            .map(|(&name, parameters)| Ty::Apply(ApplicationTy { name, parameters }))
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_projection_tys(&mut self, proj1: &ProjectionTy, proj2: &ProjectionTy) -> Ty {
        let ProjectionTy {
            associated_ty_id: name1,
            parameters: parameters1,
        } = proj1;
        let ProjectionTy {
            associated_ty_id: name2,
            parameters: parameters2,
        } = proj2;

        self.aggregate_name_and_substs(name1, parameters1, name2, parameters2)
            .map(|(&associated_ty_id, parameters)| {
                Ty::Projection(ProjectionTy {
                    associated_ty_id,
                    parameters,
                })
            })
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_unselected_projection_tys(
        &mut self,
        proj1: &UnselectedProjectionTy,
        proj2: &UnselectedProjectionTy,
    ) -> Ty {
        let UnselectedProjectionTy {
            type_name: name1,
            parameters: parameters1,
        } = proj1;
        let UnselectedProjectionTy {
            type_name: name2,
            parameters: parameters2,
        } = proj2;

        self.aggregate_name_and_substs(name1, parameters1, name2, parameters2)
            .map(|(&type_name, parameters)| {
                Ty::UnselectedProjection(UnselectedProjectionTy {
                    type_name,
                    parameters,
                })
            })
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_name_and_substs<N>(
        &mut self,
        name1: N,
        parameters1: &[Parameter],
        name2: N,
        parameters2: &[Parameter],
    ) -> Option<(N, Vec<Parameter>)>
    where
        N: Copy + Eq + Debug,
    {
        if name1 != name2 {
            return None;
        }

        let name = name1;

        assert_eq!(
            parameters1.len(),
            parameters2.len(),
            "does {:?} take {} parameters or {}? can't both be right",
            name,
            parameters1.len(),
            parameters2.len()
        );

        let parameters: Vec<_> = parameters1
            .iter()
            .zip(parameters2)
            .map(|(p1, p2)| self.aggregate_parameters(p1, p2))
            .collect();

        Some((name, parameters))
    }

    fn aggregate_parameters(&mut self, p1: &Parameter, p2: &Parameter) -> Parameter {
        match (p1, p2) {
            (ParameterKind::Ty(ty1), ParameterKind::Ty(ty2)) => {
                ParameterKind::Ty(self.aggregate_tys(ty1, ty2))
            }
            (ParameterKind::Lifetime(l1), ParameterKind::Lifetime(l2)) => {
                ParameterKind::Lifetime(self.aggregate_lifetimes(l1, l2))
            }
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => {
                panic!("mismatched parameter kinds: p1={:?} p2={:?}", p1, p2)
            }
        }
    }

    fn aggregate_lifetimes(&mut self, l1: &Lifetime, l2: &Lifetime) -> Lifetime {
        match (l1, l2) {
            (Lifetime::Var(_), _) | (_, Lifetime::Var(_)) => self.new_lifetime_variable(),

            (Lifetime::ForAll(ui1), Lifetime::ForAll(ui2)) => if ui1 == ui2 {
                Lifetime::ForAll(*ui1)
            } else {
                self.new_lifetime_variable()
            },
        }
    }

    fn new_variable(&mut self) -> Ty {
        self.infer.new_variable(self.universe).to_ty()
    }

    fn new_lifetime_variable(&mut self) -> Lifetime {
        self.infer.new_variable(self.universe).to_lifetime()
    }
}

/// Test the equivalent of `Vec<i32>` vs `Vec<u32>`
#[test]
fn vec_i32_vs_vec_u32() {
    let mut infer = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        infer: &mut infer,
        universe: UniverseIndex::root(),
    };

    let ty = anti_unifier.aggregate_tys(
        &ty!(apply (item 0) (apply (item 1))),
        &ty!(apply (item 0) (apply (item 2))),
    );
    assert_eq!(ty!(apply (item 0) (var 0)), ty);
}

/// Test the equivalent of `Vec<i32>` vs `Vec<i32>`
#[test]
fn vec_i32_vs_vec_i32() {
    let mut infer = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        infer: &mut infer,
        universe: UniverseIndex::root(),
    };

    let ty = anti_unifier.aggregate_tys(
        &ty!(apply (item 0) (apply (item 1))),
        &ty!(apply (item 0) (apply (item 1))),
    );
    assert_eq!(ty!(apply (item 0) (apply (item 1))), ty);
}

/// Test the equivalent of `Vec<X>` vs `Vec<Y>`
#[test]
fn vec_x_vs_vec_y() {
    let mut infer = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        infer: &mut infer,
        universe: UniverseIndex::root(),
    };

    // Note that the `var 0` and `var 1` in these types would be
    // referring to canonicalized free variables, not variables in
    // `infer`.
    let ty = anti_unifier.aggregate_tys(&ty!(apply (item 0) (var 0)), &ty!(apply (item 0) (var 1)));

    // But this `var 0` is from `infer.
    assert_eq!(ty!(apply (item 0) (var 0)), ty);
}
