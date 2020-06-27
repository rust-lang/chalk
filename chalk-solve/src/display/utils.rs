//! Render utilities which don't belong anywhere else.
use std::fmt::{Display, Formatter, Result};

pub fn as_display<F: Fn(&mut Formatter<'_>) -> Result>(f: F) -> impl Display {
    struct ClosureDisplay<F: Fn(&mut Formatter<'_>) -> Result>(F);

    impl<F: Fn(&mut Formatter<'_>) -> Result> Display for ClosureDisplay<F> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0(f)
        }
    }

    ClosureDisplay(f)
}

macro_rules! write_joined_non_empty_list {
    ($f:expr,$template:tt,$list:expr,$sep:expr) => {{
        let mut x = $list.into_iter().peekable();
        if x.peek().is_some() {
            write!($f, $template, x.format($sep))
        } else {
            Ok(())
        }
    }};
}
