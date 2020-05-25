use crate::{ExClause, FlounderedSubgoal, Literal};
use chalk_base::results::Fallible;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::{HasInterner, Interner, TargetInterner};
use chalk_ir::visit::{Visit, VisitResult, Visitor};
use chalk_ir::{Canonical, ConstrainedSubst, Constraint, DebruijnIndex, InEnvironment};
use std::fmt::Debug;

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for ExClause<I> {
    type Result = ExClause<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let ExClause {
            subst,
            ambiguous,
            constraints,
            subgoals,
            delayed_subgoals,
            answer_time,
            floundered_subgoals,
        } = self;
        Ok(ExClause {
            subst: subst.fold_with(folder, outer_binder)?,
            ambiguous: *ambiguous,
            constraints: constraints.fold_with(folder, outer_binder)?,
            subgoals: subgoals.fold_with(folder, outer_binder)?,
            delayed_subgoals: delayed_subgoals.fold_with(folder, outer_binder)?,
            answer_time: answer_time.fold_with(folder, outer_binder)?,
            floundered_subgoals: floundered_subgoals.fold_with(folder, outer_binder)?,
        })
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for FlounderedSubgoal<I> {
    type Result = FlounderedSubgoal<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        let FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        } = self;
        Ok(FlounderedSubgoal {
            floundered_literal: floundered_literal.fold_with(folder, outer_binder)?,
            floundered_time: floundered_time.fold_with(folder, outer_binder)?,
        })
    }
}

impl<I: Interner, TI: TargetInterner<I>> Fold<I, TI> for Literal<I> {
    type Result = Literal<TI>;

    fn fold_with<'i>(
        &self,
        folder: &mut dyn Folder<'i, I, TI>,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Self::Result>
    where
        I: 'i,
        TI: 'i,
    {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, outer_binder)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, outer_binder)?)),
        }
    }
}

impl<I: Interner + Debug> Visit<I> for ExClause<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let ExClause {
            subst,
            ambiguous: _,
            constraints,
            subgoals,
            delayed_subgoals,
            answer_time,
            floundered_subgoals,
        } = self;

        R::new()
            .and_then(|| subst.visit_with(visitor, outer_binder))
            .and_then(|| constraints.visit_with(visitor, outer_binder))
            .and_then(|| constraints.visit_with(visitor, outer_binder))
            .and_then(|| subgoals.visit_with(visitor, outer_binder))
            .and_then(|| delayed_subgoals.visit_with(visitor, outer_binder))
            .and_then(|| answer_time.visit_with(visitor, outer_binder))
            .and_then(|| floundered_subgoals.visit_with(visitor, outer_binder))
    }
}

impl<I: Interner + Debug> Visit<I> for FlounderedSubgoal<I>
where
    InEnvironment<Constraint<I>>: Visit<I>,
    Canonical<ConstrainedSubst<I>>: Visit<I>,
{
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let FlounderedSubgoal {
            floundered_literal,
            floundered_time,
        } = self;

        R::new()
            .and_then(|| floundered_literal.visit_with(visitor, outer_binder))
            .and_then(|| floundered_time.visit_with(visitor, outer_binder))
    }
}

impl<I: Interner> Visit<I> for Literal<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        match self {
            Literal::Positive(goal) => goal.visit_with(visitor, outer_binder),
            Literal::Negative(goal) => goal.visit_with(visitor, outer_binder),
        }
    }
}

chalk_ir::copy_fold!(crate::TableIndex);
chalk_ir::copy_fold!(crate::TimeStamp);

chalk_ir::const_visit!(crate::TableIndex);
chalk_ir::const_visit!(crate::TimeStamp);

impl<I: Interner> HasInterner for ExClause<I> {
    type Interner = I;
}
