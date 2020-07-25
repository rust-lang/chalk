//! `RenderAsRust` trait and related utils.
use std::fmt::{Display, Formatter, Result};

use chalk_ir::interner::Interner;

use super::state::InternalWriterState;

/// Displays `RenderAsRust` data.
///
/// This is a utility struct for making `RenderAsRust` nice to use with rust format macros.
pub(in crate::display) struct DisplayRenderAsRust<'a, I: Interner, T> {
    s: &'a InternalWriterState<'a, I>,
    rar: &'a T,
}

impl<I: Interner, T: RenderAsRust<I>> Display for DisplayRenderAsRust<'_, I, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.rar.fmt(self.s, f)
    }
}

pub(in crate::display) trait RenderAsRust<I: Interner> {
    fn fmt(&self, s: &InternalWriterState<'_, I>, f: &mut Formatter<'_>) -> Result;
    fn display<'a>(&'a self, s: &'a InternalWriterState<'a, I>) -> DisplayRenderAsRust<'a, I, Self>
    where
        Self: Sized,
    {
        DisplayRenderAsRust { s, rar: self }
    }
}
