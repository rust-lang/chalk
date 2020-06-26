#![recursion_limit = "1024"]
#![cfg_attr(feature = "bench", feature(test))]

pub mod db;
pub mod error;
pub mod interner;
pub mod lowering;
pub mod program;
pub mod program_environment;
pub mod query;
pub mod test_macros;
pub mod tls;

use chalk_engine::forest::Forest;
use chalk_engine::solve::SLGSolverImpl;
use chalk_ir::interner::HasInterner;
use chalk_ir::{Binders, Canonical, ConstrainedSubst, Goal, InEnvironment, UCanonical};
use chalk_solve::recursive::RecursiveContext;
use chalk_solve::solve::RecursiveSolverImpl;
use chalk_solve::{RustIrDatabase, Solution, Solver, SubstitutionResult};
use interner::ChalkIr;

pub use interner::{Identifier, RawId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    FnDef,
    Closure,
    Trait,
    Opaque,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unit;

impl HasInterner for Unit {
    type Interner = ChalkIr;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub binders: Binders<Unit>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SolverChoice {
    /// Run the SLG solver, producing a Solution.
    SLG {
        max_size: usize,
        expected_answers: Option<usize>,
    },
    /// Run the recursive solver.
    Recursive {
        overflow_depth: usize,
        caching_enabled: bool,
    },
}

impl SolverChoice {
    /// Returns specific SLG parameters.
    pub fn slg(max_size: usize, expected_answers: Option<usize>) -> Self {
        SolverChoice::SLG {
            max_size,
            expected_answers,
        }
    }

    /// Returns the default SLG parameters.
    pub fn slg_default() -> Self {
        SolverChoice::slg(10, None)
    }

    /// Returns the default recursive solver setup.
    pub fn recursive() -> Self {
        SolverChoice::Recursive {
            overflow_depth: 100,
            caching_enabled: true,
        }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::slg(10, None)
    }
}

#[derive(Debug)]
pub enum SolverImpl {
    Slg(SLGSolverImpl<ChalkIr>),
    Recursive(RecursiveSolverImpl<ChalkIr>),
}

impl Solver<ChalkIr> for SolverImpl {
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<ChalkIr>,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> Option<Solution<ChalkIr>> {
        match self {
            Self::Slg(solve) => solve.solve(program, goal),
            Self::Recursive(solve) => solve.solve(program, goal),
        }
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<ChalkIr>,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
        should_continue: impl std::ops::Fn() -> bool,
    ) -> Option<Solution<ChalkIr>> {
        match self {
            Self::Slg(solve) => solve.solve_limited(program, goal, should_continue),
            Self::Recursive(solve) => solve.solve_limited(program, goal, should_continue),
        }
    }

    fn solve_multiple(
        &mut self,
        program: &dyn RustIrDatabase<ChalkIr>,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
        f: impl FnMut(SubstitutionResult<Canonical<ConstrainedSubst<ChalkIr>>>, bool) -> bool,
    ) -> bool {
        match self {
            Self::Slg(solve) => solve.solve_multiple(program, goal, f),
            Self::Recursive(solve) => solve.solve_multiple(program, goal, f),
        }
    }
}

impl Into<SolverImpl> for SolverChoice {
    fn into(self) -> SolverImpl {
        match self {
            SolverChoice::SLG {
                max_size,
                expected_answers,
            } => SolverImpl::Slg(SLGSolverImpl {
                forest: Forest::new(),
                max_size,
                expected_answers,
            }),
            SolverChoice::Recursive {
                overflow_depth,
                caching_enabled,
            } => SolverImpl::Recursive(RecursiveSolverImpl {
                ctx: Box::new(RecursiveContext::new(overflow_depth, caching_enabled)),
            }),
        }
    }
}
