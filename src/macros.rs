use std::cell::RefCell;

lazy_static! {
    pub static ref DEBUG_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG").is_ok()
    };
}

thread_local! {
    pub static INDENT: RefCell<Vec<String>> = RefCell::new(vec![]);
}

// When CHALK_DEBUG is enabled, we only allow this many frames of
// nested processing, at which point we assume something has gone
// awry.
const OVERFLOW_DEPTH: usize = 100;

macro_rules! debug {
    ($($t:tt)*) => {
        if *::macros::DEBUG_ENABLED {
            ::macros::dump(&format!($($t)*));
        }
    }
}

macro_rules! debug_heading {
    ($($t:tt)*) => {
        let _ = &if *::macros::DEBUG_ENABLED {
            let string = format!($($t)*);
            ::macros::dump(&string);
            ::macros::Indent::new(string)
        } else {
            ::macros::Indent::new(String::new())
        };
    }
}

pub fn dump(string: &str) {
    if !*DEBUG_ENABLED {
        return;
    }

    let indent = ::macros::INDENT.with(|i| i.borrow().len());
    let mut first = true;
    for line in string.lines() {
        if first {
            for _ in 0..indent {
                eprint!("| ");
            }
        } else {
            for _ in 0..indent {
                eprint!("  ");
            }
        }
        eprintln!("{}", line);
        first = false;
    }
}

pub struct Indent {
    dummy: ()
}

impl Indent {
    pub fn new(value: String) -> Self {
        if *DEBUG_ENABLED {
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
        Indent { dummy: () }
    }
}

impl Drop for Indent {
    fn drop(&mut self) {
        if *DEBUG_ENABLED {
            INDENT.with(|i| i.borrow_mut().pop().unwrap());
        }
    }
}
