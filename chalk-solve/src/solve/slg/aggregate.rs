use crate::ext::*;
use crate::infer::InferenceTable;
use crate::solve::slg::SlgContext;
use crate::solve::slg::SlgContextOps;
use crate::solve::slg::SubstitutionExt;
use crate::solve::{Guidance, Solution};
use chalk_ir::cast::Cast;
use chalk_ir::interner::Interner;
use chalk_ir::*;

use chalk_engine::context::{self, AnswerResult, Context, ContextOps};
use chalk_engine::CompleteAnswer;
use std::fmt::Debug;

/// Draws as many answers as it needs from `answers` (but
/// no more!) in order to come up with a solution.
impl<I: Interner> context::AggregateOps<SlgContext<I>> for SlgContextOps<'_, I> {
    fn make_solution(
        &self,
        root_goal: &UCanonical<InEnvironment<Goal<I>>>,
        mut answers: impl context::AnswerStream<SlgContext<I>>,
        should_continue: impl std::ops::Fn() -> bool,
    ) -> Option<Solution<I>> {
        let interner = self.program.interner();
        let CompleteAnswer { subst, ambiguous } = match answers.next_answer(|| should_continue()) {
            AnswerResult::NoMoreSolutions => {
                // No answers at all
                return None;
            }
            AnswerResult::Answer(answer) => answer,
            AnswerResult::Floundered => CompleteAnswer {
                subst: self.identity_constrained_subst(root_goal),
                ambiguous: true,
            },
            AnswerResult::QuantumExceeded => {
                return Some(Solution::Ambig(Guidance::Unknown));
            }
        };

        // Exactly 1 unconditional answer?
        let next_answer = answers.peek_answer(|| should_continue());
        if next_answer.is_quantum_exceeded() {
            return Some(Solution::Ambig(Guidance::Suggested(
                subst.map(interner, |cs| cs.subst),
            )));
        }
        if next_answer.is_no_more_solutions() && !ambiguous {
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
        let mut subst = subst.map(interner, |cs| cs.subst);

        // Extract answers and merge them into `subst`. Stop once we have
        // a trivial subst (or run out of answers).
        let mut num_answers = 1;
        let guidance = loop {
            if subst.value.is_empty(interner) || is_trivial(interner, &subst) {
                break Guidance::Unknown;
            }

            if !answers
                .any_future_answer(|ref mut new_subst| new_subst.may_invalidate(interner, &subst))
            {
                break Guidance::Definite(subst);
            }

            if let Some(expected_answers) = self.expected_answers {
                if num_answers >= expected_answers {
                    panic!("Too many answers for solution.");
                }
            }

            let new_subst = match answers.next_answer(|| should_continue()) {
                AnswerResult::Answer(answer1) => answer1.subst,
                AnswerResult::Floundered => {
                    // FIXME: this doesn't trigger for any current tests
                    self.identity_constrained_subst(root_goal)
                }
                AnswerResult::NoMoreSolutions => {
                    break Guidance::Definite(subst);
                }
                AnswerResult::QuantumExceeded => {
                    break Guidance::Suggested(subst);
                }
            };
            subst = merge_into_guidance(
                interner,
                SlgContext::canonical(root_goal),
                subst,
                &new_subst,
            );
            num_answers += 1;
        };

        if let Some(expected_answers) = self.expected_answers {
            assert_eq!(
                expected_answers, num_answers,
                "Not enough answers for solution."
            );
        }
        Some(Solution::Ambig(guidance))
    }
}

/// Given a current substitution used as guidance for `root_goal`, and
/// a new possible answer to `root_goal`, returns a new set of
/// guidance that encompasses both of them. This is often more general
/// than the old guidance. For example, if we had a guidance of `?0 =
/// u32` and the new answer is `?0 = i32`, then the guidance would
/// become `?0 = ?X` (where `?X` is some fresh variable).
fn merge_into_guidance<I: Interner>(
    interner: &I,
    root_goal: &Canonical<InEnvironment<Goal<I>>>,
    guidance: Canonical<Substitution<I>>,
    answer: &Canonical<ConstrainedSubst<I>>,
) -> Canonical<Substitution<I>> {
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
        .iter(interner)
        .zip(subst1.iter(interner))
        .enumerate()
        .map(|(index, (value, value1))| {
            // We have two values for some variable X that
            // appears in the root goal. Find out the universe
            // of X.
            let universe = root_goal.binders.as_slice(interner)[index].into_inner();

            let ty = match value.data(interner) {
                ParameterKind::Ty(ty) => ty,
                ParameterKind::Lifetime(_) => {
                    // Ignore the lifetimes from the substitution: we're just
                    // creating guidance here anyway.
                    return infer
                        .new_variable(universe)
                        .to_lifetime(interner)
                        .cast(interner);
                }
            };

            let ty1 = value1.assert_ty_ref(interner);

            // Combine the two types into a new type.
            let mut aggr = AntiUnifier {
                infer: &mut infer,
                universe,
                interner,
            };
            aggr.aggregate_tys(&ty, ty1).cast(interner)
        })
        .collect();

    let aggr_subst = Substitution::from(interner, aggr_parameters);

    infer.canonicalize(interner, &aggr_subst).quantified
}

fn is_trivial<I: Interner>(interner: &I, subst: &Canonical<Substitution<I>>) -> bool {
    // A subst is trivial if..
    subst
        .value
        .iter(interner)
        .enumerate()
        .all(|(index, parameter)| match parameter.data(interner) {
            // All types are mapped to distinct variables.  Since this
            // has been canonicalized, those will also be the first N
            // variables.
            ParameterKind::Ty(t) => match t.bound(interner) {
                None => false,
                Some(bound_var) => {
                    if let Some(index1) = bound_var.index_if_innermost() {
                        index == index1
                    } else {
                        false
                    }
                }
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
struct AntiUnifier<'infer, 'intern, I: Interner> {
    infer: &'infer mut InferenceTable<I>,
    universe: UniverseIndex,
    interner: &'intern I,
}

impl<I: Interner> AntiUnifier<'_, '_, I> {
    fn aggregate_tys(&mut self, ty0: &Ty<I>, ty1: &Ty<I>) -> Ty<I> {
        let interner = self.interner;
        match (ty0.data(interner), ty1.data(interner)) {
            // If we see bound things on either side, just drop in a
            // fresh variable. This means we will sometimes
            // overgeneralize.  So for example if we have two
            // solutions that are both `(X, X)`, we just produce `(Y,
            // Z)` in all cases.
            (TyData::InferenceVar(_), TyData::InferenceVar(_)) => self.new_variable(),

            // Ugh. Aggregating two types like `for<'a> fn(&'a u32,
            // &'a u32)` and `for<'a, 'b> fn(&'a u32, &'b u32)` seems
            // kinda hard. Don't try to be smart for now, just plop a
            // variable in there and be done with it.
            (TyData::BoundVar(_), TyData::BoundVar(_))
            | (TyData::Function(_), TyData::Function(_))
            | (TyData::Dyn(_), TyData::Dyn(_)) => self.new_variable(),

            (TyData::Apply(apply1), TyData::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (
                TyData::Alias(AliasTy::Projection(proj1)),
                TyData::Alias(AliasTy::Projection(proj2)),
            ) => self.aggregate_projection_tys(proj1, proj2),

            (
                TyData::Alias(AliasTy::Opaque(opaque_ty1)),
                TyData::Alias(AliasTy::Opaque(opaque_ty2)),
            ) => self.aggregate_opaque_ty_tys(opaque_ty1, opaque_ty2),

            (TyData::Placeholder(placeholder1), TyData::Placeholder(placeholder2)) => {
                self.aggregate_placeholder_tys(placeholder1, placeholder2)
            }

            // Mismatched base kinds.
            (TyData::InferenceVar(_), _)
            | (TyData::BoundVar(_), _)
            | (TyData::Dyn(_), _)
            | (TyData::Function(_), _)
            | (TyData::Apply(_), _)
            | (TyData::Alias(_), _)
            | (TyData::Placeholder(_), _) => self.new_variable(),
        }
    }

    fn aggregate_application_tys(
        &mut self,
        apply1: &ApplicationTy<I>,
        apply2: &ApplicationTy<I>,
    ) -> Ty<I> {
        let interner = self.interner;
        let ApplicationTy {
            name: name1,
            substitution: substitution1,
        } = apply1;
        let ApplicationTy {
            name: name2,
            substitution: substitution2,
        } = apply2;

        self.aggregate_name_and_substs(name1, substitution1, name2, substitution2)
            .map(|(&name, substitution)| {
                TyData::Apply(ApplicationTy { name, substitution }).intern(interner)
            })
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_placeholder_tys(
        &mut self,
        index1: &PlaceholderIndex,
        index2: &PlaceholderIndex,
    ) -> Ty<I> {
        let interner = self.interner;
        if index1 != index2 {
            self.new_variable()
        } else {
            TyData::Placeholder(index1.clone()).intern(interner)
        }
    }

    fn aggregate_projection_tys(
        &mut self,
        proj1: &ProjectionTy<I>,
        proj2: &ProjectionTy<I>,
    ) -> Ty<I> {
        let interner = self.interner;
        let ProjectionTy {
            associated_ty_id: name1,
            substitution: substitution1,
        } = proj1;
        let ProjectionTy {
            associated_ty_id: name2,
            substitution: substitution2,
        } = proj2;

        self.aggregate_name_and_substs(name1, substitution1, name2, substitution2)
            .map(|(&associated_ty_id, substitution)| {
                TyData::Alias(AliasTy::Projection(ProjectionTy {
                    associated_ty_id,
                    substitution,
                }))
                .intern(interner)
            })
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_opaque_ty_tys(
        &mut self,
        opaque_ty1: &OpaqueTy<I>,
        opaque_ty2: &OpaqueTy<I>,
    ) -> Ty<I> {
        let OpaqueTy {
            opaque_ty_id: name1,
            substitution: substitution1,
        } = opaque_ty1;
        let OpaqueTy {
            opaque_ty_id: name2,
            substitution: substitution2,
        } = opaque_ty2;

        self.aggregate_name_and_substs(name1, substitution1, name2, substitution2)
            .map(|(&opaque_ty_id, substitution)| {
                TyData::Alias(AliasTy::Opaque(OpaqueTy {
                    opaque_ty_id,
                    substitution,
                }))
                .intern(self.interner)
            })
            .unwrap_or_else(|| self.new_variable())
    }

    fn aggregate_name_and_substs<N>(
        &mut self,
        name1: N,
        substitution1: &Substitution<I>,
        name2: N,
        substitution2: &Substitution<I>,
    ) -> Option<(N, Substitution<I>)>
    where
        N: Copy + Eq + Debug,
    {
        let interner = self.interner;
        if name1 != name2 {
            return None;
        }

        let name = name1;

        assert_eq!(
            substitution1.len(interner),
            substitution2.len(interner),
            "does {:?} take {} substitution or {}? can't both be right",
            name,
            substitution1.len(interner),
            substitution2.len(interner)
        );

        let substitution = Substitution::from(
            interner,
            substitution1
                .iter(interner)
                .zip(substitution2.iter(interner))
                .map(|(p1, p2)| self.aggregate_parameters(p1, p2)),
        );

        Some((name, substitution))
    }

    fn aggregate_parameters(&mut self, p1: &Parameter<I>, p2: &Parameter<I>) -> Parameter<I> {
        let interner = self.interner;
        match (p1.data(interner), p2.data(interner)) {
            (ParameterKind::Ty(ty1), ParameterKind::Ty(ty2)) => {
                self.aggregate_tys(ty1, ty2).cast(interner)
            }
            (ParameterKind::Lifetime(l1), ParameterKind::Lifetime(l2)) => {
                self.aggregate_lifetimes(l1, l2).cast(interner)
            }
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => {
                panic!("mismatched parameter kinds: p1={:?} p2={:?}", p1, p2)
            }
        }
    }

    fn aggregate_lifetimes(&mut self, l1: &Lifetime<I>, l2: &Lifetime<I>) -> Lifetime<I> {
        let interner = self.interner;
        match (l1.data(interner), l2.data(interner)) {
            (LifetimeData::InferenceVar(_), _) | (_, LifetimeData::InferenceVar(_)) => {
                self.new_lifetime_variable()
            }

            (LifetimeData::BoundVar(_), _) | (_, LifetimeData::BoundVar(_)) => {
                self.new_lifetime_variable()
            }

            (LifetimeData::Placeholder(_), LifetimeData::Placeholder(_)) => {
                if l1 == l2 {
                    l1.clone()
                } else {
                    self.new_lifetime_variable()
                }
            }

            (LifetimeData::Phantom(..), _) | (_, LifetimeData::Phantom(..)) => unreachable!(),
        }
    }

    fn new_variable(&mut self) -> Ty<I> {
        let interner = self.interner;
        self.infer.new_variable(self.universe).to_ty(interner)
    }

    fn new_lifetime_variable(&mut self) -> Lifetime<I> {
        let interner = self.interner;
        self.infer.new_variable(self.universe).to_lifetime(interner)
    }
}

/// Test the equivalent of `Vec<i32>` vs `Vec<u32>`
#[test]
fn vec_i32_vs_vec_u32() {
    use chalk_ir::interner::ChalkIr;
    let mut infer: InferenceTable<ChalkIr> = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        infer: &mut infer,
        universe: UniverseIndex::root(),
        interner: &ChalkIr,
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
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut infer: InferenceTable<ChalkIr> = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        interner,
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
    use chalk_ir::interner::ChalkIr;
    let interner = &ChalkIr;
    let mut infer: InferenceTable<ChalkIr> = InferenceTable::new();
    let mut anti_unifier = AntiUnifier {
        interner,
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
