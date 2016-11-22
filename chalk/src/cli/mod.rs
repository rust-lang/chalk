#![feature(conservative_impl_trait)]
#![feature(static_in_const)]

use docopt::Docopt;
use formula;
use std::error::Error;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

mod parse;

const USAGE: &'static str = "
Usage: chalk [options] <goal>
       chalk --help

Tries to parse and solve the given goal.

Options:
    -p <file>, --program <file> Specifies that we should load program clauses from `file`.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_program: Vec<String>,
    arg_goal: String,
}

pub fn main() {
    let mut stderr = io::stderr();

    let args: Args =
        Docopt::new(USAGE)
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
    for path in &args.flag_program {
        let text = file_text(path)?;
        let ast = parse::parse_program(path, &text)?;
        let clauses = formula::lower_program(path, &ast)?;
    }
    Ok(())
}

fn file_text(path: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

