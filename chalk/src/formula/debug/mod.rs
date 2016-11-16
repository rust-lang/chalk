use std::fmt::{Debug, Formatter, Error};

use super::clause::*;
use super::quant::*;
use super::goal::*;
use super::leaf::*;

mod env;

impl<L: Debug> Debug for Clause<L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.num_binders > 0 {
            write!(fmt, "forall{:?}", **self)
        } else {
            write!(fmt, "{:?}", self.skip_binders())
        }
    }
}

impl<L: Debug> Debug for ClauseImplication<L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let ClauseImplication { ref condition, ref consequence } = *self;
        if let &Some(ref goal) = condition {
            write!(fmt, "implies(")?;
            goal.fmt(fmt)?;
            write!(fmt, " => ")?;
            consequence.fmt(fmt)?;
            write!(fmt, ")")?;
        } else {
            consequence.fmt(fmt)?;
        }
        Ok(())
    }
}

impl<L: Debug> Debug for Goal<L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        self.kind.fmt(fmt)
    }
}

impl<L: Debug> Debug for GoalKind<L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            GoalKind::True => write!(fmt, "true")?,
            GoalKind::Leaf(ref l) => l.fmt(fmt)?,
            GoalKind::And(ref a, ref b) => write!(fmt, "and({:?}, {:?})", a, b)?,
            GoalKind::Or(ref a, ref b) => write!(fmt, "or({:?}; {:?})", a, b)?,
            GoalKind::Implication(ref clauses, ref l) => {
                write!(fmt, "implies(")?;
                for (index, clause) in clauses.iter().enumerate() {
                    if index > 0 {
                        write!(fmt, ", ")?;
                    }
                    clause.fmt(fmt)?;
                }
                write!(fmt, " => ")?;
                l.fmt(fmt)?;
                write!(fmt, ")")?;
            }
            GoalKind::ForAll(ref q) => {
                write!(fmt, "forall")?;
                q.fmt(fmt)?;
            }
            GoalKind::Exists(ref q) => {
                write!(fmt, "exists")?;
                q.fmt(fmt)?;
            }
        }
        Ok(())
    }
}

impl<F: Debug> Debug for Quantification<F> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "(")?;
        for i in 0..self.num_binders {
            if i > 0 {
                write!(fmt, ", ")?;
            }
            env::bind_name(fmt)?;
        }
        write!(fmt, " -> ")?;
        self.skip_binders().fmt(fmt)?;
        env::unbind_names(self.num_binders); // Nit: not exn-safe
        write!(fmt, ")")
    }
}

impl Debug for Leaf {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        self.kind.fmt(fmt)
    }
}

impl Debug for LeafKind {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            LeafKind::InferenceVariable(iv) => {
                write!(fmt, "{:?}", iv)?;
            }
            LeafKind::BoundVariable(bv) => {
                env::fmt_bound_variable(bv.depth, fmt)?;
            }
            LeafKind::Application(ref a) => {
                write!(fmt, "{:?}", a)?;
            }
        }
        Ok(())
    }
}

impl Debug for Application {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}", self.constant)?;
        if self.args.len() > 0 {
            fmt_parens(fmt, &self.args)?;
        }
        Ok(())
    }
}

impl Debug for Constant {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Constant::Program(s) => write!(fmt, "{:?}", s),
            Constant::Skolemized(ui) => write!(fmt, "?skol{}", ui.counter),
        }
    }
}

fn fmt_parens<D: Debug>(fmt: &mut Formatter, vs: &[D]) -> Result<(), Error> {
    write!(fmt, "(")?;
    for (i, v) in vs.iter().enumerate() {
        if i > 0 {
            write!(fmt, ", ")?;
        }
        write!(fmt, "{:?}", v)?;
    }
    write!(fmt, ")")
}
