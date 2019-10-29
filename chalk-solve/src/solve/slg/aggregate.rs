use crate::ext::*;
use crate::infer::InferenceTable;
use crate::solve::slg::SlgContext;
use crate::solve::slg::SlgContextOps;
use crate::solve::slg::SubstitutionExt;
use crate::solve::{Guidance, Solution};
use chalk_ir::cast::Cast;
use chalk_ir::family::ChalkIr;
use chalk_ir::*;

use chalk_engine::context;
use chalk_engine::Answer;
use std::fmt::Debug;

/// Draws as many answers as it needs from `answers` (but
/// no more!) in order to come up with a solution.
impl<'me> context::AggregateOps<SlgContext> for SlgContextOps<'me> {
    fn make_solution(
        &self,
        root_goal: &Canonical<InEnvironment<Goal<ChalkIr>>>,
        mut answers: impl context::AnswerStream<SlgContext>,
    ) -> Option<Solution> {
        // No answers at all?
        if answers.peek_answer().is_none() {
            return None;
        }
        let Answer { subst, ambiguous } = answers.next_answer().unwrap();

        // Exactly 1 unconditional answer?
        if answers.peek_answer().is_none() && !ambiguous {
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
        let guidance = loop {
            if subst.value.is_empty() || is_trivial(&subst) {
                break Guidance::Unknown;
            }

            if !answers.any_future_answer(|ref mut new_subst| new_subst.may_invalidate(&subst)) {
                break Guidance::Definite(subst);
            }

            match answers.next_answer() {
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
    root_goal: &Canonical<InEnvironment<Goal<ChalkIr>>>,
    guidance: Canonical<Substitution<ChalkIr>>,
    answer: &Canonical<ConstrainedSubst<ChalkIr>>,
) -> Canonical<Substitution<ChalkIr>> {
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

            let ty = match &value.0 {
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

fn is_trivial(subst: &Canonical<Substitution<ChalkIr>>) -> bool {
    // A subst is trivial if..
    subst
        .value
        .parameters
        .iter()
        .enumerate()
        .all(|(index, parameter)| match &parameter.0 {
            // All types are mapped to distinct variables.  Since this
            // has been canonicalized, those will also be the first N
            // variables.
            ParameterKind::Ty(t) => match t.bound() {
                None => false,
                Some(depth) => depth == index,
            },

            // And no lifetime mappings. (This is too strict, but we never
            // product substs with lifetimes.)
            ParameterKind::Lifetime(_) => false,
        })
}

/// [Anti-unification] is the act of taking two things that do not
/// unify and finding a minimal generalization of them. So for
/// example `Vec<u32>` anti-unified with `Vec<i32>` might be
/// `Vec<?X>`. This is a **very simplistic** anti-unifier.
///
/// [Anti-unification]: https://en.wikipedia.org/wiki/Anti-unification_(computer_science)
struct AntiUnifier<'infer> {
    infer: &'infer mut InferenceTable,
    universe: UniverseIndex,
}

impl<'infer> AntiUnifier<'infer> {
    fn aggregate_tys(&mut self, ty0: &Ty<ChalkIr>, ty1: &Ty<ChalkIr>) -> Ty<ChalkIr> {
        match (ty0, ty1) {
            // If we see bound things on either side, just drop in a
            // fresh variable. This means we will sometimes
            // overgeneralize.  So for example if we have two
            // solutions that are both `(X, X)`, we just produce `(Y,
            // Z)` in all cases.
            (Ty::InferenceVar(_), Ty::InferenceVar(_)) => self.new_variable(),

            // Ugh. Aggregating two types like `for<'a> fn(&'a u32,
            // &'a u32)` and `for<'a, 'b> fn(&'a u32, &'b u32)` seems
            // kinda hard. Don't try to be smart for now, just plop a
            // variable in there and be done with it.
            (Ty::BoundVar(_), Ty::BoundVar(_))
            | (Ty::ForAll(_), Ty::ForAll(_))
            | (Ty::Dyn(_), Ty::Dyn(_))
            | (Ty::Opaque(_), Ty::Opaque(_)) => self.new_variable(),

            (Ty::Apply(apply1), Ty::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (Ty::Projection(apply1), Ty::Projection(apply2)) => {
                self.aggregate_projection_tys(apply1, apply2)
            }

            // Mismatched base kinds.
            (Ty::InferenceVar(_), _)
            | (Ty::BoundVar(_), _)
            | (Ty::Dyn(_), _)
            | (Ty::Opaque(_), _)
            | (Ty::ForAll(_), _)
            | (Ty::Apply(_), _)
            | (Ty::Projection(_), _) => self.new_variable(),
        }
    }

    fn aggregate_application_tys(
        &mut self,
        apply1: &ApplicationTy<ChalkIr>,
        apply2: &ApplicationTy<ChalkIr>,
    ) -> Ty<ChalkIr> {
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

    fn aggregate_projection_tys(
        &mut self,
        proj1: &ProjectionTy<ChalkIr>,
        proj2: &ProjectionTy<ChalkIr>,
    ) -> Ty<ChalkIr> {
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

    fn aggregate_name_and_substs<N>(
        &mut self,
        name1: N,
        parameters1: &[Parameter<ChalkIr>],
        name2: N,
        parameters2: &[Parameter<ChalkIr>],
    ) -> Option<(N, Vec<Parameter<ChalkIr>>)>
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

    fn aggregate_parameters(
        &mut self,
        p1: &Parameter<ChalkIr>,
        p2: &Parameter<ChalkIr>,
    ) -> Parameter<ChalkIr> {
        match (&p1.0, &p2.0) {
            (ParameterKind::Ty(ty1), ParameterKind::Ty(ty2)) => self.aggregate_tys(ty1, ty2).cast(),
            (ParameterKind::Lifetime(l1), ParameterKind::Lifetime(l2)) => {
                self.aggregate_lifetimes(l1, l2).cast()
            }
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => {
                panic!("mismatched parameter kinds: p1={:?} p2={:?}", p1, p2)
            }
        }
    }

    fn aggregate_lifetimes(
        &mut self,
        l1: &Lifetime<ChalkIr>,
        l2: &Lifetime<ChalkIr>,
    ) -> Lifetime<ChalkIr> {
        match (l1, l2) {
            (Lifetime::InferenceVar(_), _) | (_, Lifetime::InferenceVar(_)) => {
                self.new_lifetime_variable()
            }

            (Lifetime::BoundVar(_), _) | (_, Lifetime::BoundVar(_)) => self.new_lifetime_variable(),

            (Lifetime::Placeholder(_), Lifetime::Placeholder(_)) => {
                if l1 == l2 {
                    *l1
                } else {
                    self.new_lifetime_variable()
                }
            }

            (Lifetime::Phantom(..), _) | (_, Lifetime::Phantom(..)) => unreachable!(),
        }
    }

    fn new_variable(&mut self) -> Ty<ChalkIr> {
        self.infer.new_variable(self.universe).to_ty()
    }

    fn new_lifetime_variable(&mut self) -> Lifetime<ChalkIr> {
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
    assert_eq!(ty!(apply (item 0) (infer 0)), ty);
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
    let ty = anti_unifier.aggregate_tys(
        &ty!(apply (item 0) (infer 0)),
        &ty!(apply (item 0) (infer 1)),
    );

    // But this `var 0` is from `infer.
    assert_eq!(ty!(apply (item 0) (infer 0)), ty);
}
