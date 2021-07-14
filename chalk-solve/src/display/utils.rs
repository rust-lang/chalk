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

/// Processes a name given by an [`Interner`][chalk_ir::interner::Interner] debug
/// method into something usable by the `display` module.
///
/// This is specifically useful when implementing
/// [`RustIrDatabase`][crate::RustIrDatabase] `name_*` methods.
pub fn sanitize_debug_name(func: impl Fn(&mut Formatter<'_>) -> Option<Result>) -> String {
    use std::fmt::Write;

    // First, write the debug method contents to a String.
    let mut debug_out = String::new();
    // ignore if the result is `None`, as we can just as easily tell by looking
    // to see if anything was written to `debug_out`.
    write!(
        debug_out,
        "{}",
        as_display(|fmt| { func(fmt).unwrap_or(Ok(())) })
    )
    .expect("expected writing to a String to succeed");
    if debug_out.is_empty() {
        return "Unknown".to_owned();
    }

    // now the actual sanitization
    debug_out.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}
