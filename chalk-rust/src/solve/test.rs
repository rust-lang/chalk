use chalk_rust_parse;
use errors::*;
use ir;
use lower::*;
use solve::goal::Prove;
use solve::solver::Solver;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_rust_parse::parse_goal(text)?.lower(program)
}

macro_rules! test {
    (program $program:tt $(goal $goal:tt yields { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$((stringify!($goal), $expected)),*])
    }
}

fn solve_goal(program_text: &str,
              goals: Vec<(&str, &str)>)
{
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let program = Arc::new(parse_and_lower_program(&program_text[1..program_text.len()-1]).unwrap());
    ir::set_current_program(&program, || {
        for (goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = parse_and_lower_goal(&program, &goal_text[1..goal_text.len()-1]).unwrap();
            let result = match Prove::new(&mut Solver::new(&program), goal).solve() {
                Ok(v) => format!("{:#?}", v),
                Err(e) => format!("{}", e),
            };
            println!("expected:\n{:?}", expected);
            println!("actual:\n{:#?}", result);

            // remove all whitespace:
            let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
            let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
            assert!(!expected1.is_empty() && result1.starts_with(&expected1));
        }
    });
}

#[test]
fn prove_clone() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            struct Vec<T> { }
            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl Clone for Foo { }
        }

        goal {
            Vec<Foo>: Clone
        } yields {
            "Yes"
        }

        goal {
            Foo: Clone
        } yields {
            "Yes"
        }

        goal {
            Bar: Clone
        } yields {
            "`Bar as Clone` is not implemented"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "`Vec<Bar> as Clone` is not implemented"
        }
    }
}

#[test]
fn prove_infer() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            trait Map<T> { }
            impl Map<Bar> for Foo { }
            impl Map<Foo> for Bar { }
        }

        goal {
            exists<A, B> { A: Map<B> }
        } yields {
            "Maybe"
        }

        goal {
            exists<A> { A: Map<Bar> }
        } yields {
            "Yes"
        }

        goal {
            exists<A> { Foo: Map<A> }
        } yields {
            "Yes"
        }
    }
}

#[test]
fn prove_forall() {
    test! {
        program {
            struct Foo { }
            struct Vec<T> { }

            trait Marker { }
            impl<T> Marker for Vec<T> { }

            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
        }

        goal {
            forall<T> { T: Marker }
        } yields {
            "`!1 as Marker` is not implemented"
        }

        goal {
            forall<T> { Vec<T>: Marker }
        } yields {
            "Yes"
        }

        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            "`Vec<!1> as Clone` is not implemented"
        }

        goal {
            forall<T> {
                if (T: Clone) {
                    Vec<T>: Clone
                }
            }
        } yields {
            "`Vec<!1> as Clone` is not implemented" // FIXME
        }
    }
}
