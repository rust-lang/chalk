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
        I::debug_ty(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.interned))
    }
}

impl<I: Interner> Debug for Lifetime<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_lifetime(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.interned))
    }
}

impl<I: Interner> Debug for Parameter<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_parameter(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.0))
    }
}

impl<I: Interner> Debug for Goal<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_goal(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.interned))
    }
}

impl<I: Interner> Debug for Goals<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_goals(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.goals))
    }
}

impl<I: Interner> Debug for ProgramClauseImplication<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_program_clause_implication(self, fmt)
            .unwrap_or_else(|| write!(fmt, "ProgramClauseImplication(?)"))
    }
}

impl<I: Interner> Debug for ProgramClause<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_program_clause(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.interned))
    }
}

impl<I: Interner> Debug for ProgramClauses<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_program_clauses(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.interned))
    }
}

impl<I: Interner> Debug for ApplicationTy<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_application_ty(self, fmt).unwrap_or_else(|| write!(fmt, "ApplicationTy(?)"))
    }
}

impl<I: Interner> Debug for SeparatorTraitRef<'_, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_separator_trait_ref(self, fmt)
            .unwrap_or_else(|| write!(fmt, "SeparatorTraitRef(?)"))
    }
}

impl<I: Interner> Debug for AliasTy<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_alias(self, fmt).unwrap_or_else(|| write!(fmt, "AliasTy(?)"))
    }
}

impl<I: Interner> Display for Substitution<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        I::debug_substitution(self, fmt).unwrap_or_else(|| write!(fmt, "{:?}", self.parameters))
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
            TyData::BoundVar(db) => write!(fmt, "{:?}", db),
            TyData::Dyn(clauses) => write!(fmt, "{:?}", clauses),
            TyData::InferenceVar(var) => write!(fmt, "{:?}", var),
            TyData::Apply(apply) => write!(fmt, "{:?}", apply),
            TyData::Alias(alias) => write!(fmt, "{:?}", alias),
            TyData::Placeholder(index) => write!(fmt, "{:?}", index),
            TyData::Function(function) => write!(fmt, "{:?}", function),
        }
    }
}

impl Debug for BoundVar {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let BoundVar { debruijn, index } = self;
        write!(fmt, "{:?}.{:?}", debruijn, index)
    }
}

impl Debug for DebruijnIndex {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let DebruijnIndex { depth } = self;
        write!(fmt, "^{}", depth)
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
            LifetimeData::BoundVar(db) => write!(fmt, "'{:?}", db),
            LifetimeData::InferenceVar(var) => write!(fmt, "'{:?}", var),
            LifetimeData::Placeholder(index) => write!(fmt, "'{:?}", index),
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

impl<I: Interner> Debug for GoalData<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            GoalData::Quantified(qkind, ref subgoal) => {
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
            GoalData::Implies(ref wc, ref g) => write!(fmt, "if ({:?}) {{ {:?} }}", wc, g),
            GoalData::All(ref goals) => write!(fmt, "all{:?}", goals),
            GoalData::Not(ref g) => write!(fmt, "not {{ {:?} }}", g),
            GoalData::EqGoal(ref wc) => write!(fmt, "{:?}", wc),
            GoalData::DomainGoal(ref wc) => write!(fmt, "{:?}", wc),
            GoalData::CannotProve(()) => write!(fmt, r"¯\_(ツ)_/¯"),
        }
    }
}

pub struct GoalsDebug<'a, I: Interner> {
    goals: &'a Goals<I>,
    interner: &'a I,
}

impl<'a, I: Interner> Debug for GoalsDebug<'a, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "(")?;
        for (goal, index) in self.goals.iter(self.interner).zip(0..) {
            if index > 0 {
                write!(fmt, ", ")?;
            }
            write!(fmt, "{:?}", goal)?;
        }
        write!(fmt, ")")?;
        Ok(())
    }
}

impl<I: Interner> Goals<I> {
    pub fn debug<'a>(&'a self, interner: &'a I) -> GoalsDebug<'a, I> {
        GoalsDebug {
            goals: self,
            interner,
        }
    }
}

pub struct ParameterDataInnerDebug<'a, I: Interner>(&'a ParameterData<I>);

impl<'a, I: Interner> Debug for ParameterDataInnerDebug<'a, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            ParameterKind::Ty(n) => write!(fmt, "{:?}", n),
            ParameterKind::Lifetime(n) => write!(fmt, "{:?}", n),
        }
    }
}

impl<I: Interner> ParameterData<I> {
    pub fn inner_debug(&self) -> ParameterDataInnerDebug<'_, I> {
        ParameterDataInnerDebug(self)
    }
}

pub struct ProgramClauseImplicationDebug<'a, I: Interner> {
    pci: &'a ProgramClauseImplication<I>,
    interner: &'a I,
}

impl<'a, I: Interner> Debug for ProgramClauseImplicationDebug<'a, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let ProgramClauseImplicationDebug { pci, interner } = self;
        write!(fmt, "{:?}", pci.consequence)?;

        let conditions = pci.conditions.as_slice(interner);

        let conds = conditions.len();
        if conds == 0 {
            return Ok(());
        }

        write!(fmt, " :- ")?;
        for cond in &conditions[..conds - 1] {
            write!(fmt, "{:?}, ", cond)?;
        }
        write!(fmt, "{:?}", conditions[conds - 1])
    }
}

impl<I: Interner> ProgramClauseImplication<I> {
    pub fn debug<'a>(&'a self, interner: &'a I) -> ProgramClauseImplicationDebug<'a, I> {
        ProgramClauseImplicationDebug {
            pci: self,
            interner,
        }
    }
}

pub struct ApplicationTyDebug<'a, I: Interner> {
    application_ty: &'a ApplicationTy<I>,
    interner: &'a I,
}

impl<'a, I: Interner> Debug for ApplicationTyDebug<'a, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let ApplicationTyDebug {
            application_ty,
            interner,
        } = self;
        let ApplicationTy { name, substitution } = application_ty;
        write!(fmt, "{:?}{:?}", name, substitution.with_angle(interner))
    }
}

impl<I: Interner> ApplicationTy<I> {
    pub fn debug<'a>(&'a self, interner: &'a I) -> ApplicationTyDebug<'a, I> {
        ApplicationTyDebug {
            application_ty: self,
            interner,
        }
    }
}

pub struct SubstitutionDebug<'a, I: Interner> {
    substitution: &'a Substitution<I>,
    interner: &'a I,
}

impl<'a, I: Interner> Debug for SubstitutionDebug<'a, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let SubstitutionDebug {
            substitution,
            interner,
        } = self;
        let mut first = true;

        write!(fmt, "[")?;

        for (index, value) in substitution.iter(interner).enumerate() {
            if first {
                first = false;
            } else {
                write!(fmt, ", ")?;
            }

            write!(fmt, "?{} := {:?}", index, value)?;
        }

        write!(fmt, "]")?;

        Ok(())
    }
}

impl<I: Interner> Substitution<I> {
    pub fn debug<'a>(&'a self, interner: &'a I) -> SubstitutionDebug<'a, I> {
        SubstitutionDebug {
            substitution: self,
            interner,
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

pub struct SeparatorTraitRefDebug<'a, 'me, I: Interner> {
    separator_trait_ref: &'a SeparatorTraitRef<'me, I>,
    interner: &'a I,
}

impl<'a, 'me, I: Interner> Debug for SeparatorTraitRefDebug<'a, 'me, I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        let SeparatorTraitRefDebug {
            separator_trait_ref,
            interner,
        } = self;
        let parameters = separator_trait_ref
            .trait_ref
            .substitution
            .parameters(interner);
        write!(
            fmt,
            "{:?}{}{:?}{:?}",
            parameters[0],
            separator_trait_ref.separator,
            separator_trait_ref.trait_ref.trait_id,
            Angle(&parameters[1..])
        )
    }
}

impl<'me, I: Interner> SeparatorTraitRef<'me, I> {
    pub fn debug<'a>(&'a self, interner: &'a I) -> SeparatorTraitRefDebug<'a, 'me, I> {
        SeparatorTraitRefDebug {
            separator_trait_ref: self,
            interner,
        }
    }
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

        // NB: We always print the `for<>`, even if it is empty,
        // because it may affect the debruijn indices of things
        // contained within. For example, `for<> { ^1.0 }` is very
        // different from `^1.0` in terms of what variable is being
        // referenced.

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
        Debug::fmt(value, fmt)
    }
}

impl<I: Interner> Debug for ProgramClauseData<I> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ProgramClauseData::Implies(pc) => write!(fmt, "{:?}", pc),
            ProgramClauseData::ForAll(pc) => write!(fmt, "{:?}", pc),
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
            // Ordinarily, we try to print all binder levels, if they
            // are empty, but we can skip in this *particular* case
            // because we know that `Canonical` terms are never
            // supposed to contain free variables.  In other words,
            // all "bound variables" that appear inside the canonical
            // value must reference values that appear in `binders`.
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
