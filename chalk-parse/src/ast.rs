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

/// The top-level container for a parsed .chalk file.
pub struct Program {
    pub items: Vec<Item>
}

/// The `Item` enum represents a statement in a .chalk file. 
pub enum Item {
    StructDefn(StructDefn),
    TraitDefn(TraitDefn),
    Impl(Impl),
    Clause(Clause),
}

/// Represents a rust struct.
pub struct StructDefn {
    /// The name of the structure.
    pub name: Identifier,

    /// The type and lifetime parameters for this struct. They are stored in the
    /// order they appear in the struct definition.
    pub parameter_kinds: Vec<ParameterKind>,

    /// Any bounds on the type parameters are stored here.
    pub where_clauses: Vec<QuantifiedWhereClause>,

    /// The field names and types for this struct.
    pub fields: Vec<Field>,

    /// Any modifiers on this struct.
    pub flags: StructFlags,
}

/// This structure contains any modifiers attached to a rust struct. Currently,
/// only the `extern` modifier is tracked.
pub struct StructFlags {
    /// This flag is true if the struct is `extern`.
    pub external: bool,
}

/// Represents a rust trait.
pub struct TraitDefn {
    /// The name of the trait.
    pub name: Identifier,

    /// The type and lifetime parameters for this trait. They are stored in the
    /// order they appear in the trait definition.
    pub parameter_kinds: Vec<ParameterKind>,

    /// Any bounds on the type parameters are stored here.
    pub where_clauses: Vec<QuantifiedWhereClause>,

    /// Associated types for this trait.
    pub assoc_ty_defns: Vec<AssocTyDefn>,

    /// Any modifiers on this trait.
    pub flags: TraitFlags,
}

/// This structure contains any flags attached to a rust trait.
pub struct TraitFlags {

    /// This is true if the trait is `#[auto]`.
    pub auto: bool,

    /// This is true if the trait is `#[marker]`.
    pub marker: bool,

    /// This is true if the trait is `extern`.
    pub external: bool,
}

/// Associated types are types defined inside of traits.
///
/// See also `AssocTyValue`.
// TODO: References.
pub struct AssocTyDefn {

    /// The name of the type.
    pub name: Identifier,

    /// The type and lifetime parameters for this associated type.
    pub parameter_kinds: Vec<ParameterKind>,
}

/// A generic type parameter.
/// 
/// Not to be confused with `Parameter`, which is a concrete type parameter.
/// For example, for the code:
/// ```
/// impl<T> Foo for Vec<T> {
///     // ...
/// }
/// ```
/// The `T` in `impl<T>` is a generic type parameter.
pub enum ParameterKind {
    Ty(Identifier),
    Lifetime(Identifier),
}

/// A concrete type parameter.
///
/// Not to be confused with `ParameterKind`, which is a generic type parameter.
/// For example, for the code:
/// ```
/// impl<T> Foo for Vec<T> {
///     // ...
/// }
/// ```
/// The `T` in `Vec<T>` is a concrete type parameter.
pub enum Parameter {
    Ty(Ty),
    Lifetime(Lifetime),
}

/// This enum is used as the return value of the `Kinded` trait.
// TODO: References.
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

/// This trait is an abstraction over the two different parameter enums,
/// `ParameterKind` and `Parameter`.
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

/// Represents a trait implementation for a type.
pub struct Impl {
    /// The type or lifetime parameters for this impl. These are used as
    /// parameters for the struct, the trait being impled, and the associated
    /// types.
    pub parameter_kinds: Vec<ParameterKind>,

    /// The trait this impl is implementing.
    pub trait_ref: PolarizedTraitRef,

    /// Any trait bounds on type parameters.
    pub where_clauses: Vec<QuantifiedWhereClause>,

    /// Associated type definitions, 
    pub assoc_ty_values: Vec<AssocTyValue>,
}

/// Represents an associated type that has been given a value inside a trait
/// impl.
pub struct AssocTyValue {
    /// The name of the associated type.
    pub name: Identifier,

    /// Any type or lifetime parameters for the type.
    pub parameter_kinds: Vec<ParameterKind>,

    /// Any bounds on the type parameters.
    pub where_clauses: Vec<WhereClause>,

    /// The new type assigned to the associated one.
    pub value: Ty,
}

/// This enum represents a type. 
pub enum Ty {
    /// A type with just a name.
    Id {
        name: Identifier,
    },

    /// A type with a name, and type or lifetime parameters.
    Apply {
        name: Identifier,
        args: Vec<Parameter>
    },

    /// A projection. See `ast::ProjectionTy`.
    Projection {
        proj: ProjectionTy,
    },

    /// A projection without an explicit trait name. See
    /// `ast::UnselectedProjectionTy`.
    UnselectedProjection {
        proj: UnselectedProjectionTy,
    },

    /// A type with one or more Higher-Kinded Type Bounds. Note that this variant
    /// is recursive.
    ForAll {
        lifetime_names: Vec<Identifier>,
        ty: Box<Ty>
    }
}

/// This struct represents a lifetime. There is only one kind of lifetime;
/// however, this enum leaves open the possibility of more kinds in the future.
pub enum Lifetime {
    Id {
        name: Identifier,
    }
}

/// This struct encodes a projection. A projection names an associated type of
/// another type. For example, `<Vec<T> as Foo>::Item`.
pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

/// This is the same as a projection, except without an explicit `as`. For
/// example, `Bar::Item`.
pub struct UnselectedProjectionTy {
    pub name: Identifier,
    pub args: Vec<Parameter>,
}

/// Represents a Trait applied to a type, separated by some string. The type that
/// is "recieving" the trait is the first element of the args array. For example,
/// for a trait bound (e.g. `Foo: Bar<T>`), the `TraitRef` would look like:
/// 
/// ```
/// TraitRef {
///     trait_name: "Bar",
///     args: ["Foo", "T"]
/// }
/// ```
/// (Using strings instead of actual structures for clarity.)
///
pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Parameter>,
}

/// This enum specifies whether a trait ref is included or excluded from a type.
/// 
/// For example, `Foo: Sized` is `Positive`, but `Foo: !Sized` is negative.
/// Trait bounds are `Positive` unless `!` is prepended to their name.
pub enum PolarizedTraitRef {
    Positive(TraitRef),
    Negative(TraitRef),
}

impl PolarizedTraitRef {
    /// Wraps a given `TraitRef`.
    pub fn from_bool(polarity: bool, trait_ref: TraitRef) -> PolarizedTraitRef {
        if polarity {
            PolarizedTraitRef::Positive(trait_ref)
        } else {
            PolarizedTraitRef::Negative(trait_ref)
        }
    }
}

/// A valid Rust identifier, along with a `Span` specifying line numbers.
#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    pub str: InternedString,
    pub span: Span,
}

/// A where clause; either a normal Rust trait bound, or a Chalk-specific clause.
// TODO: Add a "See" that points to a list of Chalk's where clauses, and a
// description of each.
pub enum WhereClause {
    /// A normal rust Trait bound.
    /// Example: `Foo<T>: Bar`
    Implemented { trait_ref: TraitRef },

    // TODO: Renaming a projection, I think...?
    Normalize { projection: ProjectionTy, ty: Ty },

    /// Assert that a projection is equal to a specific type.
    /// Example: `<Foo<T> as Bar>::Item == T`
    ProjectionEq { projection: ProjectionTy, ty: Ty },

    // TODO: What does "well formed" mean?
    TyWellFormed { ty: Ty },

    // TODO: What does "well formed" mean?
    TraitRefWellFormed { trait_ref: TraitRef },

    // TODO: What does "from env" mean?
    TyFromEnv { ty: Ty },

    // TODO: What does "from env" mean?
    TraitRefFromEnv { trait_ref: TraitRef },

    /// Constrain the two types to be equal.
    UnifyTys { a: Ty, b: Ty },

    /// Constrain the two lifetimes to be equal.
    UnifyLifetimes { a: Lifetime, b: Lifetime },

    /// Assert that a trait is in scope.
    TraitInScope { trait_name: Identifier },
}

pub struct QuantifiedWhereClause {
    pub parameter_kinds: Vec<ParameterKind>,
    pub where_clause: WhereClause,
}

/// A field in a struct.
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

/// A Chalk goal.
// TODO: Add a "See" that points to a list of Chalk's where clauses, and a
// description of each.
pub enum Goal {
    ForAll(Vec<ParameterKind>, Box<Goal>),
    Exists(Vec<ParameterKind>, Box<Goal>),
    Implies(Vec<Clause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Not(Box<Goal>),

    // Additional kinds of goals:
    Leaf(WhereClause),
}

