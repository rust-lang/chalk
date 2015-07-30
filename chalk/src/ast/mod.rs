pub struct Span(usize, usize);

pub struct ProgramDefn {
    
}

/// Declares the type of some terms.
pub struct Type {
    pub terms: 
}

/// D-formula, in the terminology of "PHOL".
pub struct Clause {
    pub span: Span,
    pub kind: ClauseKind,
}

pub enum ClauseKind {
    Atomic(Atomic),
    Implication(Box<Goal>, Box<Clause>),
    And(Box<Clause>, Box<Clause>),
    ForAll(Box<Clause>),
}

/// G-formula
pub struct Goal {
    pub span: Span,
    pub kind: GoalKind,
}

pub enum GoalKind {
    True,
    Atomic(Atomic),
    And(Box<Goal>, Box<Goal>),
    Or(Box<Goal>, Box<Goal>),
    Exists(Box<Goal>),
    Implication(Box<Clause>, Box<Goal>),
    ForAll(Box<Goal>),
}

pub struct Atomic {
    
}
