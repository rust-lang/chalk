use docopt::Docopt;
use formula;
use {solve_dfs, solve_rust};
use std::error::Error;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use std::time::{Duration, Instant};

mod parse;

const USAGE: &'static str = "
Usage: chalk [options]
       chalk --help

Loads a chalk program and tries to solve the given goal.

Options:
    -p <file>, --program <file> ...     Specifies that we should load program clauses from `file`.
    -g <goal>, --goal <goal>            Specifies the goal to try to solve. [default: goal(?X)]
    --prolog                            Use the Prolog strategy, rather than Rust
    -i, --interactive                   Enter yes/no after each solution
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_program: Vec<String>,
    flag_goal: String,
    flag_prolog: bool,
    flag_interactive: bool,
}

pub fn main() {
    let mut stderr = io::stderr();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args()).decode())
        .unwrap_or_else(|e| e.exit());

    match fallible_main(&args) {
        Ok(()) => { }
        Err(err) => {
            writeln!(stderr, "{}", err).unwrap();
            process::exit(1);
        }
    }
}

pub fn fallible_main(args: &Args) -> Result<(), Box<Error>> {
    let mut clauses = Vec::new();
    for path in &args.flag_program {
        let text = file_text(path)?;
        let ast = parse::parse_program(path, &text)?;
        clauses.extend(formula::lower_program(path, &ast)?);
    }

    let ast = parse::parse_goal("<goal>", &args.flag_goal)?;
    let goal = formula::lower_goal("<goal>", &ast)?;

    let mut solver: Box<Iterator<Item = String>> = if args.flag_prolog {
        Box::new(solve_dfs(clauses, goal))
    } else {
        Box::new(solve_rust(clauses, goal))
    };

    if args.flag_interactive {
        use std::io::BufRead;
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let mut stdin = stdin.lines();
        'next_solution: loop {
            let now = Instant::now();
            let solution = solver.next();
            let time = now.elapsed();
            match solution {
                Some(solution) => {
                    println!("found in {:0.3}s: {}", seconds(&time), solution);
                    loop {
                        println!("continue (y/n)?");
                        if let Some(line) = stdin.next() {
                            let line = line?;
                            let line = line.trim();
                            if line == "y" || line == "yes" {
                                continue 'next_solution;
                            } else if line == "n" || line == "no" {
                                break 'next_solution;
                            }
                        } else {
                            break 'next_solution;
                        }
                    }
                }
                None => {
                    println!("no more solutions found ({:0.3}s)", seconds(&time));
                    break;
                }
            }
        }
    } else {
        let now = Instant::now();
        let solutions: Vec<_> = solver.collect();
        let time = now.elapsed();
        println!("found {} solutions in {:0.3}s",
                 solutions.len(),
                 seconds(&time));
        for solution in solutions {
            println!("  - {}", solution);
        }
    }

    Ok(())
}

fn file_text(path: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn seconds(time: &Duration) -> f64 {
    time.as_secs() as f64 + (time.subsec_nanos() as f64) * 0.000000001
}
