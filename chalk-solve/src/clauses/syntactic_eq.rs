use std::{iter, mem::replace};

use chalk_ir::{
    cast::Cast,
    fold::{shift::Shift, Fold, Folder, SuperFold},
    interner::Interner,
    AliasEq, AliasTy, Binders, BoundVar, DebruijnIndex, Fallible, Goal, GoalData, Goals,
    ProgramClause, ProgramClauseData, ProgramClauseImplication, QuantifierKind, Ty, TyData, TyKind,
    VariableKind, VariableKinds,
};

pub fn syn_eq_lower<I: Interner, T: Fold<I>>(interner: &I, clause: &T) -> <T as Fold<I>>::Result {
    let mut folder = SynEqFolder {
        interner,
        new_params: vec![],
        new_goals: vec![],
        binders_len: 0,
    };

    clause
        .fold_with(&mut folder, DebruijnIndex::INNERMOST)
        .unwrap()
}

struct SynEqFolder<'i, I: Interner> {
    interner: &'i I,
    new_params: Vec<VariableKind<I>>,
    new_goals: Vec<Goal<I>>,
    binders_len: usize,
}

impl<'i, I: Interner> Folder<'i, I> for SynEqFolder<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Fallible<Ty<I>> {
        let interner = self.interner;
        let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.binders_len);

        let new_ty = TyData::BoundVar(bound_var).intern(interner);
        match ty.data(interner) {
            TyData::Alias(alias @ AliasTy::Projection(_)) => {
                self.new_params.push(VariableKind::Ty(TyKind::General));
                self.new_goals.push(
                    AliasEq {
                        alias: alias.clone(),
                        ty: new_ty.clone(),
                    }
                    .cast(interner),
                );
                self.binders_len += 1;
                ty.super_fold_with(self, outer_binder)?;
                Ok(new_ty)
            }
            TyData::Function(_) => Ok(ty.clone()),
            _ => Ok(ty.super_fold_with(self, outer_binder)?),
        }
    }

    fn fold_program_clause(
        &mut self,
        clause: &ProgramClause<I>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<ProgramClause<I>> {
        let interner = self.interner;

        let ProgramClauseData(binders) = clause.data(interner);

        let implication = binders.skip_binders();
        let mut binders: Vec<_> = binders.binders.as_slice(interner).into();

        let outer_binder = outer_binder.shifted_in();

        self.binders_len = binders.len();
        let consequence = implication.consequence.fold_with(self, outer_binder)?;
        // Immediately move `new_params` out of of the folder struct so it's safe
        // to call `.fold_with` again
        let new_params = replace(&mut self.new_params, vec![]);
        let new_goals = replace(&mut self.new_goals, vec![]);

        let mut conditions = implication.conditions.fold_with(self, outer_binder)?;
        let constraints = implication.constraints.fold_with(self, outer_binder)?;
        if new_params.is_empty() {
            // shift the clause out since we didn't use the dummy binder
            return Ok(ProgramClauseData(Binders::empty(
                interner,
                ProgramClauseImplication {
                    consequence,
                    conditions,
                    constraints,
                    priority: implication.priority,
                }
                .shifted_out(interner)?,
            ))
            .intern(interner));
        }

        binders.extend(new_params.into_iter());

        conditions = Goals::from_iter(
            interner,
            conditions.iter(interner).cloned().chain(new_goals),
        );

        Ok(ProgramClauseData(Binders::new(
            VariableKinds::from_iter(interner, binders),
            ProgramClauseImplication {
                consequence,
                conditions,
                constraints,
                priority: implication.priority,
            },
        ))
        .intern(interner))
    }

    fn fold_goal(&mut self, goal: &Goal<I>, outer_binder: DebruijnIndex) -> Fallible<Goal<I>> {
        assert!(self.new_params.is_empty(), true);

        let interner = self.interner;
        match goal.data(interner) {
            GoalData::DomainGoal(_) | GoalData::EqGoal(_) => (),
            _ => return goal.super_fold_with(self, outer_binder),
        };

        self.binders_len = 0;
        // shifted in because we introduce a new binder
        let outer_binder = outer_binder.shifted_in();
        let syn_goal = goal
            .shifted_in(interner)
            .super_fold_with(self, outer_binder)?;
        let new_params = replace(&mut self.new_params, vec![]);
        let new_goals = replace(&mut self.new_goals, vec![]);

        if new_params.is_empty() {
            return Ok(goal.clone());
        }

        let goal = GoalData::All(Goals::from_iter(
            interner,
            iter::once(syn_goal).into_iter().chain(new_goals),
        ))
        .intern(interner);

        Ok(GoalData::Quantified(
            QuantifierKind::Exists,
            Binders::new(VariableKinds::from_iter(interner, new_params), goal),
        )
        .intern(interner))
    }

    fn interner(&self) -> &'i I {
        self.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner
    }
}
