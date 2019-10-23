use crate::program::Program as LoweredProgram;
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::family::ChalkIr;
use chalk_ir::{self, ImplId, StructId, TraitId, TypeId, TypeKindId};
use chalk_parse::ast::*;
use chalk_rust_ir as rust_ir;
use chalk_rust_ir::IntoWhereClauses;
use chalk_rust_ir::{Anonymize, ToParameter};
use failure::{Fail, Fallible};
use itertools::Itertools;
use lalrpop_intern::intern;
use std::collections::BTreeMap;
use std::sync::Arc;

mod test;

type TypeIds = BTreeMap<chalk_ir::Identifier, chalk_ir::TypeKindId>;
type TypeKinds = BTreeMap<chalk_ir::TypeKindId, rust_ir::TypeKind>;
type AssociatedTyLookups = BTreeMap<(chalk_ir::TraitId, chalk_ir::Identifier), AssociatedTyLookup>;
type ParameterMap = BTreeMap<chalk_ir::ParameterKind<chalk_ir::Identifier>, usize>;

#[derive(Fail, Debug)]
pub enum RustIrError {
    #[fail(display = "invalid type name `{}`", _0)]
    InvalidTypeName(Identifier),
    #[fail(display = "duplicate lang item `{:?}`", _0)]
    DuplicateLangItem(rust_ir::LangItem),
    #[fail(display = "expected a trait, found `{}`, which is not a trait", _0)]
    NotTrait(Identifier),
    #[fail(
        display = "`{}` takes {} type parameters, not {}",
        identifier, expected, actual
    )]
    IncorrectNumberOfTypeParameters {
        identifier: Identifier,
        expected: usize,
        actual: usize,
    },
    #[fail(display = "cannot apply type parameter `{}`", _0)]
    CannotApplyTypeParameter(Identifier),
}

#[derive(Clone, Debug)]
struct Env<'k> {
    type_ids: &'k TypeIds,
    type_kinds: &'k TypeKinds,
    associated_ty_lookups: &'k AssociatedTyLookups,
    /// Parameter identifiers are used as keys, therefore
    /// all identifiers in an environment must be unique (no shadowing).
    parameter_map: ParameterMap,
}

/// Information about an associated type **declaration** (i.e., an
/// `AssociatedTyDatum`). This information is gathered in the first
/// phase of creating the Rust IR and is then later used to lookup the
/// "id" of an associated type.
///
/// ```ignore
/// trait Foo {
///     type Bar<'a>; // <-- associated type declaration
///          // ----
///          // |
///          // addl_parameter_kinds
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
struct AssociatedTyLookup {
    id: chalk_ir::TypeId,
    addl_parameter_kinds: Vec<chalk_ir::ParameterKind<()>>,
}

enum NameLookup {
    Type(chalk_ir::TypeKindId),
    Parameter(usize),
}

enum LifetimeLookup {
    Parameter(usize),
}

const SELF: &str = "Self";
const FIXME_SELF: &str = "__FIXME_SELF__";

impl<'k> Env<'k> {
    fn lookup(&self, name: Identifier) -> Fallible<NameLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Ty(name.str))
        {
            return Ok(NameLookup::Parameter(*k));
        }

        if let Some(id) = self.type_ids.get(&name.str) {
            return Ok(NameLookup::Type(*id));
        }

        Err(RustIrError::InvalidTypeName(name))?
    }

    fn lookup_lifetime(&self, name: Identifier) -> Fallible<LifetimeLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Lifetime(name.str))
        {
            return Ok(LifetimeLookup::Parameter(*k));
        }

        Err(format_err!("invalid lifetime name: {:?}", name.str))
    }

    fn type_kind(&self, id: chalk_ir::TypeKindId) -> &rust_ir::TypeKind {
        &self.type_kinds[&id]
    }

    /// Introduces new parameters, shifting the indices of existing
    /// parameters to accommodate them. The indices of the new binders
    /// will be assigned in order as they are iterated.
    fn introduce<I>(&self, binders: I) -> Fallible<Self>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<chalk_ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
    {
        let binders = binders.into_iter().enumerate().map(|(i, k)| (k, i));
        let len = binders.len();
        let parameter_map: ParameterMap = self
            .parameter_map
            .iter()
            .map(|(&k, &v)| (k, v + len))
            .chain(binders)
            .collect();
        if parameter_map.len() != self.parameter_map.len() + len {
            return Err(format_err!("duplicate or shadowed parameters"));
        }
        Ok(Env {
            parameter_map,
            ..*self
        })
    }

    fn in_binders<I, T, OP>(&self, binders: I, op: OP) -> Fallible<chalk_ir::Binders<T>>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<chalk_ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
        OP: FnOnce(&Self) -> Fallible<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned())?;
        Ok(chalk_ir::Binders {
            binders: binders.anonymize(),
            value: op(&env)?,
        })
    }
}

pub(crate) trait LowerProgram {
    /// Lowers from a Program AST to the internal IR for a program.
    fn lower(&self) -> Fallible<LoweredProgram>;
}

impl LowerProgram for Program {
    fn lower(&self) -> Fallible<LoweredProgram> {
        let mut index = 0;
        let mut next_item_id = || -> chalk_ir::RawId {
            let i = index;
            index += 1;
            chalk_ir::RawId { index: i }
        };

        // Make a vector mapping each thing in `items` to an id,
        // based just on its position:
        let raw_ids: Vec<_> = self.items.iter().map(|_| next_item_id()).collect();

        // Create ids for associated types
        let mut associated_ty_lookups = BTreeMap::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            if let Item::TraitDefn(ref d) = *item {
                if d.flags.auto && !d.assoc_ty_defns.is_empty() {
                    return Err(format_err!("auto trait cannot define associated types"));
                }
                for defn in &d.assoc_ty_defns {
                    let addl_parameter_kinds = defn.all_parameters();
                    let lookup = AssociatedTyLookup {
                        id: TypeId(next_item_id()),
                        addl_parameter_kinds: addl_parameter_kinds.anonymize(),
                    };
                    associated_ty_lookups.insert((TraitId(raw_id), defn.name.str), lookup);
                }
            }
        }

        let mut type_ids = BTreeMap::new();
        let mut type_kinds = BTreeMap::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            let (k, id) = match *item {
                Item::StructDefn(ref d) => (d.lower_type_kind()?, StructId(raw_id).into()),
                Item::TraitDefn(ref d) => (d.lower_type_kind()?, TraitId(raw_id).into()),
                Item::Impl(_) => continue,
                Item::Clause(_) => continue,
            };
            type_ids.insert(k.name, id);
            type_kinds.insert(id, k);
        }

        let mut struct_data = BTreeMap::new();
        let mut trait_data = BTreeMap::new();
        let mut impl_data = BTreeMap::new();
        let mut associated_ty_data = BTreeMap::new();
        let mut custom_clauses = Vec::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            let empty_env = Env {
                type_ids: &type_ids,
                type_kinds: &type_kinds,
                associated_ty_lookups: &associated_ty_lookups,
                parameter_map: BTreeMap::new(),
            };

            match *item {
                Item::StructDefn(ref d) => {
                    let struct_id = StructId(raw_id);
                    struct_data.insert(struct_id, Arc::new(d.lower_struct(struct_id, &empty_env)?));
                }
                Item::TraitDefn(ref d) => {
                    let trait_id = TraitId(raw_id);
                    trait_data.insert(trait_id, Arc::new(d.lower_trait(trait_id, &empty_env)?));

                    for defn in &d.assoc_ty_defns {
                        let lookup = &associated_ty_lookups[&(trait_id, defn.name.str)];

                        let mut parameter_kinds = defn.all_parameters();
                        parameter_kinds.extend(d.all_parameters());

                        let binders = empty_env.in_binders(parameter_kinds, |env| {
                            Ok(rust_ir::AssociatedTyDatumBound {
                                bounds: defn.bounds.lower(&env)?,
                                where_clauses: defn.where_clauses.lower(&env)?,
                            })
                        })?;

                        associated_ty_data.insert(
                            lookup.id,
                            Arc::new(rust_ir::AssociatedTyDatum {
                                trait_id: TraitId(raw_id),
                                id: lookup.id,
                                name: defn.name.str,
                                binders: binders,
                            }),
                        );
                    }
                }
                Item::Impl(ref d) => {
                    let impl_id = ImplId(raw_id);
                    impl_data.insert(impl_id, Arc::new(d.lower_impl(&empty_env, impl_id)?));
                }
                Item::Clause(ref clause) => {
                    custom_clauses.extend(clause.lower_clause(&empty_env)?);
                }
            }
        }

        let program = LoweredProgram {
            type_ids,
            type_kinds,
            struct_data,
            trait_data,
            impl_data,
            associated_ty_data,
            custom_clauses,
        };

        Ok(program)
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> Fallible<rust_ir::TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>>;
    fn declared_parameters(&self) -> &[ParameterKind];
    fn all_parameters(&self) -> Vec<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        self.synthetic_parameters()
            .into_iter()
            .chain(self.declared_parameters().iter().map(|id| id.lower()))
            .collect()

        /* TODO: switch to this ordering, but adjust *all* the code to match

        self.declared_parameters()
            .iter()
            .map(|id| id.lower())
            .chain(self.synthetic_parameters()) // (*) see below
            .collect()
         */
    }

    fn parameter_refs(&self) -> Vec<chalk_ir::Parameter<ChalkIr>> {
        self.all_parameters()
            .anonymize()
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter())
            .collect()
    }

    fn parameter_map(&self) -> ParameterMap {
        // (*) It is important that the declared parameters come
        // before the subtle parameters in the ordering. This is
        // because of traits, when used as types, only have the first
        // N parameters in their kind (that is, they do not have Self).
        //
        // Note that if `Self` appears in the where-clauses etc, the
        // trait is not object-safe, and hence not supposed to be used
        // as an object. Actually the handling of object types is
        // probably just kind of messed up right now. That's ok.
        self.all_parameters().into_iter().zip(0..).collect()
    }
}

impl LowerParameterMap for StructDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Impl {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyValue {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for TraitDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        Some(chalk_ir::ParameterKind::Ty(intern(SELF)))
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Clause {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

trait LowerParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<chalk_ir::Identifier>;
}

impl LowerParameterKind for ParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<chalk_ir::Identifier> {
        match *self {
            ParameterKind::Ty(ref n) => chalk_ir::ParameterKind::Ty(n.str),
            ParameterKind::Lifetime(ref n) => chalk_ir::ParameterKind::Lifetime(n.str),
        }
    }
}

trait LowerWhereClauses {
    fn where_clauses(&self) -> &[QuantifiedWhereClause];

    fn lower_where_clauses(
        &self,
        env: &Env,
    ) -> Fallible<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        self.where_clauses().lower(env)
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self) -> Fallible<rust_ir::TypeKind> {
        Ok(rust_ir::TypeKind {
            sort: rust_ir::TypeSort::Struct,
            name: self.name.str,
            binders: chalk_ir::Binders {
                binders: self.all_parameters().anonymize(),
                value: (),
            },
        })
    }
}

impl LowerWhereClauses for StructDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerTypeKind for TraitDefn {
    fn lower_type_kind(&self) -> Fallible<rust_ir::TypeKind> {
        let binders: Vec<_> = self.parameter_kinds.iter().map(|p| p.lower()).collect();
        Ok(rust_ir::TypeKind {
            sort: rust_ir::TypeSort::Trait,
            name: self.name.str,
            binders: chalk_ir::Binders {
                // for the purposes of the *type*, ignore `Self`:
                binders: binders.anonymize(),
                value: (),
            },
        })
    }
}

impl LowerWhereClauses for TraitDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerWhereClauses for Impl {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

trait LowerWhereClauseVec {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>>;
}

impl LowerWhereClauseVec for [QuantifiedWhereClause] {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        self.iter()
            .flat_map(|wc| match wc.lower(env) {
                Ok(v) => v.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            })
            .collect()
    }
}

trait LowerWhereClause<T> {
    /// Lower from an AST `where` clause to an internal IR.
    /// Some AST `where` clauses can lower to multiple ones, this is why we return a `Vec`.
    /// As for now, this is the only the case for `where T: Foo<Item = U>` which lowers to
    /// `Implemented(T: Foo)` and `ProjectionEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> Fallible<Vec<T>>;
}

impl LowerWhereClause<chalk_ir::WhereClause<ChalkIr>> for WhereClause {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::WhereClause<ChalkIr>>> {
        let where_clauses = match self {
            WhereClause::Implemented { trait_ref } => {
                vec![chalk_ir::WhereClause::Implemented(trait_ref.lower(env)?)]
            }
            WhereClause::ProjectionEq { projection, ty } => vec![
                chalk_ir::WhereClause::ProjectionEq(chalk_ir::ProjectionEq {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                }),
                chalk_ir::WhereClause::Implemented(projection.trait_ref.lower(env)?),
            ],
        };
        Ok(where_clauses)
    }
}
impl LowerWhereClause<chalk_ir::QuantifiedWhereClause<ChalkIr>> for QuantifiedWhereClause {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.where_clause.lower(env)?))?;
        Ok(binders.into_iter().collect())
    }
}

trait LowerDomainGoal {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::DomainGoal<ChalkIr>>>;
}

impl LowerDomainGoal for DomainGoal {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::DomainGoal<ChalkIr>>> {
        let goals = match self {
            DomainGoal::Holds { where_clause } => {
                where_clause.lower(env)?.into_iter().casted().collect()
            }
            DomainGoal::Normalize { projection, ty } => {
                vec![chalk_ir::DomainGoal::Normalize(chalk_ir::Normalize {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                })]
            }
            DomainGoal::TyWellFormed { ty } => vec![chalk_ir::DomainGoal::WellFormed(
                chalk_ir::WellFormed::Ty(ty.lower(env)?),
            )],
            DomainGoal::TraitRefWellFormed { trait_ref } => vec![chalk_ir::DomainGoal::WellFormed(
                chalk_ir::WellFormed::Trait(trait_ref.lower(env)?),
            )],
            DomainGoal::TyFromEnv { ty } => vec![chalk_ir::DomainGoal::FromEnv(
                chalk_ir::FromEnv::Ty(ty.lower(env)?),
            )],
            DomainGoal::TraitRefFromEnv { trait_ref } => vec![chalk_ir::DomainGoal::FromEnv(
                chalk_ir::FromEnv::Trait(trait_ref.lower(env)?),
            )],
            DomainGoal::IsLocal { ty } => vec![chalk_ir::DomainGoal::IsLocal(ty.lower(env)?)],
            DomainGoal::IsUpstream { ty } => vec![chalk_ir::DomainGoal::IsUpstream(ty.lower(env)?)],
            DomainGoal::IsFullyVisible { ty } => {
                vec![chalk_ir::DomainGoal::IsFullyVisible(ty.lower(env)?)]
            }
            DomainGoal::LocalImplAllowed { trait_ref } => {
                vec![chalk_ir::DomainGoal::LocalImplAllowed(
                    trait_ref.lower(env)?,
                )]
            }
            DomainGoal::Compatible => vec![chalk_ir::DomainGoal::Compatible(())],
            DomainGoal::DownstreamType { ty } => {
                vec![chalk_ir::DomainGoal::DownstreamType(ty.lower(env)?)]
            }
        };
        Ok(goals)
    }
}

trait LowerLeafGoal {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::LeafGoal<ChalkIr>>>;
}

impl LowerLeafGoal for LeafGoal {
    fn lower(&self, env: &Env) -> Fallible<Vec<chalk_ir::LeafGoal<ChalkIr>>> {
        let goals = match self {
            LeafGoal::DomainGoal { goal } => goal
                .lower(env)?
                .into_iter()
                .map(|goal| chalk_ir::LeafGoal::DomainGoal(goal))
                .collect(),
            LeafGoal::UnifyTys { a, b } => vec![chalk_ir::EqGoal {
                a: a.lower(env)?.cast(),
                b: b.lower(env)?.cast(),
            }
            .cast()],
            LeafGoal::UnifyLifetimes { ref a, ref b } => vec![chalk_ir::EqGoal {
                a: a.lower(env)?.cast(),
                b: b.lower(env)?.cast(),
            }
            .cast()],
        };
        Ok(goals)
    }
}

trait LowerStructDefn {
    fn lower_struct(
        &self,
        struct_id: chalk_ir::StructId,
        env: &Env,
    ) -> Fallible<rust_ir::StructDatum>;
}

impl LowerStructDefn for StructDefn {
    fn lower_struct(
        &self,
        struct_id: chalk_ir::StructId,
        env: &Env,
    ) -> Fallible<rust_ir::StructDatum> {
        if self.flags.fundamental && self.all_parameters().len() != 1 {
            return Err(format_err!(
                "Only fundamental types with a single parameter are supported"
            ));
        }

        let binders = env.in_binders(self.all_parameters(), |env| {
            let fields: Fallible<_> = self.fields.iter().map(|f| f.ty.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;

            Ok(rust_ir::StructDatumBound {
                fields: fields?,
                where_clauses,
            })
        })?;

        let flags = rust_ir::StructFlags {
            upstream: self.flags.upstream,
            fundamental: self.flags.fundamental,
        };

        Ok(rust_ir::StructDatum {
            id: struct_id,
            binders,
            flags,
        })
    }
}

fn check_type_kinds<A: Kinded, B: Kinded>(msg: &str, expected: &A, actual: &B) -> Fallible<()> {
    let expected_kind = expected.kind();
    let actual_kind = actual.kind();
    if expected_kind != actual_kind {
        Err(format_err!(
            "{}: expected {}, found {}",
            msg,
            expected_kind,
            actual_kind
        ))
    } else {
        Ok(())
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::TraitRef<ChalkIr>>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::TraitRef<ChalkIr>> {
        let without_self = TraitBound {
            trait_name: self.trait_name,
            args_no_self: self.args.iter().cloned().skip(1).collect(),
        }
        .lower(env)?;

        let self_parameter = self.args[0].lower(env)?;
        Ok(without_self.as_trait_ref(self_parameter.ty().unwrap()))
    }
}

trait LowerTraitBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::TraitBound>;
}

impl LowerTraitBound for TraitBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::TraitBound> {
        let trait_id = match env.lookup(self.trait_name)? {
            NameLookup::Type(TypeKindId::TraitId(trait_id)) => trait_id,
            NameLookup::Type(_) | NameLookup::Parameter(_) => {
                Err(RustIrError::NotTrait(self.trait_name))?
            }
        };

        let k = env.type_kind(trait_id.into());
        if k.sort != rust_ir::TypeSort::Trait {
            Err(RustIrError::NotTrait(self.trait_name))?;
        }

        let parameters = self
            .args_no_self
            .iter()
            .map(|a| Ok(a.lower(env)?))
            .collect::<Fallible<Vec<_>>>()?;

        if parameters.len() != k.binders.len() {
            return Err(format_err!(
                "wrong number of parameters, expected `{:?}`, got `{:?}`",
                k.binders.len(),
                parameters.len()
            ));
        }

        for (binder, param) in k.binders.binders.iter().zip(parameters.iter()) {
            check_type_kinds("incorrect kind for trait parameter", binder, param)?;
        }

        Ok(rust_ir::TraitBound {
            trait_id,
            args_no_self: parameters,
        })
    }
}

trait LowerProjectionEqBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::ProjectionEqBound>;
}

impl LowerProjectionEqBound for ProjectionEqBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::ProjectionEqBound> {
        let trait_bound = self.trait_bound.lower(env)?;
        let lookup = match env
            .associated_ty_lookups
            .get(&(trait_bound.trait_id, self.name.str))
        {
            Some(lookup) => lookup,
            None => {
                return Err(format_err!(
                    "no associated type `{}` defined in trait",
                    self.name.str
                ));
            }
        };
        let args: Vec<_> = self
            .args
            .iter()
            .map(|a| a.lower(env))
            .collect::<Fallible<_>>()?;

        if args.len() != lookup.addl_parameter_kinds.len() {
            return Err(format_err!(
                "wrong number of parameters for associated type (expected {}, got {})",
                lookup.addl_parameter_kinds.len(),
                args.len()
            ));
        }

        for (param, arg) in lookup.addl_parameter_kinds.iter().zip(args.iter()) {
            check_type_kinds("incorrect kind for associated type parameter", param, arg)?;
        }

        Ok(rust_ir::ProjectionEqBound {
            trait_bound,
            associated_ty_id: lookup.id,
            parameters: args,
            value: self.value.lower(env)?,
        })
    }
}

trait LowerInlineBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::InlineBound>;
}

impl LowerInlineBound for InlineBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::InlineBound> {
        let bound = match self {
            InlineBound::TraitBound(b) => rust_ir::InlineBound::TraitBound(b.lower(&env)?),
            InlineBound::ProjectionEqBound(b) => {
                rust_ir::InlineBound::ProjectionEqBound(b.lower(&env)?)
            }
        };
        Ok(bound)
    }
}

trait LowerQuantifiedInlineBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::QuantifiedInlineBound>;
}

impl LowerQuantifiedInlineBound for QuantifiedInlineBound {
    fn lower(&self, env: &Env) -> Fallible<rust_ir::QuantifiedInlineBound> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.bound.lower(env)?))?;
        Ok(binders)
    }
}

trait LowerQuantifiedInlineBoundVec {
    fn lower(&self, env: &Env) -> Fallible<Vec<rust_ir::QuantifiedInlineBound>>;
}

impl LowerQuantifiedInlineBoundVec for [QuantifiedInlineBound] {
    fn lower(&self, env: &Env) -> Fallible<Vec<rust_ir::QuantifiedInlineBound>> {
        self.iter().map(|b| b.lower(env)).collect()
    }
}

trait LowerPolarity {
    fn lower(&self) -> rust_ir::Polarity;
}

impl LowerPolarity for Polarity {
    fn lower(&self) -> rust_ir::Polarity {
        match *self {
            Polarity::Positive => rust_ir::Polarity::Positive,
            Polarity::Negative => rust_ir::Polarity::Negative,
        }
    }
}

trait LowerImplType {
    fn lower(&self) -> rust_ir::ImplType;
}

impl LowerImplType for ImplType {
    fn lower(&self) -> rust_ir::ImplType {
        match self {
            ImplType::Local => rust_ir::ImplType::Local,
            ImplType::External => rust_ir::ImplType::External,
        }
    }
}

trait LowerTraitFlags {
    fn lower(&self) -> rust_ir::TraitFlags;
}

impl LowerTraitFlags for TraitFlags {
    fn lower(&self) -> rust_ir::TraitFlags {
        rust_ir::TraitFlags {
            auto: self.auto,
            marker: self.marker,
            upstream: self.upstream,
            fundamental: self.fundamental,
            non_enumerable: self.non_enumerable,
        }
    }
}

trait LowerProjectionTy {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::ProjectionTy<ChalkIr>>;
}

impl LowerProjectionTy for ProjectionTy {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::ProjectionTy<ChalkIr>> {
        let ProjectionTy {
            ref trait_ref,
            ref name,
            ref args,
        } = *self;
        let chalk_ir::TraitRef {
            trait_id,
            parameters: trait_parameters,
        } = trait_ref.lower(env)?;
        let lookup = match env.associated_ty_lookups.get(&(trait_id.into(), name.str)) {
            Some(lookup) => lookup,
            None => {
                return Err(format_err!(
                    "no associated type `{}` defined in trait",
                    name.str
                ));
            }
        };
        let mut args: Vec<_> = args.iter().map(|a| a.lower(env)).collect::<Fallible<_>>()?;

        if args.len() != lookup.addl_parameter_kinds.len() {
            return Err(format_err!(
                "wrong number of parameters for associated type (expected {}, got {})",
                lookup.addl_parameter_kinds.len(),
                args.len()
            ));
        }

        for (param, arg) in lookup.addl_parameter_kinds.iter().zip(args.iter()) {
            check_type_kinds("incorrect kind for associated type parameter", param, arg)?;
        }

        args.extend(trait_parameters);

        Ok(chalk_ir::ProjectionTy {
            associated_ty_id: lookup.id,
            parameters: args,
        })
    }
}

trait LowerTy {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Ty<ChalkIr>>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Ty<ChalkIr>> {
        match *self {
            Ty::Id { name } => match env.lookup(name)? {
                NameLookup::Type(id) => {
                    let k = env.type_kind(id);
                    if k.binders.len() > 0 {
                        Err(RustIrError::IncorrectNumberOfTypeParameters {
                            identifier: name,
                            expected: k.binders.len(),
                            actual: 0,
                        })?
                    } else {
                        Ok(chalk_ir::Ty::Apply(chalk_ir::ApplicationTy {
                            name: chalk_ir::TypeName::TypeKindId(id.into()),
                            parameters: vec![],
                        }))
                    }
                }
                NameLookup::Parameter(d) => Ok(chalk_ir::Ty::BoundVar(d)),
            },

            Ty::Dyn { ref bounds } => Ok(chalk_ir::Ty::Dyn(env.in_binders(
                // FIXME: Figure out a proper name for this type parameter
                Some(chalk_ir::ParameterKind::Ty(intern(FIXME_SELF))),
                |env| {
                    Ok(bounds
                        .lower(env)?
                        .iter()
                        .flat_map(|qil| qil.into_where_clauses(chalk_ir::Ty::BoundVar(0)))
                        .collect())
                },
            )?)),

            Ty::Opaque { ref bounds } => Ok(chalk_ir::Ty::Opaque(env.in_binders(
                // FIXME: Figure out a proper name for this type parameter
                Some(chalk_ir::ParameterKind::Ty(intern(FIXME_SELF))),
                |env| {
                    Ok(bounds
                        .lower(env)?
                        .iter()
                        .flat_map(|qil| qil.into_where_clauses(chalk_ir::Ty::BoundVar(0)))
                        .collect())
                },
            )?)),

            Ty::Apply { name, ref args } => {
                let id = match env.lookup(name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => Err(RustIrError::CannotApplyTypeParameter(name))?,
                };

                let k = env.type_kind(id);
                if k.binders.len() != args.len() {
                    Err(RustIrError::IncorrectNumberOfTypeParameters {
                        identifier: name,
                        expected: k.binders.len(),
                        actual: args.len(),
                    })?;
                }

                let parameters = args
                    .iter()
                    .map(|t| Ok(t.lower(env)?))
                    .collect::<Fallible<Vec<_>>>()?;

                for (param, arg) in k.binders.binders.iter().zip(args.iter()) {
                    check_type_kinds("incorrect parameter kind", param, arg)?;
                }

                Ok(chalk_ir::Ty::Apply(chalk_ir::ApplicationTy {
                    name: chalk_ir::TypeName::TypeKindId(id.into()),
                    parameters: parameters,
                }))
            }

            Ty::Projection { ref proj } => Ok(chalk_ir::Ty::Projection(proj.lower(env)?)),

            Ty::ForAll {
                ref lifetime_names,
                ref ty,
            } => {
                let quantified_env = env.introduce(
                    lifetime_names
                        .iter()
                        .map(|id| chalk_ir::ParameterKind::Lifetime(id.str)),
                )?;

                let ty = ty.lower(&quantified_env)?;
                let quantified_ty = chalk_ir::QuantifiedTy {
                    num_binders: lifetime_names.len(),
                    ty,
                };
                Ok(chalk_ir::Ty::ForAll(Box::new(quantified_ty)))
            }
        }
    }
}

trait LowerParameter {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Parameter<ChalkIr>>;
}

impl LowerParameter for Parameter {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Parameter<ChalkIr>> {
        match *self {
            Parameter::Ty(ref t) => Ok(t.lower(env)?.cast()),
            Parameter::Lifetime(ref l) => Ok(l.lower(env)?.cast()),
        }
    }
}

trait LowerLifetime {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Lifetime<ChalkIr>>;
}

impl LowerLifetime for Lifetime {
    fn lower(&self, env: &Env) -> Fallible<chalk_ir::Lifetime<ChalkIr>> {
        match *self {
            Lifetime::Id { name } => match env.lookup_lifetime(name)? {
                LifetimeLookup::Parameter(d) => Ok(chalk_ir::Lifetime::BoundVar(d)),
            },
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, empty_env: &Env, impl_id: ImplId) -> Fallible<rust_ir::ImplDatum>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, empty_env: &Env, impl_id: ImplId) -> Fallible<rust_ir::ImplDatum> {
        let polarity = self.polarity.lower();
        let binders = empty_env.in_binders(self.all_parameters(), |env| {
            let trait_ref = self.trait_ref.lower(env)?;

            if !polarity.is_positive() && !self.assoc_ty_values.is_empty() {
                return Err(format_err!(
                    "negative impls cannot define associated values"
                ));
            }

            let trait_id = trait_ref.trait_id;
            let where_clauses = self.lower_where_clauses(&env)?;
            let associated_ty_values = self
                .assoc_ty_values
                .iter()
                .map(|v| v.lower(trait_id.into(), impl_id, env))
                .collect::<Fallible<_>>()?;
            Ok(rust_ir::ImplDatumBound {
                trait_ref,
                where_clauses,
                associated_ty_values,
            })
        })?;

        Ok(rust_ir::ImplDatum {
            polarity,
            binders: binders,
            impl_type: self.impl_type.lower(),
        })
    }
}

trait LowerClause {
    fn lower_clause(&self, env: &Env) -> Fallible<Vec<chalk_ir::ProgramClause<ChalkIr>>>;
}

impl LowerClause for Clause {
    fn lower_clause(&self, env: &Env) -> Fallible<Vec<chalk_ir::ProgramClause<ChalkIr>>> {
        let implications = env.in_binders(self.all_parameters(), |env| {
            let consequences: Vec<chalk_ir::DomainGoal<ChalkIr>> = self.consequence.lower(env)?;

            let conditions: Vec<chalk_ir::Goal<ChalkIr>> = self
                .conditions
                .iter()
                .map(|g| g.lower(env).map(|g| *g))
                .rev() // (*)
                .collect::<Fallible<_>>()?;

            // (*) Subtle: in the SLG solver, we pop conditions from R to
            // L. To preserve the expected order (L to R), we must
            // therefore reverse.

            let implications = consequences
                .into_iter()
                .map(|consequence| chalk_ir::ProgramClauseImplication {
                    consequence,
                    conditions: conditions.clone(),
                })
                .collect::<Vec<_>>();
            Ok(implications)
        })?;

        let clauses = implications
            .into_iter()
            .map(
                |implication: chalk_ir::Binders<chalk_ir::ProgramClauseImplication<ChalkIr>>| {
                    if implication.binders.is_empty() {
                        chalk_ir::ProgramClause::Implies(implication.value)
                    } else {
                        chalk_ir::ProgramClause::ForAll(implication)
                    }
                },
            )
            .collect();
        Ok(clauses)
    }
}

trait LowerAssocTyValue {
    fn lower(
        &self,
        trait_id: chalk_ir::TraitId,
        impl_id: chalk_ir::ImplId,
        env: &Env,
    ) -> Fallible<rust_ir::AssociatedTyValue>;
}

impl LowerAssocTyValue for AssocTyValue {
    fn lower(
        &self,
        trait_id: chalk_ir::TraitId,
        impl_id: chalk_ir::ImplId,
        env: &Env,
    ) -> Fallible<rust_ir::AssociatedTyValue> {
        let lookup = &env.associated_ty_lookups[&(trait_id, self.name.str)];
        let value = env.in_binders(self.all_parameters(), |env| {
            Ok(rust_ir::AssociatedTyValueBound {
                ty: self.value.lower(env)?,
            })
        })?;
        Ok(rust_ir::AssociatedTyValue {
            impl_id,
            associated_ty_id: lookup.id,
            value: value,
        })
    }
}

trait LowerTrait {
    fn lower_trait(&self, trait_id: chalk_ir::TraitId, env: &Env) -> Fallible<rust_ir::TraitDatum>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(&self, trait_id: chalk_ir::TraitId, env: &Env) -> Fallible<rust_ir::TraitDatum> {
        let all_parameters = self.all_parameters();
        let all_parameters_len = all_parameters.len();
        let binders = env.in_binders(all_parameters, |env| {
            if self.flags.auto {
                if all_parameters_len > 1 {
                    return Err(format_err!("auto trait cannot have parameters"));
                }
                if !self.where_clauses.is_empty() {
                    return Err(format_err!("auto trait cannot have where clauses"));
                }
            }

            Ok(rust_ir::TraitDatumBound {
                where_clauses: self.lower_where_clauses(env)?,
            })
        })?;

        let associated_ty_ids: Vec<_> = self
            .assoc_ty_defns
            .iter()
            .map(|defn| env.associated_ty_lookups[&(trait_id, defn.name.str)].id)
            .collect();

        Ok(rust_ir::TraitDatum {
            id: trait_id,
            binders: binders,
            flags: self.flags.lower(),
            associated_ty_ids,
        })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> Fallible<Box<chalk_ir::Goal<ChalkIr>>>;
}

impl LowerGoal<LoweredProgram> for Goal {
    fn lower(&self, program: &LoweredProgram) -> Fallible<Box<chalk_ir::Goal<ChalkIr>>> {
        let associated_ty_lookups: BTreeMap<_, _> = program
            .associated_ty_data
            .iter()
            .map(|(&associated_ty_id, datum)| {
                let trait_datum = &program.trait_data[&datum.trait_id];
                let num_trait_params = trait_datum.binders.len();
                let num_addl_params = datum.binders.len() - num_trait_params;
                let addl_parameter_kinds = datum.binders.binders[..num_addl_params].to_owned();
                let lookup = AssociatedTyLookup {
                    id: associated_ty_id,
                    addl_parameter_kinds,
                };
                ((datum.trait_id, datum.name), lookup)
            })
            .collect();

        let env = Env {
            type_ids: &program.type_ids,
            type_kinds: &program.type_kinds,
            associated_ty_lookups: &associated_ty_lookups,
            parameter_map: BTreeMap::new(),
        };

        self.lower(&env)
    }
}

impl<'k> LowerGoal<Env<'k>> for Goal {
    fn lower(&self, env: &Env<'k>) -> Fallible<Box<chalk_ir::Goal<ChalkIr>>> {
        match self {
            Goal::ForAll(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::ForAll, ids),
            Goal::Exists(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::Exists, ids),
            Goal::Implies(hyp, g) => {
                // We "elaborate" implied bounds by lowering goals like `T: Trait` and
                // `T: Trait<Assoc = U>` to `FromEnv(T: Trait)` and `FromEnv(T: Trait<Assoc = U>)`
                // in the assumptions of an `if` goal, e.g. `if (T: Trait) { ... }` lowers to
                // `if (FromEnv(T: Trait)) { ... /* this part is untouched */ ... }`.
                let where_clauses: Fallible<Vec<_>> = hyp
                    .into_iter()
                    .flat_map(|h| h.lower_clause(env).apply_result())
                    .map(|result| result.map(|h| h.into_from_env_clause()))
                    .collect();
                Ok(Box::new(chalk_ir::Goal::Implies(
                    where_clauses?,
                    g.lower(env)?,
                )))
            }
            Goal::And(g1, g2) => Ok(Box::new(chalk_ir::Goal::And(
                g1.lower(env)?,
                g2.lower(env)?,
            ))),
            Goal::Not(g) => Ok(Box::new(chalk_ir::Goal::Not(g.lower(env)?))),
            Goal::Compatible(g) => Ok(Box::new(g.lower(env)?.compatible())),
            Goal::Leaf(leaf) => {
                // A where clause can lower to multiple leaf goals; wrap these in Goal::And.
                let leaves = leaf.lower(env)?.into_iter().map(chalk_ir::Goal::Leaf);
                let goal = leaves
                    .fold1(|goal, leaf| chalk_ir::Goal::And(Box::new(goal), Box::new(leaf)))
                    .expect("at least one goal");
                Ok(Box::new(goal))
            }
        }
    }
}

trait LowerQuantifiedGoal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Fallible<Box<chalk_ir::Goal<ChalkIr>>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Fallible<Box<chalk_ir::Goal<ChalkIr>>> {
        if parameter_kinds.is_empty() {
            return self.lower(env);
        }

        let parameter_kinds = parameter_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(parameter_kinds, |env| self.lower(env))?;
        Ok(Box::new(chalk_ir::Goal::Quantified(
            quantifier_kind,
            subgoal,
        )))
    }
}

/// Lowers Fallible<Vec<T>> -> Vec<Fallible<T>>.
trait ApplyResult {
    type Output;
    fn apply_result(self) -> Self::Output;
}

impl<T> ApplyResult for Fallible<Vec<T>> {
    type Output = Vec<Fallible<T>>;
    fn apply_result(self) -> Self::Output {
        match self {
            Ok(v) => v.into_iter().map(Ok).collect(),
            Err(e) => vec![Err(e)],
        }
    }
}

trait Kinded {
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

impl<T, L> Kinded for chalk_ir::ParameterKind<T, L> {
    fn kind(&self) -> Kind {
        match *self {
            chalk_ir::ParameterKind::Ty(_) => Kind::Ty,
            chalk_ir::ParameterKind::Lifetime(_) => Kind::Lifetime,
        }
    }
}

impl Kinded for chalk_ir::Parameter<ChalkIr> {
    fn kind(&self) -> Kind {
        self.0.kind()
    }
}
