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

use chalk_engine::solve::SLGSolver;
use chalk_ir::interner::HasInterner;
use chalk_ir::Binders;
use chalk_recursive::{Cache, RecursiveSolver};
use chalk_solve::Solver;
use interner::ChalkIr;

pub use interner::{Identifier, RawId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Adt,
    FnDef,
    Closure,
    Trait,
    Opaque,
    Coroutine,
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
        max_size: usize,
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
    pub fn recursive_default() -> Self {
        SolverChoice::Recursive {
            overflow_depth: 100,
            caching_enabled: true,
            max_size: 30,
        }
    }

    /// Returns a recursive solver with specific parameters.
    pub fn recursive(max_size: usize, overflow_depth: usize) -> Self {
        SolverChoice::Recursive {
            overflow_depth,
            caching_enabled: true,
            max_size,
        }
    }

    pub fn into_solver(self) -> Box<dyn Solver<ChalkIr>> {
        match self {
            SolverChoice::SLG {
                max_size,
                expected_answers,
            } => Box::new(SLGSolver::new(max_size, expected_answers)),
            SolverChoice::Recursive {
                overflow_depth,
                caching_enabled,
                max_size,
            } => Box::new(RecursiveSolver::new(
                overflow_depth,
                max_size,
                if caching_enabled {
                    Some(Cache::default())
                } else {
                    None
                },
            )),
        }
    }
}

impl Default for SolverChoice {
    fn default() -> Self {
        SolverChoice::slg(10, None)
    }
}
