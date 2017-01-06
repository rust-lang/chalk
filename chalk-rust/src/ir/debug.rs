use std::fmt::{Debug, Formatter, Error};

use super::*;

impl Debug for ItemId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        with_current_program(|prog| {
            match prog.and_then(|p| p.type_kinds.get(self)) {
                Some(k) => write!(fmt, "{}", k.name),
                None => fmt.debug_struct("ItemId").field("index", &self.index).finish(),
            }
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

impl Debug for AssociatedType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "({:?}::{})", self.trait_id, self.name)
    }
}

impl Debug for Ty {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Ty::Var(depth) => write!(fmt, "?{}", depth),
            Ty::Apply(ref apply) => write!(fmt, "{:?}", apply),
            Ty::Projection(ref proj) => write!(fmt, "{:?}", proj),
        }
    }
}

impl Debug for ApplicationTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}{:?}", self.name, Angle(&self.args))
    }
}

impl Debug for TraitRef {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?} as {:?}{:?}", self.args[0], self.trait_id, Angle(&self.args[1..]))
    }
}

impl Debug for ProjectionTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "<{:?}>::{}", self.trait_ref, self.name)
    }
}

struct Angle<'a, T: 'a>(&'a [T]);

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
