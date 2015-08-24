use formula::*;
use formula::arena;
use std::fmt::{Debug, Formatter, Error};

impl Debug for Term {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        arena::read(|a| {
            match *a.data(*self) {
                TermData::Constant(s) => write!(fmt, "{}", s),
                TermData::FreeVariable(s) => write!(fmt, "ref({})", s),
                TermData::BoundVariable(index) => write!(fmt, "{:?}", index),
                TermData::Lambda(ref term) => write!(fmt, "(fn {:?})", term),
                TermData::Apply(ref term1, ref term2) => write!(fmt, "({:?} {:?})", term1, term2),
                TermData::Suspension(ref suspension) => write!(fmt, "{:?}", suspension),
            }
        })
    }
}

impl Debug for Suspension {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "<{:?}; {:?}: {:?}; {:?}>", self.term, self.bound, self.environment, self.lifts)
    }
}

impl Debug for Environment {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self.first_cell {
            None => write!(fmt, "nil"),
            Some(ref cell) => write!(fmt, "{:?}", cell),
        }
    }
}

impl Debug for Cell {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}::{:?}", self.data, self.next)
    }
}

impl Debug for CellData {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            CellData::Term(ref t) => write!(fmt, "{:?}", t),
            CellData::Dummy(n) => write!(fmt, "Dummy({:?})", n),
        }
    }
}

impl Debug for DebruijnIndex {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "#{:?}", self.0)
    }
}
