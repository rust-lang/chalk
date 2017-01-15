use std::cell::Cell;

lazy_static! {
    pub static ref DEBUG_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG").is_ok()
    };
}

thread_local! {
    pub static INDENT: Cell<usize> = Cell::new(0);
}

macro_rules! debug {
    ($($t:tt)*) => {
        if *::macros::DEBUG_ENABLED {
            ::macros::dump(format!($($t)*));
        }
    }
}

macro_rules! debug_heading {
    ($($t:tt)*) => {
        debug!($($t)*);
        let _ = &::macros::Indent::new();
    }
}

pub fn dump(string: String) {
    if !*DEBUG_ENABLED {
        return;
    }

    let indent = ::macros::INDENT.with(|i| i.get());
    let mut first = true;
    for line in string.lines() {
        if first {
            for _ in 0..indent {
                print!("| ");
            }
        } else {
            for _ in 0..indent {
                print!("  ");
            }
        }
        println!("{}", line);
        first = false;
    }
}

pub struct Indent {
    dummy: ()
}

impl Indent {
    pub fn new() -> Self {
        if *DEBUG_ENABLED {
            INDENT.with(|i| i.set(i.get() + 1));
        }
        Indent { dummy: () }
    }
}

impl Drop for Indent {
    fn drop(&mut self) {
        if *DEBUG_ENABLED {
            INDENT.with(|i| i.set(i.get() - 1));
        }
    }
}
