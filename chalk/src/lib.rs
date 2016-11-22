#![feature(conservative_impl_trait)]
#![feature(static_in_const)]

extern crate chalk_parse;
extern crate docopt;
extern crate lalrpop_intern;
extern crate lalrpop_util;
extern crate ena;
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

macro_rules! debug {
    ($($t:tt)*) => {
        // println!($($t)*);
    }
}

#[macro_use]
mod formula;

mod cli;
mod infer;
mod solve;
mod subst;

pub use self::formula::*;

pub fn solve(clauses: Vec<Clause<Application>>, root_goal: Goal<Application>) -> Vec<String> {
    use solve::Environment;
    use solve::Solver;
    use std::sync::Arc;

    let root_environment = Arc::new(Environment::new(None, clauses));
    Solver::solve(root_environment, root_goal)
}

pub use self::cli::main;
