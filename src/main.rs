#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::Read;
use std::process::exit;

use chalk_integration::db::ChalkDatabase;
use chalk_integration::interner::ChalkIr;
use chalk_integration::lowering::*;
use chalk_integration::query::LoweringDatabase;
use chalk_integration::SolverChoice;
use chalk_solve::ext::*;
use chalk_solve::logging;
use chalk_solve::RustIrDatabase;
use docopt::Docopt;
use rustyline::error::ReadlineError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "
chalk repl

Usage:
  chalk [options]
  chalk (-h | --help)

Options:
  --help              Show this screen.
  --program=PATH      Specifies the path to the `.chalk` file containing traits/impls.
  --goal=GOAL         Specifies a goal to evaluate (may be given more than once).
  --overflow-depth=N  Specifies the overflow depth [default: 10].
  --multiple          Output multiple answers instead of ambiguous solution.
  --solver=S          Specifies the solver to use. `slg` or `recursive`. Default is SLG.
";

/// This struct represents the various command line options available.
#[derive(Debug, Deserialize)]
struct Args {
    flag_program: Option<String>,
    flag_solver: Option<String>,
    flag_goal: Vec<String>,
    flag_overflow_depth: usize,
    flag_multiple: bool,
}

/// A loaded and parsed program.
struct LoadedProgram {
    text: String,
    db: ChalkDatabase,
}

impl LoadedProgram {
    /// Creates a new Program struct, given a `.chalk` file as a String and
    /// a [`SolverChoice`].
    ///
    /// [`SolverChoice`]: struct.solve.SolverChoice.html
    fn new(text: String, solver_choice: SolverChoice) -> Result<LoadedProgram> {
        let db = ChalkDatabase::with(&text, solver_choice);
        Ok(LoadedProgram { text, db })
    }

    /// Parse a goal and attempt to solve it, using the specified solver.
    fn goal(
        &self,
        mut rl: Option<&mut rustyline::DefaultEditor>,
        text: &str,
        multiple_answers: bool,
    ) -> Result<()> {
        let program = self.db.checked_program()?;
        let goal = lower_goal(&*chalk_parse::parse_goal(text)?, &*program)?;
        let peeled_goal = goal.into_peeled_goal(self.db.interner());
        if multiple_answers {
            let no_more_solutions = self.db.solve_multiple(&peeled_goal, &mut |v, has_next| {
                println!("{}\n", v.as_ref().map(|v| v.display(ChalkIr)));
                if has_next {
                    if let Some(ref mut rl) = rl {
                        loop {
                            if let Ok(next) = rl.readline("Show next answer (y/n): ") {
                                if "y" == next {
                                    return true;
                                } else if "n" == next {
                                    return false;
                                } else {
                                    println!("Unknown response. Try again.");
                                }
                            } else {
                                return false;
                            }
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
            if no_more_solutions {
                println!("No more solutions");
            }
        } else {
            match self.db.solve(&peeled_goal) {
                Some(v) => println!("{}\n", v.display(ChalkIr)),
                None => println!("No possible solution.\n"),
            }
        }
        Ok(())
    }
}

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
        readline_loop(&mut rustyline::Editor::new()?, "?- ", |rl, line| {
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
        prog.db.with_program(|_| -> Result<()> {
            for g in &args.flag_goal {
                if let Err(e) = prog.goal(None, g, args.flag_multiple) {
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
fn readline_loop<F>(rl: &mut rustyline::DefaultEditor, prompt: &str, mut f: F) -> Result<()>
where
    F: FnMut(&mut rustyline::DefaultEditor, &str),
{
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                // Save the line to the history list.
                let _ = rl.add_history_entry(&line);

                // Process the line.
                f(rl, &line);
            }

            // EOF: We're done.
            Err(ReadlineError::Eof) => break,

            // Some other error occurred.
            Err(e) => return Err(e.into()),
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
    rl: &mut rustyline::DefaultEditor,
    prog: &mut Option<LoadedProgram>,
) -> Result<()> {
    if command.is_empty() {
        // Ignore empty commands.
    } else if command == "help" || command == "h" {
        // Print out interpreter commands.
        // TODO: Implement "help <command>" for more specific help.
        help()
    } else if command == "program" {
        // Load a .chalk file via stdin, until EOF is found.
        let chalk_prog = LoadedProgram::new(read_program(rl)?, args.solver_choice())?;
        // Let's do a sanity check before going forward.
        let _ = chalk_prog.db.checked_program()?;
        *prog = Some(chalk_prog);
    } else if let Some(filename) = command.strip_prefix("load ") {
        // Load a .chalk file.
        let chalk_prog = load_program(args, filename)?;
        // Let's do a sanity check before going forward.
        let _ = chalk_prog.db.checked_program()?;
        *prog = Some(chalk_prog);
    } else if let Some(level) = command.strip_prefix("debug ") {
        if level.is_empty() {
            println!("debug <level> set debug level to <level>");
        } else {
            std::env::set_var("CHALK_DEBUG", level);
        }
    } else {
        // The command is either "print", "lowered", or a goal.

        // Check that a program has been loaded.
        let prog = prog
            .as_ref()
            .ok_or("no program currently loaded; type 'help' to see available commands")?;

        // Attempt to parse the program.
        prog.db.with_program(|_| -> Result<()> {
            match command {
                // Print out the loaded program.
                "print" => println!("{}", prog.text),

                // TODO: Write a line of documentation here.
                "lowered" => println!("{:#?}", prog.db.environment()),

                // Assume this is a goal.
                // TODO: Print out "type 'help' to see available commands" if it
                // fails to parse?
                _ => prog.goal(Some(rl), command, args.flag_multiple)?,
            }
            Ok(())
        })?
    }

    Ok(())
}

/// Load the file into a string, and parse it.
// TODO: Could we pass in an Options struct or something? The Args struct
// still has Strings where it should have Enums... (e.g. solver_choice)
fn load_program(args: &Args, filename: &str) -> Result<LoadedProgram> {
    let mut text = String::new();
    File::open(filename)?.read_to_string(&mut text)?;
    LoadedProgram::new(text, args.solver_choice())
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
/// an error occurs while reading, a `Err` is returned.
fn read_program(rl: &mut rustyline::DefaultEditor) -> Result<String> {
    println!("Enter a program; press Ctrl-D when finished");
    let mut text = String::new();
    readline_loop(rl, "| ", |_, line| {
        text += line;
        text += "\n";
    })?;
    Ok(text)
}

impl Args {
    fn solver_choice(&self) -> SolverChoice {
        match self.flag_solver.as_deref() {
            None | Some("slg") => SolverChoice::SLG {
                max_size: self.flag_overflow_depth,
                expected_answers: None,
            },
            Some("recursive") => SolverChoice::Recursive {
                overflow_depth: 100,
                caching_enabled: true,
                max_size: 30,
            },
            Some(s) => panic!("invalid solver {}", s),
        }
    }
}

fn main() {
    use std::io::Write;
    logging::with_tracing_logs(|| {
        ::std::process::exit(match run() {
            Ok(_) => 0,
            Err(ref e) => {
                write!(&mut ::std::io::stderr(), "{}", e).expect("Error writing to stderr");
                1
            }
        })
    });
}
