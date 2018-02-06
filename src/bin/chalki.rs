#![feature(match_default_bindings)]

extern crate chalk;
extern crate chalk_parse;
extern crate docopt;
extern crate rustyline;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

use std::io::Read;
use std::fs::File;
use std::sync::Arc;
use std::process::exit;

use chalk::ir;
use chalk::lower::*;
use chalk::solve::SolverChoice;
use docopt::Docopt;
use rustyline::error::ReadlineError;

const USAGE: &'static str = "
chalk repl

Usage:
  chalki [options]
  chalki (-h | --help)

Options:
  --help              Show this screen.
  --program=PATH      Specifies the path to the `.chalk` file containing traits/impls.
  --goal=GOAL         Specifies a goal to evaluate (may be given more than once).
  --overflow-depth=N  Specifies the overflow depth [default: 10].
  --solver S          Selects a solver (recursive, slg) [default: recursive]
  --no-cache          Disable caching.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_program: Option<String>,
    flag_goal: Vec<String>,
    flag_overflow_depth: usize,
    flag_solver: String,
    flag_no_cache: bool,
}

error_chain! {
    links {
        Parse(chalk_parse::errors::Error, chalk_parse::errors::ErrorKind);
        Chalk(chalk::errors::Error, chalk::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Rustyline(ReadlineError);
    }
}

struct Program {
    text: String,
    ir: Arc<ir::Program>,
    env: Arc<ir::ProgramEnvironment>,
}

impl Program {
    fn new(text: String, solver_choice: SolverChoice) -> Result<Program> {
        let ir = Arc::new(chalk_parse::parse_program(&text)?.lower(solver_choice)?);
        let env = Arc::new(ir.environment());
        Ok(Program { text, ir, env })
    }
}

quick_main!(run);

fn run() -> Result<()> {
    let args: &Args = &Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_overflow_depth == 0 {
        eprintln!("error: overflow depth must be at least 1");
        exit(1);
    }

    let mut prog = None;

    if let None = args.solver_choice() {
        eprintln!("error: invalid solver choice `{}`", args.flag_solver);
        eprintln!("try `recursive` or `slg`");
        exit(1);
    }

    if let Some(program) = &args.flag_program {
        match load_program(args, program) {
            Ok(p) => prog = Some(p),
            Err(err) => {
                eprintln!("error loading program: {}", err);
                exit(1);
            }
        }
    }

    if args.flag_goal.is_empty() {
        readline_loop(&mut rustyline::Editor::new(), "?- ", |rl, line| {
            if let Err(e) = process(args, line, rl, &mut prog) {
                eprintln!("error: {}", e);
            }
        })
    } else {
        let prog = match prog {
            Some(p) => p,
            None => {
                eprintln!("error: cannot eval with a program, use `--program`");
                exit(1);
            }
        };

        ir::tls::set_current_program(&prog.ir, || -> Result<()> {
            for g in &args.flag_goal {
                if let Err(e) = goal(&args, g, &prog) {
                    eprintln!("error: {}", e);
                    exit(1);
                }
            }
            Ok(())
        })?;

        Ok(())
    }
}

/// Repeatedly calls `f`, passing in each line, using the given promt, until EOF is received
fn readline_loop<F>(rl: &mut rustyline::Editor<()>, prompt: &str, mut f: F) -> Result<()>
where
    F: FnMut(&mut rustyline::Editor<()>, &str),
{
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                rl.add_history_entry(&line);
                f(rl, &line);
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => Err(e)?,
        }
    }

    Ok(())
}

/// Process a single command
fn process(
    args: &Args,
    command: &str,
    rl: &mut rustyline::Editor<()>,
    prog: &mut Option<Program>,
) -> Result<()> {
    if command == "help" {
        help()
    } else if command == "program" {
        let solver_choice = args.solver_choice().unwrap();
        *prog = Some(Program::new(read_program(rl)?, solver_choice)?);
    } else if command.starts_with("load ") {
        let filename = &command["load ".len()..];
        *prog = Some(load_program(args, filename)?);
    } else {
        let prog = prog.as_ref().ok_or("no program currently loaded")?;
        ir::tls::set_current_program(&prog.ir, || -> Result<()> {
            match command {
                "print" => println!("{}", prog.text),
                "lowered" => println!("{:#?}", prog.env),
                _ => goal(args, command, prog)?,
            }
            Ok(())
        })?
    }

    Ok(())
}

fn load_program(args: &Args, filename: &str) -> Result<Program> {
    let mut text = String::new();
    File::open(filename)?.read_to_string(&mut text)?;
    let solver_choice = args.solver_choice().unwrap();
    Ok(Program::new(text, solver_choice)?)
}

fn help() {
    println!("Commands:");
    println!("  help         print this output");
    println!("  program      provide a program via stdin");
    println!("  load <file>  load program from <file>");
    println!("  print        print the current program");
    println!("  lowered      print the lowered program");
    println!("  <goal>       attempt to solve <goal>");
}

fn read_program(rl: &mut rustyline::Editor<()>) -> Result<String> {
    println!("Enter a program; press Ctrl-D when finished");
    let mut text = String::new();
    readline_loop(rl, "| ", |_, line| {
        text += line;
        text += "\n";
    })?;
    Ok(text)
}

fn goal(args: &Args, text: &str, prog: &Program) -> Result<()> {
    let goal = chalk_parse::parse_goal(text)?.lower(&*prog.ir)?;
    let peeled_goal = goal.into_peeled_goal();
    let solver_choice = args.solver_choice().unwrap();
    match solver_choice.solve_root_goal(&prog.env, &peeled_goal) {
        Ok(Some(v)) => println!("{}\n", v),
        Ok(None) => println!("No possible solution.\n"),
        Err(e) => println!("Solver failed: {}", e),
    }
    Ok(())
}

impl Args {
    fn solver_choice(&self) -> Option<SolverChoice> {
        match &self.flag_solver[..] {
            "slg" => Some(SolverChoice::SLG {
                max_size: self.flag_overflow_depth,
            }),

            "recursive" => Some(SolverChoice::Recursive {
                overflow_depth: self.flag_overflow_depth,
                caching_enabled: !self.flag_no_cache,
            }),

            _ => None
        }
    }
}
