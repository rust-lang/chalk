extern crate rustyline;
extern crate chalk_parse;
extern crate chalk;

#[macro_use]
extern crate error_chain;

use std::io::Read;
use std::fs::File;
use std::sync::Arc;

use chalk::ir;
use chalk::lower::*;
use chalk::solve::solver::{self, Solver, CycleStrategy};

use rustyline::error::ReadlineError;

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
    fn new(text: String) -> Result<Program> {
        let ir = Arc::new(chalk_parse::parse_program(&text)?.lower()?);
        let env = Arc::new(ir.environment());
        Ok(Program { text, ir, env })
    }
}

quick_main!(run);

fn run() -> Result<()> {
    let mut prog = None;
    readline_loop(&mut rustyline::Editor::new(), "?- ", |rl, line| {
        if let Err(e) = process(line, rl, &mut prog) {
            println!("error: {}", e);
        }
    })
}

/// Repeatedly calls `f`, passing in each line, using the given promt, until EOF is received
fn readline_loop<F>(rl: &mut rustyline::Editor<()>, prompt: &str, mut f: F) -> Result<()>
    where F: FnMut(&mut rustyline::Editor<()>, &str)
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
fn process(command: &str, rl: &mut rustyline::Editor<()>, prog: &mut Option<Program>) -> Result<()> {
    if command == "help" {
        help()
    } else if command == "program" {
        *prog = Some(Program::new(read_program(rl)?)?);
    } else if command.starts_with("load ") {
        let filename = &command["load ".len()..];
        let mut text = String::new();
        File::open(filename)?.read_to_string(&mut text)?;
        *prog = Some(Program::new(text)?);
    } else {
        let prog = prog.as_ref().ok_or("no program currently loaded")?;
        ir::set_current_program(&prog.ir, || -> Result<()> {
            match command {
                "print" => println!("{}", prog.text),
                "lowered" => println!("{:#?}", prog.env),
                _ => goal(command, prog)?,
            }
            Ok(())
        })?
    }

    Ok(())
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

fn goal(text: &str, prog: &Program) -> Result<()> {
    let goal = chalk_parse::parse_goal(text)?.lower(&*prog.ir)?;
    let overflow_depth = 10;
    solver::set_overflow_depth(overflow_depth);
    let mut solver = Solver::new(&prog.env, CycleStrategy::Tabling, solver::get_overflow_depth());
    let goal = ir::InEnvironment::new(&ir::Environment::new(), *goal);
    match solver.solve_closed_goal(goal) {
        Ok(v) => println!("{}\n", v),
        Err(e) => println!("No possible solution: {}\n", e),
    }
    Ok(())
}
