use std::{iter, mem::take};

use chalk_ir::{
    cast::Cast,
    fold::{shift::Shift, Fold, Folder, SuperFold},
    interner::Interner,
    AliasTy, Binders, BoundVar, DebruijnIndex, EqGoal, Fallible, Goal, GoalData, Goals,
    ProgramClause, ProgramClauseData, ProgramClauseImplication, QuantifierKind, Ty, TyKind,
    TyVariableKind, VariableKind, VariableKinds,
};

/// Converts a set of clauses to require only syntactic equality.
/// This is done by introducing new parameters and subgoals in cases
/// where semantic equality may diverge, for instance in unnormalized projections.
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
    /// Stores the kinds of new parameters introduced during folding.
    /// The new parameters will either be added to an enclosing `exists` binder (when lowering a goal)
    /// or to an enclosing `forall` binder (when lowering a program clause).
    new_params: Vec<VariableKind<I>>,
    /// For each new parameter `X`, a new goal is introduced to define it, e.g. `EqGoal(<T as Iterator>::Item, X)
    new_goals: Vec<Goal<I>>,

    /// Stores the current number of variables in the binder we are adding parameters to.
    /// Incremented for each new variable added.
    binders_len: usize,
}

impl<'i, I: Interner> Folder<'i, I> for SynEqFolder<'i, I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Fallible<Ty<I>> {
        let interner = self.interner;
        let bound_var = BoundVar::new(DebruijnIndex::INNERMOST, self.binders_len);

        let new_ty = TyKind::BoundVar(bound_var).intern(interner);
        match ty.kind(interner) {
            TyKind::Alias(AliasTy::Projection(_)) | TyKind::Function(_) => {
                self.new_params
                    .push(VariableKind::Ty(TyVariableKind::General));
                self.new_goals.push(
                    EqGoal {
                        a: new_ty.clone().cast(interner),
                        b: ty.clone().cast(interner),
                    }
                    .cast(interner),
                );
                self.binders_len += 1;
                Ok(new_ty)
            }
            _ => ty.super_fold_with(self, outer_binder),
        }
    }

    /// Convert a program clause to rem
    ///
    /// Consider this (nonsense) example:
    ///
    /// ```notrust
    /// forall<X> {
    ///     Implemented(<X as Iterator>::Item>: Debug) :-
    ///         Implemented(X: Debug)
    /// }
    /// ```
    ///
    /// we would lower this into:
    ///
    /// ```notrust
    /// forall<X, Y> {
    ///     Implemented(Y: Debug) :-
    ///         EqGoal(<X as Iterator>::Item>, Y),
    ///         Implemented(X: Debug)
    /// }
    /// ```
    fn fold_program_clause(
        &mut self,
        clause: &ProgramClause<I>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<ProgramClause<I>> {
        let interner = self.interner;
        assert!(self.new_params.is_empty());
        assert!(self.new_goals.is_empty());

        let ProgramClauseData(binders) = clause.data(interner);

        let implication = binders.skip_binders();
        let mut binders: Vec<_> = binders.binders.as_slice(interner).into();

        // Adjust the outer binder to account for the binder in the program clause
        let outer_binder = outer_binder.shifted_in();

        // Set binders_len to binders.len() since new parameters will be added into the existing forall<...> binder on the program clause.
        self.binders_len = binders.len();

        // First lower the "consequence" -- in our example that is
        //
        // ```
        // Implemented(<X as Iterator>::Item: Debug)
        // ```
        //
        // then save out the `new_params` and `new_goals` vectors to store any new variables created as a result.
        // In this case, that would be the `Y` parameter and `EqGoal(<X as Iterator>::Item, Y)` goals.
        //
        // Note that these new parameters will have indices that come after the existing parameters,
        // so any references to existing parameters like `X` in the "conditions" are still valid even if we insert new parameters.
        let consequence = implication.consequence.fold_with(self, outer_binder)?;

        let mut new_params = take(&mut self.new_params);
        let mut new_goals = take(&mut self.new_goals);

        // Now fold the conditions (in our example, Implemented(X: Debug).
        // The resulting list might be expanded to include new EqGoal goals.
        let mut conditions = implication.conditions.fold_with(self, outer_binder)?;

        new_params.extend(take(&mut self.new_params));
        new_goals.extend(take(&mut self.new_goals));

        let constraints = implication.constraints.fold_with(self, outer_binder)?;

        new_params.extend(take(&mut self.new_params));
        new_goals.extend(take(&mut self.new_goals));

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
        assert!(self.new_params.is_empty());
        assert!(self.new_goals.is_empty());

        let interner = self.interner;
        match goal.data(interner) {
            GoalData::DomainGoal(_) | GoalData::EqGoal(_) => (),
            _ => return goal.super_fold_with(self, outer_binder),
        };

        // Set binders_len to zero as in the exists<..> binder we will create, there are no existing variables.
        self.binders_len = 0;

        // shifted in because we introduce a new binder
        let outer_binder = outer_binder.shifted_in();
        let syn_goal = goal
            .shifted_in(interner)
            .super_fold_with(self, outer_binder)?;
        let new_params = take(&mut self.new_params);
        let new_goals = take(&mut self.new_goals);

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
