use std::fmt::{Debug, Display, Error, Formatter};

use super::*;

impl Debug for RawId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.debug_struct("RawId")
            .field("index", &self.index)
            .finish()
    }
}

impl Debug for TypeKindId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            TypeKindId::TypeId(id) => write!(fmt, "{:?}", id),
            TypeKindId::TraitId(id) => write!(fmt, "{:?}", id),
            TypeKindId::StructId(id) => write!(fmt, "{:?}", id),
        }
    }
}

impl Debug for TypeId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        tls::with_current_program(|p| match p {
            Some(prog) => prog.debug_type_kind_id(TypeKindId::TypeId(*self), fmt),
            None => write!(fmt, "TypeId({:?})", self.0.index),
        })
    }
}

impl Debug for TraitId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        tls::with_current_program(|p| match p {
            Some(prog) => prog.debug_type_kind_id(TypeKindId::TraitId(*self), fmt),
            None => write!(fmt, "TraitId({:?})", self.0.index),
        })
    }
}

impl Debug for StructId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        tls::with_current_program(|p| match p {
            Some(prog) => prog.debug_type_kind_id(TypeKindId::StructId(*self), fmt),
            None => write!(fmt, "StructId({:?})", self.0.index),
        })
    }
}

impl Display for UniverseIndex {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "U{}", self.counter)
    }
}

impl Debug for UniverseIndex {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "U{}", self.counter)
    }
}

impl Debug for TypeName {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            TypeName::TypeKindId(id) => write!(fmt, "{:?}", id),
            TypeName::Placeholder(index) => write!(fmt, "{:?}", index),
            TypeName::AssociatedType(assoc_ty) => write!(fmt, "{:?}", assoc_ty),
        }
    }
}

impl Debug for ItemId {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            ItemId::StructId(id @ StructId(_)) => write!(fmt, "{:?}", TypeKindId::StructId(*id)),
            ItemId::TraitId(id @ TraitId(_)) => write!(fmt, "{:?}", TypeKindId::TraitId(*id)),
            ItemId::TypeId(id @ TypeId(_)) => write!(fmt, "{:?}", TypeKindId::TypeId(*id)),
            ItemId::ImplId(id @ ImplId(_)) => write!(fmt, "{:?}", id),
            ItemId::ClauseId(ClauseId(id)) => write!(fmt, "{:?}", id),
        }
    }
}

impl Debug for Ty {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Ty::BoundVar(depth) => write!(fmt, "^{}", depth),
            Ty::Dyn(clauses) => write!(fmt, "{:?}", clauses),
            Ty::Opaque(clauses) => write!(fmt, "{:?}", clauses),
            Ty::InferenceVar(var) => write!(fmt, "{:?}", var),
            Ty::Apply(apply) => write!(fmt, "{:?}", apply),
            Ty::Projection(proj) => write!(fmt, "{:?}", proj),
            Ty::ForAll(quantified_ty) => write!(fmt, "{:?}", quantified_ty),
        }
    }
}

impl Debug for InferenceVar {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "?{}", self.index)
    }
}

impl Debug for QuantifiedTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // FIXME -- we should introduce some names or something here
        let QuantifiedTy { num_binders, ty } = self;
        write!(fmt, "for<{}> {:?}", num_binders, ty)
    }
}

impl Debug for Lifetime {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Lifetime::BoundVar(depth) => write!(fmt, "'^{}", depth),
            Lifetime::InferenceVar(var) => write!(fmt, "'{:?}", var),
            Lifetime::Placeholder(index) => write!(fmt, "'{:?}", index),
        }
    }
}

impl Debug for PlaceholderIndex {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let PlaceholderIndex { ui, idx } = self;
        write!(fmt, "!{}_{}", ui.counter, idx)
    }
}

impl Debug for ApplicationTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}{:?}", self.name, Angle(&self.parameters))
    }
}

impl Debug for TraitRef {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(
            fmt,
            "{:?} as {:?}{:?}",
            self.parameters[0],
            self.trait_id,
            Angle(&self.parameters[1..])
        )
    }
}

impl Debug for ProjectionTy {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        tls::with_current_program(|p| match p {
            Some(program) => program.debug_projection(self, fmt),
            None => write!(
                fmt,
                "({:?}){:?}",
                self.associated_ty_id,
                Angle(&self.parameters)
            ),
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

impl Debug for Normalize {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Normalize({:?} -> {:?})", self.projection, self.ty)
    }
}

impl Debug for ProjectionEq {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "ProjectionEq({:?} = {:?})", self.projection, self.ty)
    }
}

impl Debug for WhereClause {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            WhereClause::Implemented(tr) => write!(
                fmt,
                "Implemented({:?}: {:?}{:?})",
                tr.parameters[0],
                tr.trait_id,
                Angle(&tr.parameters[1..])
            ),
            WhereClause::ProjectionEq(p) => write!(fmt, "{:?}", p),
        }
    }
}

impl Debug for FromEnv {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            FromEnv::Trait(t) => write!(fmt, "FromEnv({:?})", t),
            FromEnv::Ty(t) => write!(fmt, "FromEnv({:?})", t),
        }
    }
}

impl Debug for WellFormed {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            WellFormed::Trait(t) => write!(fmt, "WellFormed({:?})", t),
            WellFormed::Ty(t) => write!(fmt, "WellFormed({:?})", t),
        }
    }
}

impl Debug for DomainGoal {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            DomainGoal::Holds(n) => write!(fmt, "{:?}", n),
            DomainGoal::WellFormed(n) => write!(fmt, "{:?}", n),
            DomainGoal::FromEnv(n) => write!(fmt, "{:?}", n),
            DomainGoal::Normalize(n) => write!(fmt, "{:?}", n),
            DomainGoal::InScope(n) => write!(fmt, "InScope({:?})", n),
            DomainGoal::IsLocal(n) => write!(fmt, "IsLocal({:?})", n),
            DomainGoal::IsUpstream(n) => write!(fmt, "IsUpstream({:?})", n),
            DomainGoal::IsFullyVisible(n) => write!(fmt, "IsFullyVisible({:?})", n),
            DomainGoal::LocalImplAllowed(tr) => write!(
                fmt,
                "LocalImplAllowed({:?}: {:?}{:?})",
                tr.parameters[0],
                tr.trait_id,
                Angle(&tr.parameters[1..])
            ),
            DomainGoal::Compatible(_) => write!(fmt, "Compatible"),
            DomainGoal::DownstreamType(n) => write!(fmt, "DownstreamType({:?})", n),
        }
    }
}

impl Debug for LeafGoal {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            LeafGoal::EqGoal(ref eq) => write!(fmt, "{:?}", eq),
            LeafGoal::DomainGoal(ref dom) => write!(fmt, "{:?}", dom),
        }
    }
}

impl Debug for EqGoal {
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
            Goal::Not(ref g) => write!(fmt, "not {{ {:?} }}", g),
            Goal::Leaf(ref wc) => write!(fmt, "{:?}", wc),
            Goal::CannotProve(()) => write!(fmt, r"¯\_(ツ)_/¯"),
        }
    }
}

impl<T: Debug> Debug for Binders<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let Binders {
            ref binders,
            ref value,
        } = *self;
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

impl Debug for ProgramClause {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            ProgramClause::Implies(pc) => write!(fmt, "{:?}", pc),
            ProgramClause::ForAll(pc) => write!(fmt, "{:?}", pc),
        }
    }
}

impl Debug for ProgramClauseImplication {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}", self.consequence)?;

        let conds = self.conditions.len();
        if conds == 0 {
            return Ok(());
        }

        write!(fmt, " :- ")?;
        for cond in &self.conditions[..conds - 1] {
            write!(fmt, "{:?}, ", cond)?;
        }
        write!(fmt, "{:?}", self.conditions[conds - 1])
    }
}

impl Debug for Environment {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Env({:?})", self.clauses)
    }
}

impl<T: Display> Display for Canonical<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Canonical { binders, value } = self;

        if binders.is_empty() {
            write!(f, "{}", value)?;
        } else {
            write!(f, "for<")?;

            for (i, pk) in binders.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "?{}", pk.into_inner())?;
            }

            write!(f, "> {{ {} }}", value)?;
        }

        Ok(())
    }
}

impl<T: Debug, L: Debug> Debug for ParameterKind<T, L> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            ParameterKind::Ty(ref n) => write!(fmt, "Ty({:?})", n),
            ParameterKind::Lifetime(ref n) => write!(fmt, "Lifetime({:?})", n),
        }
    }
}

impl Debug for Parameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match &self.0 {
            ParameterKind::Ty(n) => write!(fmt, "{:?}", n),
            ParameterKind::Lifetime(n) => write!(fmt, "{:?}", n),
        }
    }
}

impl Debug for Constraint {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Constraint::LifetimeEq(a, b) => write!(fmt, "{:?} == {:?}", a, b),
        }
    }
}

impl Display for ConstrainedSubst {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let ConstrainedSubst { subst, constraints } = self;

        write!(
            f,
            "substitution {}, lifetime constraints {:?}",
            subst, constraints,
        )
    }
}

impl Debug for Substitution {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Display::fmt(self, f)
    }
}

impl Display for Substitution {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut first = true;

        write!(f, "[")?;

        for (index, value) in self.parameters.iter().enumerate() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }

            write!(f, "?{} := {:?}", index, value)?;
        }

        write!(f, "]")?;

        Ok(())
    }
}
