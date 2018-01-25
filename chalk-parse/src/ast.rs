use lalrpop_intern::InternedString;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Span { lo: lo, hi: hi }
    }
}

pub struct Program {
    pub items: Vec<Item>
}

pub enum Item {
    StructDefn(StructDefn),
    TraitDefn(TraitDefn),
    Impl(Impl),
    Clause(Clause),
}

pub struct StructDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<WhereClause>,
    pub fields: Vec<Field>,
}

pub struct TraitDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_defns: Vec<AssocTyDefn>,
    pub flags: TraitFlags,
}

pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
}

pub struct AssocTyDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
}

pub enum ParameterKind {
    Ty(Identifier),
    Lifetime(Identifier),
}

pub enum Parameter {
    Ty(Ty),
    Lifetime(Lifetime),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Ty,
    Lifetime,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            match *self {
                Kind::Ty => "type",
                Kind::Lifetime => "lifetime",
            }
        )
    }
}

pub trait Kinded {
    fn kind(&self) -> Kind;
}

impl Kinded for ParameterKind {
    fn kind(&self) -> Kind {
        match *self {
            ParameterKind::Ty(_) => Kind::Ty,
            ParameterKind::Lifetime(_) => Kind::Lifetime,
        }
    }
}

impl Kinded for Parameter {
    fn kind(&self) -> Kind {
        match *self {
            Parameter::Ty(_) => Kind::Ty,
            Parameter::Lifetime(_) => Kind::Lifetime,
        }
    }
}

pub struct Impl {
    pub parameter_kinds: Vec<ParameterKind>,
    pub trait_ref: PolarizedTraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

pub struct AssocTyValue {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<WhereClause>,
    pub value: Ty,
}

pub enum Ty {
    Id {
        name: Identifier,
    },
    Apply {
        name: Identifier,
        args: Vec<Parameter>
    },
    Projection {
        proj: ProjectionTy,
    },
    UnselectedProjection {
        proj: UnselectedProjectionTy,
    },
    ForAll {
        lifetime_names: Vec<Identifier>,
        ty: Box<Ty>
    }
}

pub enum Lifetime {
    Id {
        name: Identifier,
    }
}

pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

pub struct UnselectedProjectionTy {
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Parameter>,
}

pub enum PolarizedTraitRef {
    Positive(TraitRef),
    Negative(TraitRef),
}

impl PolarizedTraitRef {
    pub fn from_bool(polarity: bool, trait_ref: TraitRef) -> PolarizedTraitRef {
        if polarity {
            PolarizedTraitRef::Positive(trait_ref)
        } else {
            PolarizedTraitRef::Negative(trait_ref)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    pub str: InternedString,
    pub span: Span,
}

pub enum WhereClause {
    Implemented { trait_ref: TraitRef },
    Normalize { projection: ProjectionTy, ty: Ty },
    ProjectionEq { projection: ProjectionTy, ty: Ty },
    TyWellFormed { ty: Ty },
    TraitRefWellFormed { trait_ref: TraitRef },
    TyFromEnv { ty: Ty },
    TraitRefFromEnv { trait_ref: TraitRef },
    UnifyTys { a: Ty, b: Ty },
    UnifyLifetimes { a: Lifetime, b: Lifetime },
    TraitInScope { trait_name: Identifier },
}

pub struct Field {
    pub name: Identifier,
    pub ty: Ty,
}

/// This allows users to add arbitrary `A :- B` clauses into the
/// logic; it has no equivalent in Rust, but it's useful for testing.
pub struct Clause {
    pub parameter_kinds: Vec<ParameterKind>,
    pub consequence: WhereClause,
    pub conditions: Vec<Box<Goal>>,
}

pub enum Goal {
    ForAll(Vec<ParameterKind>, Box<Goal>),
    Exists(Vec<ParameterKind>, Box<Goal>),
    Implies(Vec<WhereClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Not(Box<Goal>),

    // Additional kinds of goals:
    Leaf(WhereClause),
}

