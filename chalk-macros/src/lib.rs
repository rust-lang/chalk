use std::cell::RefCell;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod index;

lazy_static! {
    pub static ref DEBUG_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(|x| x >= 2)
            .unwrap_or(false)
    };
    pub static ref INFO_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(|x| x >= 1)
            .unwrap_or(false)
    };
}

thread_local! {
    pub(crate) static INDENT: RefCell<Vec<String>> = RefCell::new(vec![]);
}

// When CHALK_DEBUG is enabled, we only allow this many frames of
// nested processing, at which point we assume something has gone
// away.
const OVERFLOW_DEPTH: usize = 100;

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {
        if *$crate::DEBUG_ENABLED {
            $crate::dump(&format!($($t)*), "");
        }
    }
}

#[macro_export]
macro_rules! debug_heading {
    ($($t:tt)*) => {
        let _ = &if *$crate::DEBUG_ENABLED {
            let string = format!($($t)*);
            $crate::dump(&string, " {");
            $crate::Indent::new(true, string)
        } else {
            $crate::Indent::new(false, String::new())
        };
    }
}

#[macro_export]
macro_rules! info {
    ($($t:tt)*) => {
        if *$crate::INFO_ENABLED {
            $crate::dump(&format!($($t)*), "");
        }
    }
}

#[macro_export]
macro_rules! info_heading {
    ($($t:tt)*) => {
        let _ = &if *$crate::INFO_ENABLED {
            let string = format!($($t)*);
            $crate::dump(&string, " {");
            $crate::Indent::new(true, string)
        } else {
            $crate::Indent::new(false, String::new())
        };
    }
}

pub fn dump(string: &str, suffix: &str) {
    let indent = INDENT.with(|i| i.borrow().len());
    let mut first = true;
    for line in string.lines() {
        if first {
            for _ in 0..indent {
                eprint!(": ");
            }
            eprint!("| ");
        } else {
            eprintln!();
            for _ in 0..indent {
                eprint!(": ");
            }
            eprint!(": ");
        }
        eprint!("{}", line);
        first = false;
    }

    eprintln!("{}", suffix);
}

pub struct Indent {
    enabled: bool,
}

impl Indent {
    pub fn new(enabled: bool, value: String) -> Self {
        if enabled {
            INDENT.with(|i| {
                i.borrow_mut().push(value);
                if i.borrow().len() > OVERFLOW_DEPTH {
                    eprintln!("CHALK_DEBUG OVERFLOW:");
                    for v in i.borrow().iter().rev() {
                        eprintln!("- {}", v);
                    }
                    panic!("CHALK_DEBUG OVERFLOW")
                }
            });
        }
        Indent { enabled }
    }
}

impl Drop for Indent {
    fn drop(&mut self) {
        if self.enabled {
            INDENT.with(|i| i.borrow_mut().pop().unwrap());
            dump("}", "");
        }
    }
}
