use lalrpop_intern::InternedString;

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
    Goal(WhereClause),
}

pub struct StructDefn {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub where_clauses: Vec<WhereClause>,
}

pub struct TraitDefn {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_defns: Vec<AssocTyDefn>,
}

pub struct AssocTyDefn {
    pub name: Identifier,
}

pub struct Impl {
    pub parameters: Vec<Identifier>,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

pub struct AssocTyValue {
    pub name: Identifier,
    pub value: Ty,
}

pub enum Ty {
    Id {
        name: Identifier,
    },
    Apply {
        name: Identifier,
        args: Vec<Ty>
    },
    Projection {
        proj: ProjectionTy,
    },
}

pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
}

pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Ty>,
}

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    pub str: InternedString,
    pub span: Span,
}

pub enum WhereClause {
    Implemented { trait_ref: TraitRef },
    ProjectionEq { projection: ProjectionTy, ty: Ty },
}
