extern crate chalk;
extern crate chalk_engine;
extern crate chalk_ir;
extern crate chalk_parse;
extern crate chalk_solve;
extern crate docopt;
extern crate rustyline;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::sync::Arc;

use chalk::db::ChalkDatabase;
use chalk::query::{LoweringDatabase, ProgramSolverChoice, ProgramText};
use chalk::rust_ir;
use chalk::rust_ir::lowering::*;
use chalk_engine::fallible::NoSolution;
use chalk_solve::ext::*;
use chalk_solve::solve::SolverChoice;
use docopt::Docopt;
use rustyline::error::ReadlineError;
use salsa::Database;

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
  --no-cache          Disable caching.
";

/// This struct represents the various command line options available.
#[derive(Debug, Deserialize)]
struct Args {
    flag_program: Option<String>,
    flag_goal: Vec<String>,
    flag_overflow_depth: usize,
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

/// A loaded and parsed program.
struct Program {
    text: String,
    ir: Arc<rust_ir::Program>,
    env: Arc<chalk_ir::ProgramEnvironment>,
}

impl Program {
    /// Creates a new Program struct, given a `.chalk` file as a String and
    /// a [`SolverChoice`].
    ///
    /// [`SolverChoice`]: struct.solve.SolverChoice.html
    fn new(text: String, solver_choice: SolverChoice) -> Result<Program> {
        let mut db = ChalkDatabase::default();

        db.query_mut(ProgramText)
            .set((), Arc::new(text.to_string()));
        db.query_mut(ProgramSolverChoice).set((), solver_choice);

        let ir = db.checked_program().unwrap();
        let env = Arc::new(ir.environment());
        Ok(Program { text, ir, env })
    }
}

quick_main!(run);

fn run() -> Result<()> {
    // Parse the command line arguments.
    let args: &Args = &Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Validate arguments.
    if args.flag_overflow_depth == 0 {
        eprintln!("error: overflow depth must be at least 1");
        exit(1);
    }

    // Load the .chalk file, if given.
    let mut prog = None;
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
        // The user specified no goal. Enter interactive mode.
        readline_loop(&mut rustyline::Editor::new(), "?- ", |rl, line| {
            if let Err(e) = process(args, line, rl, &mut prog) {
                eprintln!("error: {}", e);
            }
        })
    } else {
        // Check that a program was provided.
        // TODO: It's customary to print Usage info when an error like this
        // happens.
        let prog =
            prog.ok_or("error: cannot eval without a program; use `--program` to specify one.")?;

        // Evaluate the goal(s). If any goal returns an error, print the error
        // and exit.
        chalk_ir::tls::set_current_program(&prog.ir, || -> Result<()> {
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

/// Reads input lines from the user. Lines start with the string given by `prompt`.
/// Each line the user enters is passed to the function `f` for processing.
///
/// The loop terminates (and the program ends) when EOF is reached or if an error
/// occurs while reading the next line.
fn readline_loop<F>(rl: &mut rustyline::Editor<()>, prompt: &str, mut f: F) -> Result<()>
where
    F: FnMut(&mut rustyline::Editor<()>, &str),
{
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                // Save the line to the history list.
                rl.add_history_entry(&line);

                // Process the line.
                f(rl, &line);
            }

            // EOF: We're done.
            Err(ReadlineError::Eof) => break,

            // Some other error occured.
            Err(e) => Err(e)?,
        }
    }

    Ok(())
}

/// Process a single command. `args` is a struct containing the command-line
/// arguments, and `prog` is a parsed `.chalk` file.
// TODO: Could we pass in an Options struct or something? The Args struct
// still has Strings where it should have Enums... (e.g. solver_choice)
fn process(
    args: &Args,
    command: &str,
    rl: &mut rustyline::Editor<()>,
    prog: &mut Option<Program>,
) -> Result<()> {
    if command == "help" || command == "h" {
        // Print out interpreter commands.
        // TODO: Implement "help <command>" for more specific help.
        help()
    } else if command == "program" {
        // Load a .chalk file via stdin, until EOF is found.
        *prog = Some(Program::new(read_program(rl)?, args.solver_choice())?);
    } else if command.starts_with("load ") {
        // Load a .chalk file.
        let filename = &command["load ".len()..];
        *prog = Some(load_program(args, filename)?);
    } else if command.starts_with("debug ") {
        match command.split_whitespace().nth(1) {
            Some(level) => std::env::set_var("CHALK_DEBUG", level),
            None => println!("debug <level> set debug level to <level>"),
        }
    } else {
        // The command is either "print", "lowered", or a goal.

        // Check that a program has been loaded.
        let prog = prog
            .as_ref()
            .ok_or("no program currently loaded; type 'help' to see available commands")?;

        // Attempt to parse the program.
        chalk_ir::tls::set_current_program(&prog.ir, || -> Result<()> {
            match command {
                // Print out the loaded program.
                "print" => println!("{}", prog.text),

                // TODO: Write a line of documentation here.
                "lowered" => println!("{:#?}", prog.env),

                // Assume this is a goal.
                // TODO: Print out "type 'help' to see available commands" if it
                // fails to parse?
                _ => goal(args, command, prog)?,
            }
            Ok(())
        })?
    }

    Ok(())
}

/// Load the file into a string, and parse it.
// TODO: Could we pass in an Options struct or something? The Args struct
// still has Strings where it should have Enums... (e.g. solver_choice)
fn load_program(args: &Args, filename: &str) -> Result<Program> {
    let mut text = String::new();
    File::open(filename)?.read_to_string(&mut text)?;
    Ok(Program::new(text, args.solver_choice())?)
}

/// Print out help for commands in interpreter mode.
// TODO: Implement "help <command>" for more info.
fn help() {
    println!("Commands:");
    println!("  help          print this output");
    println!("  program       provide a program via stdin");
    println!("  load <file>   load program from <file>");
    println!("  print         print the current program");
    println!("  lowered       print the lowered program");
    println!("  <goal>        attempt to solve <goal>");
    println!("  debug <level> set debug level to <level>");
}

/// Read a program from the command-line. Stop reading when EOF is read. If
/// an error occurs while reading, a Result::Err is returned.
fn read_program(rl: &mut rustyline::Editor<()>) -> Result<String> {
    println!("Enter a program; press Ctrl-D when finished");
    let mut text = String::new();
    readline_loop(rl, "| ", |_, line| {
        text += line;
        text += "\n";
    })?;
    Ok(text)
}

/// Parse a goal and attempt to solve it, using the specified solver.
// TODO: Could we pass in an Options struct or something? The Args struct
// still has Strings where it should have Enums... (e.g. solver_choice)
fn goal(args: &Args, text: &str, prog: &Program) -> Result<()> {
    let goal = chalk_parse::parse_goal(text)?.lower(&*prog.ir)?;
    let peeled_goal = goal.into_peeled_goal();
    match args
        .solver_choice()
        .solve_root_goal(&prog.env, &peeled_goal)
    {
        Ok(Some(v)) => println!("{}\n", v),
        Ok(None) => println!("No possible solution.\n"),
        Err(NoSolution) => println!("Solver failed"),
    }
    Ok(())
}

impl Args {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::SLG {
            max_size: self.flag_overflow_depth,
        }
    }
}
