//! Thread some naming context down between Debug impls to give names
//! to variables instead of just debruijn indices.

use std::cell::RefCell;
use std::fmt::{Error, Formatter};

thread_local!(static ENV: RefCell<Env> = RefCell::new(Env::new()));

pub fn bind_name(fmt: &mut Formatter) -> Result<(), Error> {
    ENV.with(|env| {
        let mut env = env.borrow_mut();
        write!(fmt, "{}", env.bind_name())
    })
}

pub fn unbind_names(count: usize) {
    ENV.with(|env| {
        let mut env = env.borrow_mut();
        env.unbind_names(count);
    })
}

pub fn fmt_bound_variable(depth: usize, fmt: &mut Formatter) -> Result<(), Error> {
    ENV.with(|env| {
        let env = env.borrow_mut();
        env.fmt_name(depth, fmt)
    })
}

struct Env {
    names: Vec<String>,
}

impl Env {
    fn new() -> Self {
        Env { names: vec![] }
    }

    fn bind_name(&mut self) -> &str {
        const NICE_NAMES: &[&str] = &["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
                                      "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
                                      "Y", "Z"];
        let name = if self.names.len() < NICE_NAMES.len() {
            format!("{}", NICE_NAMES[self.names.len()])
        } else {
            format!("_{}", self.names.len())
        };

        self.names.push(name);
        self.names.last().unwrap()
    }

    fn unbind_names(&mut self, count: usize) {
        for _ in 0 .. count {
            self.names.pop().unwrap();
        }
    }

    fn fmt_name(&self, depth: usize, fmt: &mut Formatter) -> Result<(), Error> {
        if depth < self.names.len() {
            let index = self.names.len() - depth - 1;
            write!(fmt, "{}", self.names[index])
        } else {
            write!(fmt, "FV({})", depth)
        }
    }
}

