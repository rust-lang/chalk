use std::fmt::{Display, Formatter, Result};

use chalk_ir::interner::Interner;

use super::state::WriterState;

/// Displays `RenderAsRust` data.
///
/// This is a utility struct for making `RenderAsRust` nice to use with rust format macros.
pub struct DisplayRenderAsRust<'a, I: Interner, T> {
    s: &'a WriterState<'a, I>,
    rar: &'a T,
}

impl<I: Interner, T: RenderAsRust<I>> Display for DisplayRenderAsRust<'_, I, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.rar.fmt(self.s, f)
    }
}
pub trait RenderAsRust<I: Interner> {
    fn fmt(&self, s: &WriterState<'_, I>, f: &mut Formatter<'_>) -> Result;
    fn display<'a>(&'a self, s: &'a WriterState<'a, I>) -> DisplayRenderAsRust<'a, I, Self>
    where
        Self: Sized,
    {
        DisplayRenderAsRust { s, rar: self }
    }
}
