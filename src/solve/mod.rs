use std::fmt;
use std::sync::Arc;
use ir::*;

crate mod infer;
crate mod recursive;
crate mod slg;
mod test;
mod truncate;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A (possible) solution for a proposed goal. Usually packaged in a `Result`,
/// where `Err` represents definite *failure* to prove a goal.
pub enum Solution {
    /// The goal indeed holds, and there is a unique value for all existential
    /// variables. In this case, we also record a set of lifetime constraints
    /// which must also hold for the goal to be valid.
    Unique(Canonical<ConstrainedSubst>),

    /// The goal may be provable in multiple ways, but regardless we may have some guidance
    /// for type inference. In this case, we don't return any lifetime
    /// constraints, since we have not "committed" to any particular solution
    /// yet.
    Ambig(Guidance),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// When a goal holds ambiguously (e.g., because there are multiple possible
/// solutions), we issue a set of *guidance* back to type inference.
pub enum Guidance {
    /// The existential variables *must* have the given values if the goal is
    /// ever to hold, but that alone isn't enough to guarantee the goal will
    /// actually hold.
    Definite(Canonical<Substitution>),

    /// There are multiple plausible values for the existentials, but the ones
    /// here are suggested as the preferred choice heuristically. These should
    /// be used for inference fallback only.
    Suggested(Canonical<Substitution>),

    /// There's no useful information to feed back to type inference
    Unknown,
}

impl Solution {
    /// There are multiple candidate solutions, which may or may not agree on
    /// the values for existential variables; attempt to combine them. This
    /// operation does not depend on the order of its arguments.
    //
    // This actually isn't as precise as it could be, in two ways:
    //
    // a. It might be that while there are multiple distinct candidates, they
    //    all agree about *some things*. To be maximally precise, we would
    //    compute the intersection of what they agree on. It's not clear though
    //    that this is actually what we want Rust's inference to do, and it's
    //    certainly not what it does today.
    //
    // b. There might also be an ambiguous candidate and a successful candidate,
    //    both with the same refined-goal. In that case, we could probably claim
    //    success, since if the conditions of the ambiguous candidate were met,
    //    we know the success would apply.  Example: `?0: Clone` yields ambiguous
    //    candidate `Option<?0>: Clone` and successful candidate `Option<?0>:
    //    Clone`.
    //
    // But you get the idea.
    crate fn combine(self, other: Solution) -> Solution {
        use self::Guidance::*;

        if self == other {
            return self;
        }

        // Otherwise, always downgrade to Ambig:

        let guidance = match (self.into_guidance(), other.into_guidance()) {
            (Definite(ref subst1), Definite(ref subst2)) if subst1 == subst2 => {
                Definite(subst1.clone())
            }
            (Suggested(ref subst1), Suggested(ref subst2)) if subst1 == subst2 => {
                Suggested(subst1.clone())
            }
            _ => Unknown,
        };
        Solution::Ambig(guidance)
    }

    /// There are multiple candidate solutions, which may or may not agree on
    /// the values for existential variables; attempt to combine them, while
    /// favoring `self` for the purposes of giving suggestions to type
    /// inference. This is used in particular to favor the `where` clause
    /// environment over `impl`s in guiding inference in ambiguous situations.
    ///
    /// It should always be the case that `x.favor_over(y)` is at least as
    /// informative as `x.combine(y)`, in terms of guidance to type inference.
    crate fn favor_over(self, other: Solution) -> Solution {
        use self::Guidance::*;

        if self == other {
            return self;
        }

        // Otherwise, always downgrade to Ambig:

        let guidance = match (self.into_guidance(), other.into_guidance()) {
            (Definite(subst), _) | (Suggested(subst), _) => Suggested(subst),
            _ => Unknown,
        };
        Solution::Ambig(guidance)
    }

    /// View this solution purely in terms of type inference guidance
    crate fn into_guidance(self) -> Guidance {
        match self {
            Solution::Unique(constrained) => Guidance::Definite(Canonical {
                value: constrained.value.subst,
                binders: constrained.binders,
            }),
            Solution::Ambig(guidance) => guidance,
        }
    }

    /// Extract a constrained substitution from this solution, even if ambiguous.
    crate fn constrained_subst(&self) -> Option<Canonical<ConstrainedSubst>> {
        match *self {
            Solution::Unique(ref constrained) => Some(constrained.clone()),
            Solution::Ambig(Guidance::Definite(ref canonical))
            | Solution::Ambig(Guidance::Suggested(ref canonical)) => {
                let value = ConstrainedSubst {
                    subst: canonical.value.clone(),
                    constraints: vec![],
                };
                Some(Canonical {
                    value,
                    binders: canonical.binders.clone(),
                })
            }
            Solution::Ambig(_) => None,
        }
    }

    /// Determine whether this solution contains type information that *must*
    /// hold.
    crate fn has_definite(&self) -> bool {
        match *self {
            Solution::Unique(_) => true,
            Solution::Ambig(Guidance::Definite(_)) => true,
            _ => false,
        }
    }

    crate fn is_ambig(&self) -> bool {
        match *self {
            Solution::Ambig(_) => true,
            _ => false,
        }
    }

    crate fn is_unique(&self) -> bool {
        match *self {
            Solution::Unique(..) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Solution::Unique(constrained) => write!(f, "Unique; {}", constrained,),
            Solution::Ambig(Guidance::Definite(subst)) => {
                write!(f, "Ambiguous; definite substitution {}", subst)
            }
            Solution::Ambig(Guidance::Suggested(subst)) => {
                write!(f, "Ambiguous; suggested substitution {}", subst)
            }
            Solution::Ambig(Guidance::Unknown) => write!(f, "Ambiguous; no inference guidance"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SolverChoice {
    /// Chalk's recursive solving strategy.
    Recursive {
        overflow_depth: usize,
        caching_enabled: bool,
    },

    /// Run the SLG solver, producing a Solution.
    SLG { max_size: usize },
}

impl SolverChoice {
    /// Attempts to solve the given root goal, which must be in
    /// canonical form. The solution is searching for unique answers
    /// to any free existential variables in this goal.
    ///
    /// # Returns
    ///
    /// - `Ok(None)` is the goal cannot be proven.
    /// - `Ok(Some(solution))` if we succeeded in finding *some* answers,
    ///   although `solution` may reflect ambiguity and unknowns.
    /// - `Err` if there was an internal error solving the goal, which does not
    ///   reflect success nor failure.
    pub fn solve_root_goal(
        self,
        env: &Arc<ProgramEnvironment<DomainGoal>>,
        canonical_goal: &UCanonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> ::errors::Result<Option<Solution>> {
        match self {
            SolverChoice::Recursive {
                overflow_depth,
                caching_enabled,
            } => {
                let mut solver = recursive::Solver::new(env, overflow_depth, caching_enabled);
                match solver.solve_root_goal(canonical_goal) {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                }
            }

            SolverChoice::SLG { max_size } => Ok(slg::forest::Forest::solve_root_goal(
                env.clone(),
                max_size,
                &canonical_goal,
            )),
        }
    }

    /// Returns the default recursive parameters.
    pub fn recursive() -> Self {
        SolverChoice::Recursive {
            overflow_depth: 10,
            caching_enabled: true,
        }
    }

    /// Returns the default SLG parameters.
    pub fn slg() -> Self {
        SolverChoice::SLG { max_size: 10 }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::recursive()
    }
}
