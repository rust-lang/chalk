use std::fmt;
use string_cache::DefaultAtom;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Span { lo, hi }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item {
    StructDefn(StructDefn),
    TraitDefn(TraitDefn),
    Impl(Impl),
    Clause(Clause),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub fields: Vec<Field>,
    pub flags: StructFlags,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructFlags {
    pub upstream: bool,
    pub fundamental: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TraitDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub assoc_ty_defns: Vec<AssocTyDefn>,
    pub flags: TraitFlags,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
    pub upstream: bool,
    pub fundamental: bool,
    pub non_enumerable: bool,
    pub coinductive: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssocTyDefn {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub bounds: Vec<QuantifiedInlineBound>,
    pub where_clauses: Vec<QuantifiedWhereClause>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParameterKind {
    Ty(Identifier),
    Lifetime(Identifier),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Parameter {
    Ty(Ty),
    Lifetime(Lifetime),
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// An inline bound, e.g. `: Foo<K>` in `impl<K, T: Foo<K>> SomeType<T>`.
pub enum InlineBound {
    TraitBound(TraitBound),
    ProjectionEqBound(ProjectionEqBound),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QuantifiedInlineBound {
    pub parameter_kinds: Vec<ParameterKind>,
    pub bound: InlineBound,
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Represents a trait bound on e.g. a type or type parameter.
/// Does not know anything about what it's binding.
pub struct TraitBound {
    pub trait_name: Identifier,
    pub args_no_self: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Represents a projection equality bound on e.g. a type or type parameter.
/// Does not know anything about what it's binding.
pub struct ProjectionEqBound {
    pub trait_bound: TraitBound,
    pub name: Identifier,
    pub args: Vec<Parameter>,
    pub value: Ty,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    Ty,
    Lifetime,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Kind::Ty => "type",
            Kind::Lifetime => "lifetime",
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Impl {
    pub parameter_kinds: Vec<ParameterKind>,
    pub trait_ref: TraitRef,
    pub polarity: Polarity,
    pub where_clauses: Vec<QuantifiedWhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
    pub impl_type: ImplType,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ImplType {
    Local,
    External,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssocTyValue {
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
    pub value: Ty,
    pub default: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
    Id {
        name: Identifier,
    },
    Dyn {
        bounds: Vec<QuantifiedInlineBound>,
    },
    Opaque {
        bounds: Vec<QuantifiedInlineBound>,
    },
    Apply {
        name: Identifier,
        args: Vec<Parameter>,
    },
    Projection {
        proj: ProjectionTy,
    },
    ForAll {
        lifetime_names: Vec<Identifier>,
        ty: Box<Ty>,
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Lifetime {
    Id { name: Identifier },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Polarity {
    /// `impl Foo for Bar`
    Positive,

    /// `impl !Foo for Bar`
    Negative,
}

impl Polarity {
    pub fn from_bool(polarity: bool) -> Polarity {
        if polarity {
            Polarity::Positive
        } else {
            Polarity::Negative
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Identifier {
    pub str: DefaultAtom,
    pub span: Span,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WhereClause {
    Implemented { trait_ref: TraitRef },
    ProjectionEq { projection: ProjectionTy, ty: Ty },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DomainGoal {
    Holds { where_clause: WhereClause },
    Normalize { projection: ProjectionTy, ty: Ty },
    TraitRefWellFormed { trait_ref: TraitRef },
    TyWellFormed { ty: Ty },
    TyFromEnv { ty: Ty },
    TraitRefFromEnv { trait_ref: TraitRef },
    IsLocal { ty: Ty },
    IsUpstream { ty: Ty },
    IsFullyVisible { ty: Ty },
    LocalImplAllowed { trait_ref: TraitRef },
    Compatible,
    DownstreamType { ty: Ty },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LeafGoal {
    DomainGoal { goal: DomainGoal },
    UnifyTys { a: Ty, b: Ty },
    UnifyLifetimes { a: Lifetime, b: Lifetime },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QuantifiedWhereClause {
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clause: WhereClause,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub name: Identifier,
    pub ty: Ty,
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// This allows users to add arbitrary `A :- B` clauses into the
/// logic; it has no equivalent in Rust, but it's useful for testing.
pub struct Clause {
    pub parameter_kinds: Vec<ParameterKind>,
    pub consequence: DomainGoal,
    pub conditions: Vec<Box<Goal>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Goal {
    ForAll(Vec<ParameterKind>, Box<Goal>),
    Exists(Vec<ParameterKind>, Box<Goal>),
    Implies(Vec<Clause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Not(Box<Goal>),

    /// The `compatible { G }` syntax
    Compatible(Box<Goal>),

    // Additional kinds of goals:
    Leaf(LeafGoal),
}
