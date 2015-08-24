use super::super::*;
use intern::{intern, InternedString};

macro_rules! term {
    ($($tokens:tt)+) => {
        {
            let term_fn = term_tt!($($tokens,)*);
            let mut env = vec![];
            let term = term_fn(&mut env);
            term.unwrap()
        }
    }
}

fn maybe_apply(t1: Term, t2: Option<Term>) -> Option<Term> {
    match t2 {
        None => Some(t1),
        Some(t2) => Some(Term::new(TermData::Apply(t1, t2)))
    }
}

macro_rules! term_tt {
    () => {
        |_: &mut Vec<InternedString>| None
    };

    (($($terms:tt)+), $($remainder:tt,)*) => {
        |env: &mut Vec<InternedString>| {
            let terms = term_tt!($($terms,)*);
            let t1 = terms(env).unwrap();
            let remainder = term_tt!($($remainder,)*);
            maybe_apply(t1, remainder(env))
        }
    };

    (fn, $x:ident, $($remainder:tt,)+) => {
        |env: &mut Vec<InternedString>| {
            env.push(intern(stringify!($x)));
            let remainder = term_tt!($($remainder,)*);
            let term = remainder(env).unwrap();
            env.pop();
            Some(Term::new(TermData::Lambda(term)))
        }
    };

    (ref, ($n:expr), $($remainder:tt,)+) => {
        |env: &mut Vec<InternedString>| {
            let remainder = term_tt!($($remainder,)*);
            maybe_apply(Term::new(TermData::FreeVariable($n)), remainder(env))
        }
    };

    ($x:ident, $($remainder:tt,)*) => {
        |env: &mut Vec<InternedString>| {
            let x_string = stringify!($x);
            let x = intern(x_string);
            let t1 = match env.iter().rev().position(|&y| x == y) {
                Some(index) => {
                    Term::new(TermData::BoundVariable(DebruijnIndex(index as u32)))
                }
                None => {
                    Term::new(TermData::Constant(x))
                }
            };
            let remainder = term_tt!($($remainder,)*);
            maybe_apply(t1, remainder(env))
        }
    };
}

#[test]
fn term_macro_1() {
    // Example from the paper
    let term = term!(fn x (fn y fn z (y x)) (fn w x));
    assert_eq!(&format!("{:?}", term),
               "(fn ((fn (fn (#1 #2))) (fn #1)))");
}

#[test]
fn hnf() {
    // Example from the paper
    let term = term!((fn x (x (c x))) ((fn y (ref(0) b)) a));

    term.head_normal_form();
    assert_eq!(
        &format!("{:?}", term),
        "((ref(0) <<b; 1: Dummy(0)::nil; 1>; 1: <a; 0: nil; 0>::nil; 0>) \
          <<b; 1: Dummy(0)::nil; 1>; 1: <a; 0: nil; 0>::nil; 0>)");
}

