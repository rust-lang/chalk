use std::fmt::{Debug, Formatter, Error};

use super::*;

impl Debug for ItemId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        with_current_program(|p| match p {
            Some(prog) => {
                if let Some(k) = prog.type_kinds.get(self) {
                    write!(fmt, "{}", k.name)
                } else if let Some(k) = prog.associated_ty_data.get(self) {
                    write!(fmt, "({:?}::{})", k.trait_id, k.name)
                } else {
                    fmt.debug_struct("ItemId").field("index", &self.index).finish()
                }
            }
            None => fmt.debug_struct("ItemId").field("index", &self.index).finish(),
        })
    }
}

impl Debug for UniverseIndex {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "U{}", self.counter)
    }
}

impl Debug for TypeName {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            TypeName::ItemId(id) => write!(fmt, "{:?}", id),
            TypeName::ForAll(universe) => write!(fmt, "!{}", universe.counter),
            TypeName::AssociatedType(assoc_ty) => write!(fmt, "{:?}", assoc_ty),
        }
    }
}

impl<T: Debug, L: Debug> Debug for ParameterKind<T, L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            ParameterKind::Ty(ref n) => write!(fmt, "{:?}", n),
            ParameterKind::Lifetime(ref n) => write!(fmt, "{:?}", n),
        }
    }
}

impl Debug for Ty {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Ty::Var(depth) => write!(fmt, "?{}", depth),
            Ty::Apply(ref apply) => write!(fmt, "{:?}", apply),
            Ty::Projection(ref proj) => write!(fmt, "{:?}", proj),
            Ty::ForAll(ref quantified_ty) => write!(fmt, "{:?}", quantified_ty),
        }
    }
}

impl Debug for QuantifiedTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // FIXME -- we should introduce some names or something here
        let QuantifiedTy { num_binders, ref ty } = *self;
        write!(fmt, "for<{}> {:?}", num_binders, ty)
    }
}

impl Debug for Lifetime {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Lifetime::Var(depth) => write!(fmt, "'?{}", depth),
            Lifetime::ForAll(universe) => write!(fmt, "'!{}", universe.counter),
        }
    }
}

impl Debug for ApplicationTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}{:?}", self.name, Angle(&self.parameters))
    }
}

impl Debug for TraitRef {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt,
               "{:?} as {:?}{:?}",
               self.parameters[0],
               self.trait_id,
               Angle(&self.parameters[1..]))
    }
}

impl Debug for ProjectionTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        with_current_program(|p| match p {
            Some(program) => {
                let (associated_ty_data, trait_params, other_params) =
                    program.split_projection(self);
                write!(fmt,
                       "<{:?} as {:?}{:?}>::{}{:?}",
                       &trait_params[0],
                       associated_ty_data.trait_id,
                       Angle(&trait_params[1..]),
                       associated_ty_data.name,
                       Angle(&other_params))
            }
            None => {
                write!(fmt,
                       "({:?}){:?}",
                       self.associated_ty_id,
                       Angle(&self.parameters))
            }
        })
    }
}

pub struct Angle<'a, T: 'a>(pub &'a [T]);

impl<'a, T: Debug> Debug for Angle<'a, T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.len() > 0 {
            write!(fmt, "<")?;
            for (index, elem) in self.0.iter().enumerate() {
                if index > 0 {
                    write!(fmt, ", {:?}", elem)?;
                } else {
                    write!(fmt, "{:?}", elem)?;
                }
            }
            write!(fmt, ">")?;
        }
        Ok(())
    }
}

struct Assignment<'a>(Identifier, &'a Ty);

impl<'a> Debug for Assignment<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{} = {:?}", self.0, self.1)
    }
}

impl Debug for Normalize {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?} ==> {:?}", self.projection, self.ty)
    }
}

impl Debug for WhereClause {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            WhereClause::Normalize(ref n) => write!(fmt, "{:?}", n),
            WhereClause::Implemented(ref n) => {
                write!(fmt,
                       "{:?}: {:?}{:?}",
                       n.parameters[0],
                       n.trait_id,
                       Angle(&n.parameters[1..]))
            }
        }
    }
}

impl Debug for WhereClauseGoal {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            WhereClauseGoal::Normalize(ref n) => write!(fmt, "{:?}", n),
            WhereClauseGoal::Implemented(ref n) => {
                write!(fmt,
                       "{:?}: {:?}{:?}",
                       n.parameters[0],
                       n.trait_id,
                       Angle(&n.parameters[1..]))
            }
            WhereClauseGoal::UnifyTys(ref n) => write!(fmt, "{:?}", n),
            WhereClauseGoal::WellFormed(ref n) => write!(fmt, "{:?}", n),
            WhereClauseGoal::LocalTo(ref n) => write!(fmt, "{:?}", n),
        }
    }
}

impl Debug for LocalTo {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let LocalTo { ref ty, ref crate_id } = *self;
        write!(fmt, "LocalTo({:?}, {:?})", ty, crate_id)
    }
}

impl Debug for WellFormed {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let value: &Debug = match *self {
            WellFormed::Ty(ref t) => t,
            WellFormed::TraitRef(ref t) => t,
        };
        write!(fmt, "WellFormed({:?})", value)
    }
}

impl<T: Debug> Debug for Unify<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "({:?} = {:?})", self.a, self.b)
    }
}

impl Debug for Goal {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Goal::Quantified(qkind, ref subgoal) => {
                write!(fmt, "{:?}<", qkind)?;
                for (index, binder) in subgoal.binders.iter().enumerate() {
                    if index > 0 {
                        write!(fmt, ", ")?;
                    }
                    match *binder {
                        ParameterKind::Ty(()) => write!(fmt, "type")?,
                        ParameterKind::Lifetime(()) => write!(fmt, "lifetime")?,
                    }
                }
                write!(fmt, "> {{ {:?} }}", subgoal.value)
            }
            Goal::Implies(ref wc, ref g) => write!(fmt, "if ({:?}) {{ {:?} }}", wc, g),
            Goal::And(ref g1, ref g2) => write!(fmt, "({:?}, {:?})", g1, g2),
            Goal::Leaf(ref wc) => write!(fmt, "{:?}", wc),
        }
    }
}

impl Debug for CrateId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", self.name)
    }
}

impl<T: Debug> Debug for Binders<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let Binders { ref binders, ref value } = *self;
        if !binders.is_empty() {
            write!(fmt, "for<")?;
            for (index, binder) in binders.iter().enumerate() {
                if index > 0 {
                    write!(fmt, ", ")?;
                }
                match *binder {
                    ParameterKind::Ty(()) => write!(fmt, "type")?,
                    ParameterKind::Lifetime(()) => write!(fmt, "lifetime")?,
                }
            }
            write!(fmt, "> ")?;
        }
        Debug::fmt(value, fmt)
    }
}
