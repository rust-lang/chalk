use std::fmt::{Debug, Display, Error, Formatter};

use super::*;

impl<I: Interner> Debug for TraitId<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_trait_id(*self, fmt).unwrap_or_else(|| write!(fmt, "TraitId({:?})", self.0))
    }
}

impl<I: Interner> Debug for StructId<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_struct_id(*self, fmt).unwrap_or_else(|| write!(fmt, "StructId({:?})", self.0))
    }
}

impl<I: Interner> Debug for AssocTypeId<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_assoc_type_id(*self, fmt)
            .unwrap_or_else(|| write!(fmt, "AssocTypeId({:?})", self.0))
    }
}

impl<I: Interner> Debug for Ty<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_ty(self, fmt)
            .unwrap_or_else(|| unimplemented!("cannot format Ty without setting Program in tls"))
    }
}

impl<I: Interner> Debug for Lifetime<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_lifetime(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format Lifetime without setting Program in tls")
        })
    }
}

impl<I: Interner> Debug for Parameter<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_parameter(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format Parameter without setting Program in tls")
        })
    }
}

impl<I: Interner> Debug for Goal<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_goal(self, fmt)
            .unwrap_or_else(|| unimplemented!("cannot format Goal without setting Program in tls"))
    }
}

impl<I: Interner> Debug for Goals<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_goals(self, fmt)
            .unwrap_or_else(|| unimplemented!("cannot format Goals without setting Program in tls"))
    }
}

impl<I: Interner> Debug for ProgramClauseImplication<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_program_clause_implication(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format ProgramClauseImplication without setting Program in tls")
        })
    }
}

impl<I: Interner> Debug for ApplicationTy<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_application_ty(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format ApplicationTy without setting Program in tls")
        })
    }
}

impl<I: Interner> Debug for SeparatorTraitRef<'_, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_separator_trait_ref(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format Substitution without setting Program in tls")
        })
    }
}

impl<I: Interner> Debug for AliasTy<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_alias(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format AliasTy without setting Program in tls")
        })
    }
}

impl<I: Interner> Display for Substitution<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_substitution(self, fmt).unwrap_or_else(|| {
            unimplemented!("cannot format Substitution without setting Program in tls")
        })
    }
}

impl Display for UniverseIndex {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "U{}", self.counter)
    }
}

impl Debug for UniverseIndex {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "U{}", self.counter)
    }
}

impl<I: Interner> Debug for TypeName<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TypeName::Struct(id) => write!(fmt, "{:?}", id),
            TypeName::AssociatedType(assoc_ty) => write!(fmt, "{:?}", assoc_ty),
            TypeName::Error => write!(fmt, "{{error}}"),
        }
    }
}

impl<I: Interner> Debug for TyData<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TyData::BoundVar(depth) => write!(fmt, "^{}", depth),
            TyData::Dyn(clauses) => write!(fmt, "{:?}", clauses),
            TyData::InferenceVar(var) => write!(fmt, "{:?}", var),
            TyData::Apply(apply) => write!(fmt, "{:?}", apply),
            TyData::Alias(alias) => write!(fmt, "{:?}", alias),
            TyData::Placeholder(index) => write!(fmt, "{:?}", index),
            TyData::Function(function) => write!(fmt, "{:?}", function),
        }
    }
}

impl<I: Interner> Debug for DynTy<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let DynTy { bounds } = self;
        write!(fmt, "dyn {:?}", bounds)
    }
}

impl Debug for InferenceVar {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "?{}", self.index)
    }
}

impl<I: Interner> Debug for Fn<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        // FIXME -- we should introduce some names or something here
        let Fn {
            num_binders,
            parameters,
        } = self;
        write!(fmt, "for<{}> {:?}", num_binders, parameters)
    }
}

impl<I: Interner> Debug for LifetimeData<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            LifetimeData::BoundVar(depth) => write!(fmt, "'^{}", depth),
            LifetimeData::InferenceVar(var) => write!(fmt, "'{:?}", var),
            LifetimeData::Placeholder(index) => write!(fmt, "'{:?}", index),
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

impl Debug for PlaceholderIndex {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let PlaceholderIndex { ui, idx } = self;
        write!(fmt, "!{}_{}", ui.counter, idx)
    }
}

impl<I: Interner> TraitRef<I> {
    /// Returns a "Debuggable" type that prints like `P0 as Trait<P1..>`
    pub fn with_as(&self) -> impl std::fmt::Debug + '_ {
        SeparatorTraitRef {
            trait_ref: self,
            separator: " as ",
        }
    }

    /// Returns a "Debuggable" type that prints like `P0: Trait<P1..>`
    pub fn with_colon(&self) -> impl std::fmt::Debug + '_ {
        SeparatorTraitRef {
            trait_ref: self,
            separator: ": ",
        }
    }
}

impl<I: Interner> Debug for TraitRef<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        Debug::fmt(&self.with_as(), fmt)
    }
}

pub struct SeparatorTraitRef<'me, I: Interner> {
    pub trait_ref: &'me TraitRef<I>,
    pub separator: &'me str,
}

pub struct Angle<'a, T>(pub &'a [T]);

impl<'a, T: Debug> Debug for Angle<'a, T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
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

impl<I: Interner> Debug for Normalize<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "Normalize({:?} -> {:?})", self.alias, self.ty)
    }
}

impl<I: Interner> Debug for AliasEq<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "AliasEq({:?} = {:?})", self.alias, self.ty)
    }
}

impl<I: Interner> Debug for WhereClause<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            WhereClause::Implemented(tr) => write!(fmt, "Implemented({:?})", tr.with_colon()),
            WhereClause::AliasEq(a) => write!(fmt, "{:?}", a),
        }
    }
}

impl<I: Interner> Debug for FromEnv<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            FromEnv::Trait(t) => write!(fmt, "FromEnv({:?})", t.with_colon()),
            FromEnv::Ty(t) => write!(fmt, "FromEnv({:?})", t),
        }
    }
}

impl<I: Interner> Debug for WellFormed<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            WellFormed::Trait(t) => write!(fmt, "WellFormed({:?})", t.with_colon()),
            WellFormed::Ty(t) => write!(fmt, "WellFormed({:?})", t),
        }
    }
}

impl<I: Interner> Debug for DomainGoal<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            DomainGoal::Holds(n) => write!(fmt, "{:?}", n),
            DomainGoal::WellFormed(n) => write!(fmt, "{:?}", n),
            DomainGoal::FromEnv(n) => write!(fmt, "{:?}", n),
            DomainGoal::Normalize(n) => write!(fmt, "{:?}", n),
            DomainGoal::IsLocal(n) => write!(fmt, "IsLocal({:?})", n),
            DomainGoal::IsUpstream(n) => write!(fmt, "IsUpstream({:?})", n),
            DomainGoal::IsFullyVisible(n) => write!(fmt, "IsFullyVisible({:?})", n),
            DomainGoal::LocalImplAllowed(tr) => {
                write!(fmt, "LocalImplAllowed({:?})", tr.with_colon(),)
            }
            DomainGoal::Compatible(_) => write!(fmt, "Compatible"),
            DomainGoal::DownstreamType(n) => write!(fmt, "DownstreamType({:?})", n),
        }
    }
}

impl<I: Interner> Debug for EqGoal<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "({:?} = {:?})", self.a, self.b)
    }
}

impl<T: Debug> Debug for Binders<T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
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

impl<I: Interner> Debug for ProgramClause<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ProgramClause::Implies(pc) => write!(fmt, "{:?}", pc),
            ProgramClause::ForAll(pc) => write!(fmt, "{:?}", pc),
        }
    }
}

impl<I: Interner> Debug for Environment<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "Env({:?})", self.clauses)
    }
}

impl<T: Display> Display for Canonical<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            ParameterKind::Ty(ref n) => write!(fmt, "Ty({:?})", n),
            ParameterKind::Lifetime(ref n) => write!(fmt, "Lifetime({:?})", n),
        }
    }
}

impl<I: Interner> Debug for Constraint<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Constraint::LifetimeEq(a, b) => write!(fmt, "{:?} == {:?}", a, b),
        }
    }
}

impl<I: Interner> Display for ConstrainedSubst<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ConstrainedSubst { subst, constraints } = self;

        write!(
            f,
            "substitution {}, lifetime constraints {:?}",
            subst, constraints,
        )
    }
}

impl<I: Interner> Substitution<I> {
    /// Displays the substitution in the form `< P0, .. Pn >`, or (if
    /// the substitution is empty) as an empty string.
    pub fn with_angle(&self, interner: &I) -> Angle<'_, Parameter<I>> {
        Angle(self.parameters(interner))
    }
}

impl<I: Interner> Debug for Substitution<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        Display::fmt(self, fmt)
    }
}
