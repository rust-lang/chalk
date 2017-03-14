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
    KrateDefn(KrateDefn),
}

pub struct StructDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<WhereClause>,
}

pub struct TraitDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_defns: Vec<AssocTyDefn>,
}

pub struct AssocTyDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
}

pub struct KrateDefn {
    pub name: Identifier,
    pub items: Vec<Item>
}

pub enum ParameterKind {
    Ty(Identifier),
    Lifetime(Identifier),
    Krate(Identifier),
}

pub enum Parameter {
    Ty(Ty),
    Lifetime(Lifetime),
    Krate(Krate),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Ty,
    Lifetime,
    Krate,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            match *self {
                Kind::Ty => "type",
                Kind::Lifetime => "lifetime",
                Kind::Krate => "crate",
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
            ParameterKind::Krate(_) => Kind::Krate,
        }
    }
}

impl Kinded for Parameter {
    fn kind(&self) -> Kind {
        match *self {
            Parameter::Ty(_) => Kind::Ty,
            Parameter::Lifetime(_) => Kind::Lifetime,
            Parameter::Krate(_) => Kind::Krate,
        }
    }
}

pub struct Impl {
    pub parameter_kinds: Vec<ParameterKind>,
    pub trait_ref: TraitRef,
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

pub enum Krate {
    Id {
        name: Identifier,
    }
}

pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Parameter>,
}

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    pub str: InternedString,
    pub span: Span,
}

pub enum WhereClause {
    Implemented { trait_ref: TraitRef },
    NotImplemented { trait_ref: TraitRef },
    ProjectionEq { projection: ProjectionTy, ty: Ty, eq: bool},
    TyWellFormed { ty: Ty },
    TraitRefWellFormed { trait_ref: TraitRef },
    LocalTo { ty: Ty, krate: Krate },
    UnifyTys { a: Ty, b: Ty, eq: bool },
    UnifyKrates { a: Krate, b: Krate },
    UnifyLifetimes { a: Lifetime, b: Lifetime },
}

pub enum Goal {
    ForAll(Vec<ParameterKind>, Box<Goal>),
    Exists(Vec<ParameterKind>, Box<Goal>),
    Implies(Vec<WhereClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Krate(Krate, Box<Goal>),

    // Additional kinds of goals:
    Leaf(WhereClause),
}
