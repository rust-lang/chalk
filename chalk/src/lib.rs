#![feature(conservative_impl_trait)]
#![feature(static_in_const)]

extern crate chalk_parse;
extern crate docopt;
extern crate ena;
extern crate lalrpop_intern;
extern crate lalrpop_util;
#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;

/// Create a deref impl. We do this a lot.
macro_rules! deref_to {
    ($source:ident<$($param:ident),*>.$field:ident => $target:ty) => {
        impl<$($param),*> ::std::ops::Deref for $source<$($param),*> {
            type Target = $target;

            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    };

    ($source:ident.$field:ident => $target:ty) => {
        impl ::std::ops::Deref for $source {
            type Target = $target;

            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    };
}

lazy_static! {
    static ref DEBUG_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG").is_ok()
    };
}

macro_rules! debug {
    ($($t:tt)*) => {
        if *::DEBUG_ENABLED {
            println!($($t)*);
        }
    }
}

#[macro_use]
mod formula;

mod cli;
mod infer;
mod solve;
mod subst;

pub use self::formula::*;

pub fn solve_dfs(clauses: Vec<Clause<Application>>, root_goal: Goal<Application>) -> Vec<String> {
    use solve::Environment;
    use solve::{Solver, Strategy};
    use std::sync::Arc;

    let root_environment = Arc::new(Environment::new(None, clauses));
    Solver::solve(root_environment, root_goal, Strategy::DepthFirstSearch)
}

pub fn solve_rust(clauses: Vec<Clause<Application>>, root_goal: Goal<Application>) -> Vec<String> {
    use solve::Environment;
    use solve::{Solver, Strategy};
    use std::sync::Arc;

    let root_environment = Arc::new(Environment::new(None, clauses));
    Solver::solve(root_environment, root_goal, Strategy::Rust)
}

pub use self::cli::main;
